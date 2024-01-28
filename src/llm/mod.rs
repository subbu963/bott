pub mod ollama;
pub mod openai;

use crate::config::BottConfig;
use crate::llm::ollama::{
    generate as ollama_generate, print_answer_and_context as ollama_print_answer_and_context,
};
use crate::llm::openai::{
    generate as openai_generate, print_answer_and_context as openai_print_answer_and_context,
};
use crate::result::BottResult;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageContent};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use std::string::ToString;

const LLM_OLLAMA: &str = "ollama";
const LLM_OPENAI: &str = "openai";

#[derive(Debug, Clone)]
pub struct GenerateOutputOllama {
    raw_answer: String,
    shell_command: String,
    context: Vec<usize>,
}
fn content_from_user_message(content: ChatCompletionRequestUserMessageContent) -> String {
    return match content {
        ChatCompletionRequestUserMessageContent::Text(t) => t,
        _ => unimplemented!("Array(Vec<ChatCompletionRequestMessageContentPart>) not implemented!"),
    };
}
fn _encode_content(content: String) -> String {
    general_purpose::STANDARD.encode(content)
}
fn decode_content(
    content: ChatCompletionRequestUserMessageContent,
) -> ChatCompletionRequestUserMessageContent {
    return match content {
        ChatCompletionRequestUserMessageContent::Text(t) => {
            ChatCompletionRequestUserMessageContent::Text(
                String::from_utf8(general_purpose::STANDARD.decode(t.as_bytes()).unwrap()).unwrap(),
            )
        }
        _ => unimplemented!("Array(Vec<ChatCompletionRequestMessageContentPart>) not implemented!"),
    };
}
fn _decode_content(content: String) -> String {
    String::from_utf8(
        general_purpose::STANDARD
            .decode(content.as_bytes())
            .unwrap(),
    )
    .unwrap()
}
#[derive(Debug, Clone)]
pub struct GenerateOutputOpenai {
    raw_answer: String,
    shell_command: String,
    context: Vec<ChatCompletionRequestMessage>,
}
impl GenerateOutputOpenai {
    pub fn encode_context(
        context: &Vec<ChatCompletionRequestMessage>,
    ) -> Vec<ChatCompletionRequestMessage> {
        return context
            .iter()
            .map(|m| match m.clone() {
                ChatCompletionRequestMessage::User(mut _m) => {
                    _m.content = ChatCompletionRequestUserMessageContent::Text(_encode_content(
                        content_from_user_message(_m.content),
                    ));
                    ChatCompletionRequestMessage::User(_m)
                }
                ChatCompletionRequestMessage::System(mut _m) => {
                    _m.content = _encode_content(_m.content);
                    ChatCompletionRequestMessage::System(_m)
                }
                ChatCompletionRequestMessage::Assistant(mut _m) => {
                    if _m.content.is_some() {
                        _m.content = Some(_encode_content(_m.content.unwrap()));
                    }
                    ChatCompletionRequestMessage::Assistant(_m)
                }
                ChatCompletionRequestMessage::Tool(mut _m) => {
                    _m.content = _encode_content(_m.content);
                    ChatCompletionRequestMessage::Tool(_m)
                }
                ChatCompletionRequestMessage::Function(mut _m) => {
                    if _m.content.is_some() {
                        _m.content = Some(_encode_content(_m.content.unwrap()));
                    }
                    ChatCompletionRequestMessage::Function(_m)
                }
            })
            .collect::<Vec<ChatCompletionRequestMessage>>();
    }
    pub fn decode_context(
        context: &Vec<ChatCompletionRequestMessage>,
    ) -> Vec<ChatCompletionRequestMessage> {
        return context
            .iter()
            .map(|m| match m.clone() {
                ChatCompletionRequestMessage::User(mut _m) => {
                    let content = match _m.content {
                        ChatCompletionRequestUserMessageContent::Text(t) => t,
                        _ => unimplemented!(
                            "Array(Vec<ChatCompletionRequestMessageContentPart>) not implemented!"
                        ),
                    };
                    _m.content =
                        ChatCompletionRequestUserMessageContent::Text(_decode_content(content));
                    ChatCompletionRequestMessage::User(_m)
                }
                ChatCompletionRequestMessage::System(mut _m) => {
                    _m.content = _decode_content(_m.content);
                    ChatCompletionRequestMessage::System(_m)
                }
                ChatCompletionRequestMessage::Assistant(mut _m) => {
                    if _m.content.is_some() {
                        _m.content = Some(_decode_content(_m.content.unwrap()));
                    }
                    ChatCompletionRequestMessage::Assistant(_m)
                }
                ChatCompletionRequestMessage::Tool(mut _m) => {
                    _m.content = _decode_content(_m.content);
                    ChatCompletionRequestMessage::Tool(_m)
                }
                ChatCompletionRequestMessage::Function(mut _m) => {
                    if _m.content.is_some() {
                        _m.content = Some(_decode_content(_m.content.unwrap()));
                    }
                    ChatCompletionRequestMessage::Function(_m)
                }
            })
            .collect::<Vec<ChatCompletionRequestMessage>>();
    }
}

pub enum GenerateOutput {
    Ollama(GenerateOutputOllama),
    Openai(GenerateOutputOpenai),
}
impl GenerateOutput {
    pub async fn get_output(
        llm: &str,
        query: &str,
        distro: &str,
        shell: &str,
        debug: bool,
    ) -> BottResult<GenerateOutput> {
        let output = match llm {
            LLM_OLLAMA => {
                let _output = ollama_generate(query, distro, shell, debug).await?;
                GenerateOutput::Ollama(_output)
            }
            LLM_OPENAI => {
                let _output = openai_generate(query, distro, shell, debug).await?;
                GenerateOutput::Openai(_output)
            }
            _ => unimplemented!(),
        };
        Ok(output)
    }
}
pub fn get_query_system_prompt(distro: &str, shell: &str) -> String {
    return format!(
        r#"
    You are a helpful assistant who helps people with:
    1) Writing single line shell scripts for terminal usage. Bash/Shell scripts must always be enclosed between ```bash and ``` tags. Return a single bash script only please. Also, give a very brief explanation of what the script does. The bash code needs to be compatible with the users operating system and shell.
    2) Any other miscellaneous tasks.
    For your information, 
    Operating system: {distro}
    Shell: {shell}
    "#,
        distro = distro,
        shell = shell,
    );
}

pub fn get_debug_system_prompt(distro: &str, shell: &str) -> String {
    return format!(
        r#"
    You are a helpful code assistant who helps people write single line bash scripts for terminal usage. Given an input command and the corresponding output, tell the user why the command is failing. Write your answer in a single line with newlines using `\n` and double quoutes escaped
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
pub async fn generate(
    query: &str,
    distro: &str,
    shell: &str,
    debug: bool,
) -> BottResult<GenerateOutput> {
    let mut config: BottConfig = BottConfig::load()?;
    let llm = config.get_key("llm")?.unwrap_or("".to_string());
    let output = GenerateOutput::get_output(llm.as_str(), query, distro, shell, debug).await?;
    return Ok(output);
}
pub fn print_answer_and_context(output: GenerateOutput) -> BottResult<()> {
    match output {
        GenerateOutput::Ollama(o) => ollama_print_answer_and_context(o),
        GenerateOutput::Openai(o) => openai_print_answer_and_context(o),
        _ => unimplemented!(),
    }
    Ok(())
}
