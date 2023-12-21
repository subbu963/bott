use crate::config::BottConfig;
use crate::errors::{BottError, BottOllamaError};
use crate::llm::{
    get_debug_prompt, get_debug_system_prompt, get_query_system_prompt, GenerateOutputOpenai,
};
use crate::result::BottResult;
use async_openai::types::ChatCompletionRequestUserMessage;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use regex::Regex;
use std::env;

pub fn get_context(distro: &str, shell: &str, debug: bool) -> Vec<ChatCompletionRequestMessage> {
    let system_prompt: String;
    if debug {
        system_prompt = get_debug_system_prompt(distro, shell);
    } else {
        system_prompt = get_query_system_prompt(distro, shell);
    }
    let default_messages = vec![ChatCompletionRequestSystemMessageArgs::default()
        .content(system_prompt)
        .build()
        .unwrap()];
    let default_bott_context = serde_json::to_string(&default_messages).unwrap();
    let mut context_env: String;
    if (debug) {
        context_env = default_bott_context;
    } else {
        context_env = env::var("bott_context").unwrap_or(default_bott_context.clone());
        if context_env.is_empty() {
            context_env = default_bott_context
        }
    }
    let context: Vec<ChatCompletionRequestMessage> =
        serde_json::from_str(context_env.as_str()).unwrap();
    return context;
}
pub async fn get_model() -> BottResult<String> {
    let mut config = BottConfig::load()?;
    let model = config.get_key("openai:model")?.unwrap();
    Ok(model)
}
pub async fn generate(
    query: &str,
    distro: &str,
    shell: &str,
    debug: bool,
) -> BottResult<GenerateOutputOpenai> {
    let model: String = get_model().await?;
    let mut config = BottConfig::load()?;
    let api_key = config.get_key("openai:api_key")?.unwrap();
    let openai_config = OpenAIConfig::new().with_api_key(api_key);
    let mut context: Vec<ChatCompletionRequestMessage> = get_context(distro, shell, debug);
    let prompt: String;
    if (debug) {
        let input = env::var("bott_last_run_executed_code").unwrap_or(String::from(""));
        let output = env::var("bott_last_run_output").unwrap_or(String::from(""));
        prompt = get_debug_prompt(input.as_str(), output.as_str());
    } else {
        prompt = query.to_string();
    }
    context.push(ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()
            .unwrap(),
    ));
    let client = Client::with_config(openai_config);

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(context.clone())
        .build()
        .unwrap();

    let response = client.chat().create(request).await.unwrap();
    let output = response.choices.get(0).unwrap();

    let re = Regex::new(r"```bash(?P<bash_code>[\s\S]*?)```").unwrap();
    let content = output.message.content.as_ref().unwrap().clone();
    let matches = re.captures(content.as_str());
    context.push(ChatCompletionRequestMessage::Assistant(
        ChatCompletionRequestAssistantMessageArgs::default()
            .content(output.message.content.clone().unwrap_or("".to_string()))
            .tool_calls(output.message.tool_calls.clone().unwrap_or(vec![]))
            .build()
            .unwrap(),
    ));
    return match matches {
        Some(c) => Ok(GenerateOutputOpenai {
            answer: String::from(&c["bash_code"]).trim().to_string(),
            context: context,
        }),
        None => Err(BottError::OllamaErr(BottOllamaError::UnableToGetResponse)),
    };
}

pub fn print_answer_and_context(output: GenerateOutputOpenai) {
    let context = serde_json::to_string(&output.context).unwrap();
    print!(
        "<ANSWER>{answer}</ANSWER><CONTEXT>{context}</CONTEXT>",
        answer = output.answer.trim(),
        context = context
    );
}
