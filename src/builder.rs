// Forked and refactored from https://github.com/0xPlaygrounds/rig/blob/rig-core-v0.31.0/rig/rig-core/src/client/builder.rs
#![allow(deprecated)]
use delegate::delegate;
use disjoint_impls::disjoint_impls;
use kinded::Kinded;
use rig::{
    OneOrMany,
    agent::AgentBuilder,
    client::{
        Capabilities, Capable, Client, FinalCompletionResponse, Nothing, ProviderClient,
        completion::{CompletionClientDyn, CompletionModelHandle},
        embeddings::EmbeddingsClientDyn,
        transcription::TranscriptionClientDyn,
    },
    completion::CompletionModel,
    completion::{CompletionError, CompletionModelDyn, CompletionRequest},
    embeddings::EmbeddingModel,
    embeddings::EmbeddingModelDyn,
    message::Message,
    providers::{
        anthropic, azure, cohere, deepseek, galadriel, gemini, groq, huggingface, hyperbolic, mira,
        mistral, moonshot, ollama, openai, openrouter, perplexity, together, xai,
    },
    streaming::StreamingCompletionResponse,
    transcription::TranscriptionModel,
    transcription::TranscriptionModelDyn,
    wasm_compat::WasmCompatSend,
};
use std::collections::HashMap;

#[cfg(feature = "audio")]
use rig::{
    audio_generation::{AudioGenerationModel, AudioGenerationModelDyn},
    client::audio_generation::AudioGenerationClientDyn,
};

#[cfg(feature = "image")]
use rig::{
    client::image_generation::ImageGenerationClientDyn,
    image_generation::{ImageGenerationModel, ImageGenerationModelDyn},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Provider '{0}' not found")]
    NotFound(String),
    #[error("Provider '{provider}' cannot be coerced to a '{role}'")]
    NotCapable { provider: String, role: String },
    #[error("Error generating response\n{0}")]
    Completion(#[from] CompletionError),
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

disjoint_impls! {
    pub trait EmbeddingsInflector {
        fn as_embedding(&self) -> Option<&dyn EmbeddingsClientDyn>;
    }

    impl<M, Ext, H> EmbeddingsInflector for Client<Ext, H>
    where
        Ext: Capabilities<H, Embeddings = Capable<M>>,
        M: EmbeddingModel<Client = Self> + 'static,
    {
        fn as_embedding(&self) -> Option<&dyn EmbeddingsClientDyn> {
            Some(self as &dyn EmbeddingsClientDyn)
        }
    }

    impl<Ext, H> EmbeddingsInflector for Client<Ext, H>
    where
        Ext: Capabilities<H, Embeddings = Nothing>,
    {
        fn as_embedding(&self) -> Option<&dyn EmbeddingsClientDyn> {
            None
        }
    }
}

disjoint_impls! {
    pub trait TranscriptionInflector {
        fn as_transcription(&self) -> Option<&dyn TranscriptionClientDyn>;
    }

    impl<M, Ext, H> TranscriptionInflector for Client<Ext, H>
    where
        Ext: Capabilities<H, Transcription = Capable<M>>,
        M: TranscriptionModel<Client = Self> + 'static,
    {
        fn as_transcription(&self) -> Option<&dyn TranscriptionClientDyn> {
            Some(self as &dyn TranscriptionClientDyn)
        }
    }

    impl<Ext, H> TranscriptionInflector for Client<Ext, H>
    where
        Ext: Capabilities<H, Transcription = Nothing>,
    {
        fn as_transcription(&self) -> Option<&dyn TranscriptionClientDyn> {
            None
        }
    }
}

#[cfg(feature = "image")]
disjoint_impls! {
    pub trait ImageGenerationInflector {
        fn as_image_generation(&self) -> Option<&dyn ImageGenerationClientDyn>;
    }

    impl<M, Ext, H> ImageGenerationInflector for Client<Ext, H>
    where
        Ext: Capabilities<H, ImageGeneration = Capable<M>>,
        M: ImageGenerationModel<Client = Self> + 'static,
    {
        fn as_image_generation(&self) -> Option<&dyn ImageGenerationClientDyn> {
            Some(self as &dyn ImageGenerationClientDyn)
        }
    }

    impl<Ext, H> ImageGenerationInflector for Client<Ext, H>
    where
        Ext: Capabilities<H, ImageGeneration = Nothing>,
    {
        fn as_image_generation(&self) -> Option<&dyn ImageGenerationClientDyn> {
            None
        }
    }
}
// }

#[cfg(feature = "audio")]
disjoint_impls! {
    pub trait AudioGenerationInflector {
        fn as_audio_generation(&self) -> Option<&dyn AudioGenerationClientDyn>;
    }

    impl<M, Ext, H> AudioGenerationInflector for Client<Ext, H>
    where
        Ext: Capabilities<H, AudioGeneration = Capable<M>>,
        M: AudioGenerationModel<Client = Self> + 'static,
    {
        fn as_audio_generation(&self) -> Option<&dyn AudioGenerationClientDyn> {
            Some(self as &dyn AudioGenerationClientDyn)
        }
    }

    impl<Ext, H> AudioGenerationInflector for Client<Ext, H>
    where
        Ext: Capabilities<H, AudioGeneration = Nothing>,
    {
        fn as_audio_generation(&self) -> Option<&dyn AudioGenerationClientDyn> {
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
    pub fn from_env(&self) -> AnyClient {
        use AnyClient::*;
        match self {
            Provider::Anthropic => Anthropic(anthropic::Client::from_env()),
            Provider::Cohere => Cohere(cohere::Client::from_env()),
            Provider::Gemini => Gemini(gemini::Client::from_env()),
            Provider::HuggingFace => HuggingFace(huggingface::Client::from_env()),
            Provider::OpenAI => OpenAI(openai::Client::from_env()),
            Provider::OpenRouter => OpenRouter(openrouter::Client::from_env()),
            Provider::Together => Together(together::Client::from_env()),
            Provider::XAI => XAI(xai::Client::from_env()),
            Provider::Azure => Azure(azure::Client::from_env()),
            Provider::DeepSeek => DeepSeek(deepseek::Client::from_env()),
            Provider::Galadriel => Galadriel(galadriel::Client::from_env()),
            Provider::Groq => Groq(groq::Client::from_env()),
            Provider::Hyperbolic => Hyperbolic(hyperbolic::Client::from_env()),
            Provider::Moonshot => Moonshot(moonshot::Client::from_env()),
            Provider::Mira => Mira(mira::Client::from_env()),
            Provider::Mistral => Mistral(mistral::Client::from_env()),
            Provider::Ollama => Ollama(ollama::Client::from_env()),
            Provider::Perplexity => Perplexity(perplexity::Client::from_env()),
        }
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

            pub fn as_embedding(&self) -> Option<&dyn EmbeddingsClientDyn>;

            pub fn as_transcription(&self) -> Option<&dyn TranscriptionClientDyn>;

            #[cfg(feature = "image")]
            pub fn as_image_generation(&self) -> Option<&dyn ImageGenerationClientDyn>;

            #[cfg(feature = "audio")]
            pub fn as_audio_generation(&self) -> Option<&dyn AudioGenerationClientDyn>;
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
            .map(|kind| kind.from_env())
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
            .map(|kind| kind.from_env())?;

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
            .map(|kind| kind.from_env())?;

        let completion = client.as_completion().ok_or(Error::NotCapable {
            provider: provider_name.into(),
            role: "Completion Model".into(),
        })?;

        Ok(completion.completion_model(&model.to_string()))
    }

    /// Get a boxed embedding model based on the provider and model.
    pub fn embeddings<Models>(
        &self,
        provider_name: &'static str,
        model: Models,
    ) -> Result<Box<dyn EmbeddingModelDyn>, Error>
    where
        Models: ToString,
    {
        let client = self
            .0
            .get(provider_name)
            .ok_or_else(|| Error::NotFound(provider_name.into()))
            .map(|kind| kind.from_env())?;

        let embeddings = client.as_embedding().ok_or(Error::NotCapable {
            provider: provider_name.into(),
            role: "Embedding Model".into(),
        })?;

        Ok(embeddings.embedding_model(&model.to_string()))
    }

    /// Get a boxed transcription model based on the provider and model.
    pub fn transcription<Models>(
        &self,
        provider_name: &'static str,
        model: Models,
    ) -> Result<Box<dyn TranscriptionModelDyn>, Error>
    where
        Models: ToString,
    {
        let client = self
            .0
            .get(provider_name)
            .ok_or_else(|| Error::NotFound(provider_name.into()))
            .map(|kind| kind.from_env())?;

        let transcription = client.as_transcription().ok_or(Error::NotCapable {
            provider: provider_name.into(),
            role: "transcription model".into(),
        })?;

        Ok(transcription.transcription_model(&model.to_string()))
    }

    #[cfg(feature = "image")]
    pub fn image_generation<Models>(
        &self,
        provider_name: &'static str,
        model: Models,
    ) -> Result<Box<dyn ImageGenerationModelDyn>, Error>
    where
        Models: ToString,
    {
        let client = self
            .0
            .get(provider_name)
            .ok_or_else(|| Error::NotFound(provider_name.into()))
            .map(|kind| kind.from_env())?;

        let image_generation = client.as_image_generation().ok_or(Error::NotCapable {
            provider: provider_name.into(),
            role: "Image generation".into(),
        })?;

        Ok(image_generation.image_generation_model(&model.to_string()))
    }

    #[cfg(feature = "audio")]
    pub fn audio_generation<Models>(
        &self,
        provider_name: &'static str,
        model: Models,
    ) -> Result<Box<dyn AudioGenerationModelDyn>, Error>
    where
        Models: ToString,
    {
        let client = self
            .0
            .get(provider_name)
            .ok_or_else(|| Error::NotFound(provider_name.into()))
            .map(|kind| kind.from_env())?;

        let audio_generation = client.as_audio_generation().ok_or(Error::NotCapable {
            provider: provider_name.into(),
            role: "Image generation".into(),
        })?;

        Ok(audio_generation.audio_generation_model(&model.to_string()))
    }

    /// Stream a completion request to the specified provider and model.
    pub async fn stream_completion<Models>(
        &self,
        provider_name: &'static str,
        model: Models,
        request: CompletionRequest,
    ) -> Result<StreamingCompletionResponse<FinalCompletionResponse>, Error>
    where
        Models: ToString,
    {
        let completion = self.completion(provider_name, model)?;

        completion.stream(request).await.map_err(Error::Completion)
    }

    /// Stream a simple prompt to the specified provider and model.
    pub async fn stream_prompt<Models, Prompt>(
        &self,
        provider_name: impl Into<&'static str>,
        model: Models,
        prompt: Prompt,
    ) -> Result<StreamingCompletionResponse<FinalCompletionResponse>, Error>
    where
        Models: ToString,
        Prompt: Into<Message> + WasmCompatSend,
    {
        let completion = self.completion(provider_name.into(), model)?;

        let request = CompletionRequest {
            model: None,
            preamble: None,
            tools: vec![],
            documents: vec![],
            temperature: None,
            max_tokens: None,
            additional_params: None,
            tool_choice: None,
            chat_history: rig::OneOrMany::one(prompt.into()),
            output_schema: None,
        };

        completion.stream(request).await.map_err(Error::Completion)
    }

    /// Stream a chat with history to the specified provider and model.
    pub async fn stream_chat<Models, Prompt>(
        &self,
        provider_name: &'static str,
        model: Models,
        prompt: Prompt,
        mut history: Vec<Message>,
    ) -> Result<StreamingCompletionResponse<FinalCompletionResponse>, Error>
    where
        Models: ToString,
        Prompt: Into<Message> + WasmCompatSend,
    {
        let completion = self.completion(provider_name, model)?;

        history.push(prompt.into());
        let request = CompletionRequest {
            model: None,
            preamble: None,
            tools: vec![],
            documents: vec![],
            temperature: None,
            max_tokens: None,
            additional_params: None,
            tool_choice: None,
            chat_history: OneOrMany::many(history)
                .unwrap_or_else(|_| OneOrMany::one(Message::user(""))),
            output_schema: None,
        };

        completion.stream(request).await.map_err(Error::Completion)
    }
}
