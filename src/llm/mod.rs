pub mod ollama;
pub mod openai;

use crate::config::BottConfig;
use crate::llm::ollama::{
    generate as ollama_generate, print_answer_and_context as ollama_print_answer_and_context,
};
use crate::llm::openai::generate as openai_generate;
use crate::result::BottResult;
use async_openai::types::ChatCompletionRequestMessage;
use std::string::ToString;

const LLM_OLLAMA: &str = "ollama";
const LLM_OPENAI: &str = "openai";

#[derive(Debug)]
pub struct GenerateOutputOllama {
    answer: String,
    context: Vec<usize>,
}
#[derive(Debug)]
pub struct GenerateOutputOpenai {
    answer: String,
    context: Vec<ChatCompletionRequestMessage>,
}

pub enum GenerateOutput {
    Ollama(GenerateOutputOllama),
    Openai(GenerateOutputOpenai),
}
pub fn get_query_system_prompt(distro: &str, shell: &str) -> String {
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
    let output = match llm.as_str() {
        LLM_OLLAMA => GenerateOutput::Ollama(ollama_generate(query, distro, shell, debug).await?),
        LLM_OPENAI => GenerateOutput::Openai(openai_generate(query, distro, shell, debug).await?),
        _ => unimplemented!(),
    };
    return Ok(output);
}
pub fn print_answer_and_context(output: GenerateOutput) -> BottResult<()> {
    let mut config: BottConfig = BottConfig::load()?;
    let llm = config.get_key("llm")?;
    match output {
        GenerateOutput::Ollama(o) => ollama_print_answer_and_context(o),
        _ => unimplemented!(),
    }
    Ok(())
}
