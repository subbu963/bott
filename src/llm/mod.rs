pub mod ollama;
pub mod openai;

use crate::config::BottConfig;
use crate::llm::ollama::{
    generate as ollama_generate, print_answer_and_context as ollama_print_answer_and_context,
};
use crate::result::BottResult;
use async_openai::types::ChatCompletionRequestMessage;

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
pub async fn generate(
    query: &str,
    distro: &str,
    shell: &str,
    debug: bool,
) -> BottResult<GenerateOutput> {
    let mut config: BottConfig = BottConfig::load()?;
    let llm = config.get_key("llm")?;
    let output = ollama_generate(query, distro, shell, debug).await?;
    return Ok(GenerateOutput::Ollama(output));
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
