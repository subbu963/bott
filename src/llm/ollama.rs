use crate::errors::{BottError, BottOllamaError};
use crate::result::BottResult;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::process::exit;

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
pub async fn get_codellama_model() -> BottResult<String> {
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
    let mut codellama_models = Vec::from_iter(
        body.models
            .iter()
            .filter(|model| model.name.starts_with("codellama:")),
    );
    codellama_models.sort_by(|a, b| b.size.cmp(&a.size));
    if codellama_models.is_empty() {
        return Err(BottError::OllamaErr(BottOllamaError::CodeLlamaUnavailable));
    }
    let first = codellama_models.get(0).unwrap().name.clone();
    Ok(first)
}
pub fn get_query_system_prompt(distro: &str, shell: &str) -> String {
    return format!(
        r#"
    You are a helpful code assistant who helps people write single line bash scripts for terminal usage.Bash code must always be enclosed between ```bash and ``` tags. 
    The bash code needs to be compatible with the users operating system and shell.
    For your information, 
    Operating system: {distro}
    Shell: {shell}
    Last executed command: 
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
pub struct GenerateOutput {
    answer: String,
    context: Vec<usize>,
}
pub async fn generate(
    query: &str,
    model: &str,
    distro: &str,
    shell: &str,
    context: Vec<usize>,
    debug: bool,
) -> BottResult<GenerateOutput> {
    let client = reqwest::Client::new();
    let mut system_prompt = String::from("");
    if (debug) {
        system_prompt = get_debug_system_prompt(distro, shell);
    } else {
        system_prompt = get_query_system_prompt(distro, shell);
    }
    let body: GenerateResponse;
    if let Ok(req) = client
        .post("http://localhost:11434/api/generate")
        .json(&GenerateRequest {
            model: String::from(model),
            prompt: String::from(query),
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
        return Ok(GenerateOutput {
            answer: String::from(body.response),
            context: body.context,
        });
    }
    let re = Regex::new(r"```bash(?P<bash_code>[\s\S]*?)```").unwrap();
    // Iterate over and collect all of the matches.
    let matches = re.captures(body.response.as_str());
    return match matches {
        Some(c) => Ok(GenerateOutput {
            answer: String::from(&c["bash_code"]).trim().to_string(),
            context: body.context,
        }),
        None => Err(BottError::OllamaErr(BottOllamaError::UnableToGetResponse)),
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
pub fn print_answer_and_context(output: GenerateOutput) {
    let context = output
        .context
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    print!(
        "<ANSWER>{answer}</ANSWER><CONTEXT>{context}</CONTEXT>",
        answer = output.answer.trim(),
        context = context
    );
}
