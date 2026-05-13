// Forked and refactored from https://github.com/0xPlaygrounds/rig/blob/rig-core-v0.31.0/rig/rig-core/src/client/builder.rs
use delegate::delegate;
use disjoint_impls::disjoint_impls;
use kinded::Kinded;
use rig_core::client::ProviderClientError;
use std::collections::HashMap;

use rig_core::{
    agent::AgentBuilder,
    client::{Capabilities, Capable, Client, Nothing, ProviderClient},
    completion::{CompletionError, CompletionModel},
    providers::{
        anthropic, azure, cohere, deepseek, galadriel, gemini, groq, huggingface, hyperbolic, mira,
        mistral, moonshot, ollama, openai, openrouter, perplexity, together, xai,
    },
};

use crate::completion::{CompletionClientDyn, CompletionModelDyn, CompletionModelHandle};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Provider '{0}' not found")]
    NotFound(String),

    #[error("Provider '{provider}' cannot be coerced to a '{role}'")]
    NotCapable { provider: String, role: String },

    #[error("Error generating response\n{0}")]
    Completion(#[from] CompletionError),

    #[error("Error from provider client\n{0}")]
    Client(#[from] ProviderClientError),
}

disjoint_impls! {
    pub trait CompletionInflector {
        fn as_completion(&self) -> Option<&dyn CompletionClientDyn>;
    }

    impl<M, Ext, H> CompletionInflector for Client<Ext, H>
    where
        Ext: Capabilities<H, Completion = Capable<M>>,
        M: CompletionModel<Client = Self> + 'static,
    {
        fn as_completion(&self) -> Option<&dyn CompletionClientDyn> {
            Some(self as &dyn CompletionClientDyn)
        }
    }
    impl<Ext, H> CompletionInflector for Client<Ext, H>
    where
        Ext: Capabilities<H, Completion = Nothing>,
    {
        fn as_completion(&self) -> Option<&dyn CompletionClientDyn> {
            None
        }
    }
}

#[derive(Kinded)]
#[kinded(kind=Provider, derive(Debug))]
pub enum AnyClient {
    Anthropic(anthropic::Client),
    Cohere(cohere::Client),
    Gemini(gemini::Client),
    HuggingFace(huggingface::Client),
    OpenAI(openai::Client),
    OpenRouter(openrouter::Client),
    Together(together::Client),
    XAI(xai::Client),
    Azure(azure::Client),
    DeepSeek(deepseek::Client),
    Galadriel(galadriel::Client),
    Groq(groq::Client),
    Hyperbolic(hyperbolic::Client),
    Moonshot(moonshot::Client),
    Mira(mira::Client),
    Mistral(mistral::Client),
    Ollama(ollama::Client),
    Perplexity(perplexity::Client),
}

impl Provider {
    pub fn from_env(&self) -> Result<AnyClient, ProviderClientError> {
        use AnyClient::*;
        Ok(match self {
            Provider::Anthropic => Anthropic(anthropic::Client::from_env()?),
            Provider::Cohere => Cohere(cohere::Client::from_env()?),
            Provider::Gemini => Gemini(gemini::Client::from_env()?),
            Provider::HuggingFace => HuggingFace(huggingface::Client::from_env()?),
            Provider::OpenAI => OpenAI(openai::Client::from_env()?),
            Provider::OpenRouter => OpenRouter(openrouter::Client::from_env()?),
            Provider::Together => Together(together::Client::from_env()?),
            Provider::XAI => XAI(xai::Client::from_env()?),
            Provider::Azure => Azure(azure::Client::from_env()?),
            Provider::DeepSeek => DeepSeek(deepseek::Client::from_env()?),
            Provider::Galadriel => Galadriel(galadriel::Client::from_env()?),
            Provider::Groq => Groq(groq::Client::from_env()?),
            Provider::Hyperbolic => Hyperbolic(hyperbolic::Client::from_env()?),
            Provider::Moonshot => Moonshot(moonshot::Client::from_env()?),
            Provider::Mira => Mira(mira::Client::from_env()?),
            Provider::Mistral => Mistral(mistral::Client::from_env()?),
            Provider::Ollama => Ollama(ollama::Client::from_env()?),
            Provider::Perplexity => Perplexity(perplexity::Client::from_env()?),
        })
    }
}

impl AnyClient {
    delegate! {
        to match self {
            AnyClient::Anthropic(client) => client,
            AnyClient::Cohere(client) => client,
            AnyClient::Gemini(client) => client,
            AnyClient::HuggingFace(client) => client,
            AnyClient::OpenAI(client) => client,
            AnyClient::OpenRouter(client) => client,
            AnyClient::Together(client) => client,
            AnyClient::XAI(client) => client,
            AnyClient::Azure(client) => client,
            AnyClient::DeepSeek(client) => client,
            AnyClient::Galadriel(client) => client,
            AnyClient::Groq(client) => client,
            AnyClient::Hyperbolic(client) => client,
            AnyClient::Moonshot(client) => client,
            AnyClient::Mira(client) => client,
            AnyClient::Mistral(client) => client,
            AnyClient::Ollama(client) => client,
            AnyClient::Perplexity(client) => client,
        } {
            pub fn as_completion(&self) -> Option<&dyn CompletionClientDyn>;
        }
    }

    pub fn name(&self) -> String {
        self.kind().to_string().to_lowercase()
    }
}

#[derive(Debug, Clone)]
pub struct DynClientBuilder(HashMap<String, Provider>);

impl Default for DynClientBuilder {
    fn default() -> Self {
        // Give it a capacity ~the number of providers we have from the start
        Self(HashMap::with_capacity(32))
    }
}

impl DynClientBuilder {
    pub fn new() -> Self {
        Self::default().register_all()
    }

    fn register_all(mut self) -> Self {
        for provider in Provider::all() {
            self.0
                .insert(provider.to_string().to_lowercase(), *provider);
        }

        self
    }

    pub fn from_env<T, Models>(
        &self,
        provider_name: &'static str,
        _model: Models,
    ) -> Result<AnyClient, Error>
    where
        T: 'static,
        Models: ToString,
    {
        self.0
            .get(provider_name)
            .ok_or_else(|| Error::NotFound(provider_name.into()))
            .and_then(|kind| kind.from_env().map_err(Error::from))
    }

    /// Get a boxed agent based on the provider and model, as well as an API key.
    pub fn agent<Models>(
        &self,
        provider_name: impl Into<&'static str>,
        model: Models,
    ) -> Result<AgentBuilder<CompletionModelHandle<'_>>, Error>
    where
        Models: ToString,
    {
        let provider_name = provider_name.into();

        let client = self
            .0
            .get(provider_name)
            .ok_or_else(|| Error::NotFound(provider_name.into()))
            .and_then(|kind| kind.from_env().map_err(Error::from))?;

        let completion = client.as_completion().ok_or(Error::NotCapable {
            provider: provider_name.into(),
            role: "Completion".into(),
        })?;

        Ok(completion.agent(&model.to_string()))
    }

    /// Get a boxed completion model based on the provider and model.
    pub fn completion<Models>(
        &self,
        provider_name: &'static str,
        model: Models,
    ) -> Result<Box<dyn CompletionModelDyn>, Error>
    where
        Models: ToString,
    {
        let client = self
            .0
            .get(provider_name)
            .ok_or_else(|| Error::NotFound(provider_name.into()))
            .and_then(|kind| kind.from_env().map_err(Error::from))?;

        let completion = client.as_completion().ok_or(Error::NotCapable {
            provider: provider_name.into(),
            role: "Completion Model".into(),
        })?;

        Ok(completion.completion_model(&model.to_string()))
    }
}
