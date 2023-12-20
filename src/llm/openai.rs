use crate::config::BottConfig;
use crate::errors::{BottError, BottOllamaError};
use crate::llm::{GenerateOutputOllama, GenerateOutputOpenai};
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

pub fn get_system_prompt(distro: &str, shell: &str) -> String {
    return format!(
        r#"
    You are a helpful code assistant who helps people write single line bash scripts for terminal usage.Bash code must always be enclosed between ```bash and ``` tags. 
    The bash code needs to be compatible with the users operating system and shell.
    For your information, 
    Operating system: {distro}
    Shell: {shell}
    "#,
        distro = distro,
        shell = shell,
    );
}
pub fn get_debug_prompt(input: &str, output: &str) -> String {
    return format!(
        r#"
    input: {input}
    output: {output}
    "#,
        input = input,
        output = output,
    );
}
pub struct GenerateOutput {
    answer: String,
    context: String,
}
pub fn get_context(distro: &str, shell: &str) -> Vec<ChatCompletionRequestMessage> {
    let system_prompt = get_system_prompt(distro, shell);
    let default_messages = vec![ChatCompletionRequestSystemMessageArgs::default()
        .content(system_prompt)
        .build()
        .unwrap()];
    let default_bott_context = serde_json::to_string(&default_messages).unwrap();
    let mut context_env = env::var("bott_context").unwrap_or(default_bott_context.clone());
    if context_env.is_empty() {
        context_env = default_bott_context
    }
    let context: Vec<ChatCompletionRequestMessage> =
        serde_json::from_str(context_env.as_str()).unwrap();
    return context;
}

pub async fn generate(
    query: &str,
    distro: &str,
    shell: &str,
    debug: bool,
) -> BottResult<GenerateOutputOpenai> {
    let mut config = BottConfig::load()?;
    let api_key = config.get_key("openai:api_key")?.unwrap();
    let openai_config = OpenAIConfig::new().with_api_key(api_key);
    let mut context: Vec<ChatCompletionRequestMessage> = get_context(distro, shell);
    context.push(ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessageArgs::default()
            .content("figure out all the rust files that i have changed in the current directory and add them to the commit")
            .build()
            .unwrap(),
    ));
    let client = Client::with_config(openai_config);

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4")
        .messages(context.clone())
        .build()
        .unwrap();

    // println!("{}", serde_json::to_string(&request).unwrap());

    let response = client.chat().create(request).await.unwrap();
    let output = response.choices.get(0).unwrap();
    // println!("\nResponse:\n");
    // for choice in response.choices {
    //     println!(
    //         "{}: Role: {}  Content: {:?}",
    //         choice.index, choice.message.role, choice.message.content
    //     );
    // }

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
