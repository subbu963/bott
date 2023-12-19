use crate::result::BottResult;
use async_openai::{
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};

pub struct GenerateOutput {
    answer: String,
    context: String,
}
async fn generate() {
    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default().model("gpt-3.5-turbo");
}
