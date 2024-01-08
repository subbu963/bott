use crate::config::BottConfig;
use crate::errors::{BottError, BottOllamaError};
use crate::llm::{
    get_debug_prompt, get_debug_system_prompt, get_query_system_prompt, GenerateOutputOllama,
};
use crate::result::BottResult;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Debug)]
pub struct ModelMetadata {
    name: String,
    size: usize,
}
#[derive(Deserialize, Debug)]
pub struct ModelTags {
    models: Vec<ModelMetadata>,
}
#[derive(Serialize, Debug)]
pub struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    system: String,
    context: Vec<usize>,
}
#[derive(Deserialize, Debug)]
pub struct GenerateResponse {
    model: String,
    response: String,
    context: Vec<usize>,
}
pub async fn get_model() -> BottResult<String> {
    let body: ModelTags;
    if let Ok(req) = reqwest::get("http://localhost:11434/api/tags").await {
        if let Ok(_body) = req.json::<ModelTags>().await {
            body = _body;
        } else {
            return Err(BottError::OllamaErr(BottOllamaError::InvalidResponse));
        }
    } else {
        return Err(BottError::OllamaErr(BottOllamaError::NotRunning));
    }
    let mut config: BottConfig = BottConfig::load()?;
    let chosen_model = config.get_key("ollama:model")?.unwrap();
    if !body.models.iter().any(|m| m.name == chosen_model) {
        return Err(BottError::OllamaErr(BottOllamaError::ModelUnavailable(
            chosen_model,
        )));
    }
    Ok(chosen_model)
}

pub async fn generate(
    query: &str,
    distro: &str,
    shell: &str,
    debug: bool,
) -> BottResult<GenerateOutputOllama> {
    let model: String = get_model().await?;
    let context: Vec<usize>;
    let prompt: String;
    let client = reqwest::Client::new();
    let mut system_prompt = String::from("");
    if (debug) {
        let input = env::var("bott_last_run_executed_code").unwrap_or(String::from(""));
        let output = env::var("bott_last_run_output").unwrap_or(String::from(""));
        prompt = get_debug_prompt(input.as_str(), output.as_str());
        context = vec![];
        system_prompt = get_debug_system_prompt(distro, shell);
    } else {
        prompt = String::from(query);
        context = get_context();
        system_prompt = get_query_system_prompt(distro, shell);
    }
    let body: GenerateResponse;
    if let Ok(req) = client
        .post("http://localhost:11434/api/generate")
        .json(&GenerateRequest {
            model: model,
            prompt: prompt,
            stream: false,
            system: system_prompt,
            context: context,
        })
        .send()
        .await
    {
        if let Ok(_body) = req.json::<GenerateResponse>().await {
            body = _body;
        } else {
            return Err(BottError::OllamaErr(BottOllamaError::InvalidResponse));
        }
    } else {
        return Err(BottError::OllamaErr(BottOllamaError::NotRunning));
    }
    if (debug) {
        return Ok(GenerateOutputOllama {
            raw_answer: body.response.to_string(),
            shell_command: String::from(body.response),
            context: body.context,
        });
    }
    let re = Regex::new(r"```bash(?P<bash_code>[\s\S]*?)```").unwrap();
    let matches = re.captures(body.response.as_str());
    return match matches {
        Some(c) => Ok(GenerateOutputOllama {
            raw_answer: body.response.to_string(),
            shell_command: String::from(&c["bash_code"]).trim().to_string(),
            context: body.context,
        }),
        None => Ok(GenerateOutputOllama {
            raw_answer: body.response.to_string(),
            shell_command: String::from(""),
            context: body.context,
        }),
    };
}
pub fn get_context() -> Vec<usize> {
    let context_env = env::var("bott_context").unwrap_or(String::from(""));
    let context_strings = context_env.split(" ").collect::<Vec<&str>>();
    let mut context: Vec<usize> = vec![];
    if !context_strings.get(0).unwrap().is_empty() {
        context = context_strings
            .iter()
            .map(|x| x.parse::<usize>().unwrap())
            .collect::<Vec<usize>>();
    }
    return context;
}
pub fn print_answer_and_context(output: GenerateOutputOllama) {
    let context = output
        .context
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    print!(
        "<RAW-ANSWER>{raw_answer}</RAW-ANSWER><SHELL-COMMAND>{shell_command}</SHELL-COMMAND><CONTEXT>{context}</CONTEXT>",
        raw_answer = output.raw_answer,
        shell_command = output.shell_command.trim(),
        context = context
    );
}
