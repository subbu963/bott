use clap::{arg, Command};
use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::process::exit;

const SYSTEM_PROMPT: &str = "You are a helpful code assistant who helps people write single line bash scripts for terminal usage. For your information, user is running {distro} operating system and {shell} shell. Bash code must always be enclosed between ```bash and ``` tags.";
fn cli() -> Command {
    Command::new("bott")
        .about("Your friendly terminal-hood chatbot")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("query")
                .about("Query")
                .arg_required_else_help(true)
                .arg(
                    arg!(distro: -d --distro <DISTRO> "distro")
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                )
                .arg(
                    arg!(shell: -s --shell <SHELL> "shell")
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                )
                .arg(
                    arg!(query: -q --query <QUERY> "query text")
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                ),
        )
}
#[derive(Deserialize, Debug)]
struct ModelMetadata {
    name: String,
    size: usize,
}
#[derive(Deserialize, Debug)]
struct ModelTags {
    models: Vec<ModelMetadata>,
}
#[derive(Serialize, Debug)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    system: String,
    context: Vec<usize>,
}
#[derive(Deserialize, Debug)]
struct GenerateResponse {
    model: String,
    response: String,
    context: Vec<usize>,
}
async fn get_codellama_model() -> Result<Option<String>, reqwest::Error> {
    let body = reqwest::get("http://localhost:11434/api/tags")
        .await?
        .json::<ModelTags>()
        .await?;
    let mut codellama_models = Vec::from_iter(
        body.models
            .iter()
            .filter(|model| model.name.starts_with("codellama:")),
    );
    codellama_models.sort_by(|a, b| b.size.cmp(&a.size));
    if codellama_models.is_empty() {
        return Ok(None);
    }
    let first = codellama_models.get(0).unwrap().name.clone();
    Ok(Some(first))
}
fn get_system_prompt(distro: &str, shell: &str) -> String {
    return format!("You are a helpful code assistant who helps people write single line bash scripts for terminal usage. For your information, user is running {distro} operating system and {shell} shell. Bash code must always be enclosed between ```bash and ``` tags.", distro=distro, shell=shell);
}
async fn generate(
    query: &str,
    model: &str,
    distro: &str,
    shell: &str,
) -> Result<Option<String>, reqwest::Error> {
    let client = reqwest::Client::new();
    let body = client
        .post("http://localhost:11434/api/generate")
        .json(&GenerateRequest {
            model: String::from(model),
            prompt: String::from(query),
            stream: false,
            system: get_system_prompt(distro, shell),
            context: vec![],
        })
        .send()
        .await?
        .json::<GenerateResponse>()
        .await?;
    let re = Regex::new(r"```bash(?P<bash_code>[\s\S]*?)```").unwrap();
    // Iterate over and collect all of the matches.
    let matches = re.captures(body.response.as_str());
    return match matches {
        Some(c) => Ok(Some(String::from(&c["bash_code"]).trim().to_string())),
        None => Ok(None),
    };
    // println!("matches {}", &matches["bash_code"]);
    // if matches.is_empty() {
    //     println!("body is {}", body.response);
    // }
    // Ok(Some(body.response))
}
#[tokio::main]
async fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("query", sub_matches)) => {
            let mut sp = Spinner::new(Spinners::Dots, "Thinking...".into());
            let codellama_model: String = match get_codellama_model().await {
                Ok(c) => match c {
                    Some(model) => model,
                    None => {
                        println!(
                            "codellama not installed. Do `ollama pull codellama:13b-instruct`"
                        );
                        exit(exitcode::CONFIG)
                    }
                },
                Err(_) => {
                    println!("Ollama not running?");
                    exit(exitcode::UNAVAILABLE)
                }
            };
            let query = sub_matches.get_one::<String>("query").unwrap().trim();
            let distro = sub_matches.get_one::<String>("distro").unwrap().trim();
            let shell = sub_matches.get_one::<String>("shell").unwrap().trim();
            match generate(query, codellama_model.as_str(), distro, shell).await {
                Ok(res) => match res {
                    Some(output) => {
                        sp.stop_with_message("".to_string());
                        print!("{}", output);
                        exit(exitcode::OK)
                    }
                    None => {
                        sp.stop_with_message("".to_string());
                        print!("Unable to get code");
                        exit(exitcode::UNAVAILABLE)
                    }
                },
                Err(e) => {
                    sp.stop_with_message("".to_string());
                    print!("error is {:?}", e);
                    exit(exitcode::UNAVAILABLE)
                }
            }
        }
        _ => unreachable!(),
    }
}
