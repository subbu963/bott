mod ollama;
mod openai;

pub mod prelude {
    pub use super::ollama::*;
}

use crate::config::BottConfig;
use crate::llm::ollama::{
    generate as ollama_generate, print_answer_and_context as ollama_print_answer_and_context,
};
use crate::result::BottResult;

pub struct GenerateOutputOllama {
    answer: String,
    context: Vec<usize>,
}

pub struct GenerateOutputOpenai {
    answer: String,
    context: Vec<String>,
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
