use futures::StreamExt as _;
use rig_core::{
    agent::AgentBuilder,
    client::CompletionClient,
    completion::{
        CompletionError, CompletionModel, CompletionRequest, CompletionRequestBuilder,
        CompletionResponse, GetTokenUsage, Usage,
    },
    message::{Message, ToolCall},
    streaming::{
        RawStreamingChoice, RawStreamingToolCall, StreamedAssistantContent,
        StreamingCompletionResponse,
    },
    wasm_compat::{WasmBoxedFuture, WasmCompatSend, WasmCompatSync},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// The final streaming response from a dynamic client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalCompletionResponse {
    pub usage: Option<Usage>,
}

impl GetTokenUsage for FinalCompletionResponse {
    fn token_usage(&self) -> Usage {
        self.usage.unwrap_or_default()
    }
}

/// Wraps a CompletionModel in a dyn-compatible way for AgentBuilder.
#[derive(Clone)]
pub struct CompletionModelHandle<'a>(Arc<dyn CompletionModelDyn + 'a>);

impl<'a> CompletionModelHandle<'a> {
    pub fn new(handle: Arc<dyn CompletionModelDyn + 'a>) -> Self {
        Self(handle)
    }
}

impl CompletionModel for CompletionModelHandle<'_> {
    type Response = ();
    type StreamingResponse = FinalCompletionResponse;
    type Client = ();

    fn make(_: &Self::Client, _: impl Into<String>) -> Self {
        panic!("Cannot create a completion model handle from a client")
    }

    fn completion(
        &self,
        request: CompletionRequest,
    ) -> impl Future<Output = Result<CompletionResponse<Self::Response>, CompletionError>> + WasmCompatSend
    {
        self.0.completion(request)
    }

    fn stream(
        &self,
        request: CompletionRequest,
    ) -> impl Future<
        Output = Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>,
    > + WasmCompatSend {
        self.0.stream(request)
    }
}

pub trait CompletionClientDyn {
    /// Create a completion model with the given name.
    fn completion_model<'a>(&self, model: &str) -> Box<dyn CompletionModelDyn + 'a>;

    /// Create an agent builder with the given completion model.
    fn agent<'a>(&self, model: &str) -> AgentBuilder<CompletionModelHandle<'a>>;
}

impl<T, M, R> CompletionClientDyn for T
where
    T: CompletionClient<CompletionModel = M>,
    M: CompletionModel<StreamingResponse = R> + 'static,
    R: Clone + Unpin + GetTokenUsage + WasmCompatSend + 'static,
{
    fn completion_model<'a>(&self, model: &str) -> Box<dyn CompletionModelDyn + 'a> {
        Box::new(self.completion_model(model))
    }

    fn agent<'a>(&self, model: &str) -> AgentBuilder<CompletionModelHandle<'a>> {
        AgentBuilder::new(CompletionModelHandle(Arc::new(
            self.completion_model(model),
        )))
    }
}

pub trait CompletionModelDyn: WasmCompatSend + WasmCompatSync {
    fn completion(
        &self,
        request: CompletionRequest,
    ) -> WasmBoxedFuture<'_, Result<CompletionResponse<()>, CompletionError>>;

    fn stream(
        &self,
        request: CompletionRequest,
    ) -> WasmBoxedFuture<
        '_,
        Result<StreamingCompletionResponse<FinalCompletionResponse>, CompletionError>,
    >;

    fn completion_request(
        &self,
        prompt: Message,
    ) -> CompletionRequestBuilder<CompletionModelHandle<'_>>;
}

impl<T, R> CompletionModelDyn for T
where
    T: CompletionModel<StreamingResponse = R>,
    R: Clone + Unpin + GetTokenUsage + WasmCompatSend + 'static,
{
    fn completion(
        &self,
        request: CompletionRequest,
    ) -> WasmBoxedFuture<'_, Result<CompletionResponse<()>, CompletionError>> {
        Box::pin(async move {
            self.completion(request)
                .await
                .map(|resp| CompletionResponse {
                    choice: resp.choice,
                    usage: resp.usage,
                    raw_response: (),
                    message_id: resp.message_id,
                })
        })
    }

    fn stream(
        &self,
        request: CompletionRequest,
    ) -> WasmBoxedFuture<
        '_,
        Result<StreamingCompletionResponse<FinalCompletionResponse>, CompletionError>,
    > {
        Box::pin(async move {
            let stream = self.stream(request).await?.map(assistant_content_to_raw);

            Ok(StreamingCompletionResponse::stream(Box::pin(stream)))
        })
    }

    /// Generates a completion request builder for the given `prompt`.
    fn completion_request(
        &self,
        prompt: Message,
    ) -> CompletionRequestBuilder<CompletionModelHandle<'_>> {
        CompletionRequestBuilder::new(CompletionModelHandle::new(Arc::new(self.clone())), prompt)
    }
}

/// Converts processed streaming content back into raw variants to fake a raw stream
fn assistant_content_to_raw<R>(
    chunk: Result<StreamedAssistantContent<R>, CompletionError>,
) -> Result<RawStreamingChoice<FinalCompletionResponse>, CompletionError>
where
    R: Clone + Unpin + GetTokenUsage,
{
    let item = match chunk? {
        StreamedAssistantContent::Final(f) => {
            RawStreamingChoice::FinalResponse(FinalCompletionResponse {
                usage: Some(f.token_usage()),
            })
        }
        StreamedAssistantContent::Text(text) => RawStreamingChoice::Message(text.text),
        StreamedAssistantContent::ToolCall {
            tool_call,
            internal_call_id,
        } => RawStreamingChoice::ToolCall(tool_call_to_raw(internal_call_id, tool_call)),
        StreamedAssistantContent::ToolCallDelta {
            id,
            internal_call_id,
            content,
        } => RawStreamingChoice::ToolCallDelta {
            id,
            internal_call_id,
            content,
        },
        StreamedAssistantContent::Reasoning(reasoning) => RawStreamingChoice::Reasoning {
            id: reasoning.id.clone(),
            content: rig_core::message::ReasoningContent::Text {
                text: reasoning.display_text(),
                signature: None,
            },
        },
        StreamedAssistantContent::ReasoningDelta { id, reasoning } => {
            RawStreamingChoice::ReasoningDelta { id, reasoning }
        }
    };

    Ok(item)
}

fn tool_call_to_raw(internal_call_id: String, tool_call: ToolCall) -> RawStreamingToolCall {
    RawStreamingToolCall {
        id: tool_call.id,
        internal_call_id,
        call_id: tool_call.call_id,
        name: tool_call.function.name,
        arguments: tool_call.function.arguments,
        signature: tool_call.signature,
        additional_params: tool_call.additional_params,
    }
}
