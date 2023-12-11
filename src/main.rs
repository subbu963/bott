use clap::{arg, Command};
use dialoguer::{theme::ColorfulTheme, Confirm};
use keyring::Entry;
use regex::Regex;
use reqwest;
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::env;
use std::process::exit;
use users::{get_current_uid, get_user_by_uid};

#[derive(Debug, Serialize, Deserialize)]
struct BottConfig {
    version: String,
    llm: String,
}
impl Default for BottConfig {
    fn default() -> Self {
        Self {
            version: String::from("0.1.0"),
            llm: String::from("ollama"),
        }
    }
}
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
        .subcommand(
            Command::new("debug")
                .about("Debug")
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
                ),
        )
        .subcommand(
            Command::new("config").about("Config").subcommand(
                Command::new("set")
                    .about("Set")
                    .arg_required_else_help(true)
                    .arg(
                        arg!(key: -k --key <KEY> "key")
                            .required(true)
                            .value_parser(clap::value_parser!(String)),
                    )
                    .arg(
                        arg!(value: -v --value <VALUE> "value")
                            .required(true)
                            .value_parser(clap::value_parser!(String)),
                    ),
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
fn get_query_system_prompt(distro: &str, shell: &str) -> String {
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

fn get_debug_system_prompt(distro: &str, shell: &str) -> String {
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
fn get_debug_prompt(input: &str, output: &str) -> String {
    return format!(
        r#"
    input: {input}
    output: {output}
    "#,
        input = input,
        output = output,
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
    debug: bool,
) -> Result<Option<GenerateOutput>, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut system_prompt = String::from("");
    if (debug) {
        system_prompt = get_debug_system_prompt(distro, shell);
    } else {
        system_prompt = get_query_system_prompt(distro, shell);
    }
    let body = client
        .post("http://localhost:11434/api/generate")
        .json(&GenerateRequest {
            model: String::from(model),
            prompt: String::from(query),
            stream: false,
            system: system_prompt,
            context: context,
        })
        .send()
        .await?
        .json::<GenerateResponse>()
        .await?;
    if debug {
        return Ok(Some(GenerateOutput {
            answer: String::from(body.response),
            context: body.context,
        }));
    }
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
async fn check_and_get_codellama() -> String {
    return match get_codellama_model().await {
        Ok(c) => match c {
            Some(model) => model,
            None => {
                println!("codellama not installed. Do `ollama pull codellama:13b-instruct`");
                exit(exitcode::CONFIG)
            }
        },
        Err(_) => {
            println!("Ollama not running?");
            exit(exitcode::UNAVAILABLE)
        }
    };
}
fn get_context() -> Vec<usize> {
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
fn print_answer_and_context(output: GenerateOutput) {
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
struct Keyring {
    user: String,
    namespace: String,
}

enum KeyringOperation {
    get,
    set,
    delete,
}
impl Keyring {
    pub fn new(&mut self, namespace: &str) {
        let current_user = get_user_by_uid(get_current_uid()).unwrap();
        self.user = current_user.name().to_string_lossy().to_string();
        self.namespace = String::from(namespace);
    }
    fn operate(
        &self,
        key: &str,
        value: Option<&str>,
        operation: KeyringOperation,
    ) -> Result<Option<String>, keyring::Error> {
        let entry = Entry::new(
            format!("bott_cli_service:{}:{}", self.namespace, key).as_str(),
            self.user.as_ref(),
        )?;
        return match operation {
            KeyringOperation::get => {
                let password = entry.get_password()?;
                Ok(Some(password))
            }
            KeyringOperation::set => {
                let val = value.unwrap();
                entry.set_password(val)?;
                Ok(None)
            }
            KeyringOperation::delete => {
                entry.delete_password()?;
                Ok(None)
            }
        };
    }
    pub fn get(&self, key: &str) -> Result<Option<String>, keyring::Error> {
        let password = self.operate(key, None, KeyringOperation::get)?;
        Ok(password)
    }
    pub fn set(&self, key: &str, value: &str) -> Result<(), keyring::Error> {
        self.operate(key, Some(value), KeyringOperation::set)?;
        Ok(())
    }
    pub fn delete(&self, key: &str) -> Result<(), keyring::Error> {
        self.operate(key, None, KeyringOperation::delete)?;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("query", sub_matches)) => {
            let mut sp = Spinner::new(Spinners::Dots, "Thinking...".into());
            let codellama_model: String = check_and_get_codellama().await;
            let query = sub_matches.get_one::<String>("query").unwrap().trim();
            let distro = sub_matches.get_one::<String>("distro").unwrap().trim();
            let shell = sub_matches.get_one::<String>("shell").unwrap().trim();
            let context = get_context();

            match generate(
                query,
                codellama_model.as_str(),
                distro,
                shell,
                context,
                false,
            )
            .await
            {
                Ok(res) => match res {
                    Some(output) => {
                        sp.stop_with_message("".to_string());
                        print_answer_and_context(output);
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
        Some(("debug", sub_matches)) => {
            let codellama_model: String = check_and_get_codellama().await;
            let mut sp = Spinner::new(Spinners::Dots, "Thinking...".into());
            let input = env::var("bott_last_executed_code").unwrap_or(String::from(""));
            let output = env::var("bott_last_output").unwrap_or(String::from(""));
            let prompt = get_debug_prompt(input.as_str(), output.as_str());
            let distro = sub_matches.get_one::<String>("distro").unwrap().trim();
            let shell = sub_matches.get_one::<String>("shell").unwrap().trim();
            let context = vec![];
            match generate(
                prompt.as_str(),
                codellama_model.as_str(),
                distro,
                shell,
                context,
                true,
            )
            .await
            {
                Ok(res) => match res {
                    Some(output) => {
                        sp.stop_with_message("".to_string());
                        print_answer_and_context(output);
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
        Some(("config", sub_matches)) => {
            let config_command = sub_matches.subcommand().unwrap_or(("get", sub_matches));
            match config_command {
                ("set", sub_matches) => {
                    let key = sub_matches.get_one::<String>("key").unwrap().trim();
                    let value = sub_matches.get_one::<String>("value").unwrap().trim();

                    print!("key {:?}, value {:?}", key, value);
                    let cfg: BottConfig = confy::load("bott-cli-config", None).unwrap();
                    dbg!(
                        cfg,
                        confy::get_configuration_file_path("bott-cli-config", None)
                    );
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}
