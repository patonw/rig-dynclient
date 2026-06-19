// Forked and refactored from https://github.com/0xPlaygrounds/rig/blob/rig-core-v0.31.0/rig/rig-core/src/client/builder.rs
use delegate::delegate;
use disjoint_impls::disjoint_impls;
use enum_assoc::Assoc;
use kinded::Kinded;
use rig_core::client::ProviderClientError;
use std::{collections::HashMap, str::FromStr};

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

    #[error("Environment Variable '{0}' not found")]
    MissingEnv(String),

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
#[kinded(kind=Provider, derive(Debug, Assoc), attrs(
    func(pub fn api_key(&self) -> &'static str),
    func(pub fn required(&self) -> bool { true })
))]
pub enum AnyClient {
    #[kinded(attrs(assoc(api_key = "ANTHROPIC_API_KEY")))]
    Anthropic(anthropic::Client),
    #[kinded(attrs(assoc(api_key = "COHERE_API_KEY")))]
    Cohere(cohere::Client),
    #[kinded(attrs(assoc(api_key = "GEMINI_API_KEY")))]
    Gemini(gemini::Client),
    #[kinded(attrs(assoc(api_key = "HUGGINGFACE_API_KEY")))]
    HuggingFace(huggingface::Client),
    #[kinded(attrs(assoc(api_key = "OPENAI_API_KEY")))]
    OpenAI(openai::Client),
    #[kinded(attrs(assoc(api_key = "OPENROUTER_API_KEY")))]
    OpenRouter(openrouter::Client),
    #[kinded(attrs(assoc(api_key = "TOGETHER_API_KEY")))]
    Together(together::Client),
    #[kinded(attrs(assoc(api_key = "XAI_API_KEY")))]
    XAI(xai::Client),
    #[kinded(attrs(assoc(api_key = "AZURE_API_KEY")))]
    Azure(azure::Client),
    #[kinded(attrs(assoc(api_key = "DEEPSEEK_API_KEY")))]
    DeepSeek(deepseek::Client),
    #[kinded(attrs(assoc(api_key = "GALADRIEL_API_KEY")))]
    Galadriel(galadriel::Client),
    #[kinded(attrs(assoc(api_key = "GROQ_API_KEY")))]
    Groq(groq::Client),
    #[kinded(attrs(assoc(api_key = "HYPERBOLIC_API_KEY")))]
    Hyperbolic(hyperbolic::Client),
    #[kinded(attrs(assoc(api_key = "MOONSHOT_API_KEY")))]
    Moonshot(moonshot::Client),
    #[kinded(attrs(assoc(api_key = "MIRA_API_KEY")))]
    Mira(mira::Client),
    #[kinded(attrs(assoc(api_key = "MISTRAL_API_KEY")))]
    Mistral(mistral::Client),
    #[kinded(attrs(assoc(api_key = "OLLAMA_API_KEY", required = false)))]
    Ollama(ollama::Client),
    #[kinded(attrs(assoc(api_key = "PERPLEXITY_API_KEY")))]
    Perplexity(perplexity::Client),
}

impl Provider {
    #[deprecated]
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

    pub fn from_key(&self, key: &str) -> Result<AnyClient, ProviderClientError> {
        use AnyClient::*;
        Ok(match self {
            Provider::Anthropic => Anthropic(anthropic::Client::from_val(key.into())?),
            Provider::Cohere => Cohere(cohere::Client::from_val(key.into())?),
            Provider::Gemini => Gemini(gemini::Client::from_val(key.into())?),
            Provider::HuggingFace => HuggingFace(huggingface::Client::from_val(key.into())?),
            Provider::OpenAI => OpenAI(openai::Client::from_val(key.into())?),
            Provider::OpenRouter => OpenRouter(openrouter::Client::from_val(key.into())?),
            Provider::Together => Together(together::Client::from_val(key.into())?),
            Provider::XAI => XAI(xai::Client::from_val(key.into())?),
            Provider::Azure => {
                let api_version = rig_core::client::required_env_var("AZURE_API_VERSION")?;
                let azure_endpoint = rig_core::client::required_env_var("AZURE_ENDPOINT")?;
                let auth = azure::AzureOpenAIAuth::ApiKey(key.to_string());
                let client = azure::Client::builder()
                    .api_key(auth)
                    .azure_endpoint(azure_endpoint)
                    .api_version(&api_version)
                    .build()?;
                Azure(client)
            }
            Provider::DeepSeek => DeepSeek(deepseek::Client::from_val(key.into())?),
            Provider::Galadriel => Galadriel(galadriel::Client::from_val((key.into(), None))?),
            Provider::Groq => Groq(groq::Client::from_val(key.into())?),
            Provider::Hyperbolic => Hyperbolic(hyperbolic::Client::from_val(key.into())?),
            Provider::Moonshot => Moonshot(moonshot::Client::from_val(key.into())?),
            Provider::Mira => Mira(mira::Client::from_val(key.into())?),
            Provider::Mistral => Mistral(mistral::Client::from_val(key.into())?),
            Provider::Ollama => Ollama(ollama::Client::from_val(key.into())?),
            Provider::Perplexity => Perplexity(perplexity::Client::from_val(key.into())?),
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

    pub fn agent<'a, Models>(
        &self,
        model: Models,
    ) -> Result<AgentBuilder<CompletionModelHandle<'a>>, Error>
    where
        Models: ToString,
    {
        // let provider_name = provider_name.into();

        let completion = self.as_completion().ok_or(Error::NotCapable {
            provider: self.name(),
            role: "Completion".into(),
        })?;

        Ok(completion.agent(&model.to_string()))
    }

    /// Get a boxed completion model based on the provider and model.
    pub fn completion<Models>(&self, model: Models) -> Result<Box<dyn CompletionModelDyn>, Error>
    where
        Models: ToString,
    {
        let completion = self.as_completion().ok_or(Error::NotCapable {
            provider: self.name(),
            role: "Completion Model".into(),
        })?;

        Ok(completion.completion_model(&model.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct DynClientBuilder {
    env: HashMap<String, String>,
}

impl Default for DynClientBuilder {
    fn default() -> Self {
        Self::with_env(std::env::vars())
    }
}

impl DynClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_env(vars: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            env: vars.into_iter().collect(),
        }
    }

    #[deprecated = "Use `client(..., None)` instead"]
    pub fn from_env<T, Models>(
        &self,
        provider_name: &str,
        _model: Models,
    ) -> Result<AnyClient, Error>
    where
        T: 'static,
        Models: ToString,
    {
        let provider =
            Provider::from_str(provider_name).map_err(|_| Error::NotFound(provider_name.into()))?;

        #[allow(deprecated)]
        let client = provider.from_env().map_err(Error::from)?;

        Ok(client)
    }

    pub fn client(&self, provider_name: &str, api_key: Option<&str>) -> Result<AnyClient, Error> {
        let provider =
            Provider::from_str(provider_name).map_err(|_| Error::NotFound(provider_name.into()))?;

        let api_key = match api_key.or_else(|| self.env.get(provider.api_key()).map(String::as_str))
        {
            Some(key) => key,
            None if !provider.required() => "",
            _ => Err(Error::MissingEnv(provider.api_key().into()))?,
        };

        let client = provider.from_key(api_key).map_err(Error::from)?;

        Ok(client)
    }

    /// Get a boxed agent based on the provider and model, as well as an API key.
    pub fn agent<Models>(
        &self,
        provider_name: &str,
        model: Models,
    ) -> Result<AgentBuilder<CompletionModelHandle<'_>>, Error>
    where
        Models: ToString,
    {
        self.client(provider_name, None)?.agent(model)
    }

    /// Get a boxed completion model based on the provider and model.
    pub fn completion<Models>(
        &self,
        provider_name: &str,
        model: Models,
    ) -> Result<Box<dyn CompletionModelDyn>, Error>
    where
        Models: ToString,
    {
        self.client(provider_name, None)?.completion(model)
    }
}
