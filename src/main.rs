use clap::{arg, Command};
use dialoguer::{theme::ColorfulTheme, Confirm};
use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::env;
use std::process::exit;

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
        .subcommand(
            Command::new("confirm").about("Confirm").arg(
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
struct GenerateOutput {
    answer: String,
    context: Vec<usize>,
}
async fn generate(
    query: &str,
    model: &str,
    distro: &str,
    shell: &str,
    context: Vec<usize>,
) -> Result<Option<GenerateOutput>, reqwest::Error> {
    let client = reqwest::Client::new();
    let body = client
        .post("http://localhost:11434/api/generate")
        .json(&GenerateRequest {
            model: String::from(model),
            prompt: String::from(query),
            stream: false,
            system: get_system_prompt(distro, shell),
            context: context,
        })
        .send()
        .await?
        .json::<GenerateResponse>()
        .await?;
    let re = Regex::new(r"```bash(?P<bash_code>[\s\S]*?)```").unwrap();
    // Iterate over and collect all of the matches.
    let matches = re.captures(body.response.as_str());
    return match matches {
        Some(c) => Ok(Some(GenerateOutput {
            answer: String::from(&c["bash_code"]).trim().to_string(),
            context: body.context,
        })),
        None => Ok(None),
    };
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
            let context_env = env::var("bott_context").unwrap_or(String::from(""));
            let context_strings = context_env.split(" ").collect::<Vec<&str>>();
            let mut context: Vec<usize> = vec![];
            if !context_strings.get(0).unwrap().is_empty() {
                context = context_strings
                    .iter()
                    .map(|x| x.parse::<usize>().unwrap())
                    .collect::<Vec<usize>>();
            }

            match generate(query, codellama_model.as_str(), distro, shell, context).await {
                Ok(res) => match res {
                    Some(output) => {
                        sp.stop_with_message("".to_string());
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
        Some(("confirm", sub_matches)) => {
            let query = sub_matches.get_one::<String>("query").unwrap().trim();
            match Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(query)
                .default(true)
                .wait_for_newline(true)
                .interact_opt()
                .unwrap()
            {
                Some(true) => exit(exitcode::OK),
                Some(false) => exit(exitcode::UNAVAILABLE),
                None => println!("Ok, we can start over later"),
            }
            exit(exitcode::UNAVAILABLE)
        }
        _ => unreachable!(),
    }
}
