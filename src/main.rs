mod config;
mod errors;
mod keychain;
mod llm;
mod result;

use crate::config::{get_bott_config, BottConfig};
use crate::llm::prelude::{
    check_and_get_codellama, generate, get_context, get_debug_prompt, print_answer_and_context,
};
use clap::{arg, Command};
use dialoguer::{theme::ColorfulTheme, Confirm};
use reqwest;
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
                    let cfg: BottConfig = get_bott_config().unwrap();
                    dbg!(cfg,);
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}
