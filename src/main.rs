mod config;
mod errors;
mod keychain;
mod llm;
mod result;

use crate::config::BottConfig;
use crate::llm::{generate, print_answer_and_context};
use clap::{arg, Command};
use dialoguer::{theme::ColorfulTheme, Confirm};
use spinners::{Spinner, Spinners};
use std::process::exit;

fn cli() -> Command {
    Command::new("bott")
        .about("Your friendly terminal-hood chatbot")
        .arg_required_else_help(true)
        .arg(arg!(version: -v --version))
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
            Command::new("config")
                .about("Config")
                .subcommand(
                    Command::new("get")
                        .about("Get")
                        .arg_required_else_help(true)
                        .arg(
                            arg!(key: -k --key <KEY> "key")
                                .required(true)
                                .value_parser(clap::value_parser!(String)),
                        ),
                )
                .subcommand(
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
                )
                .subcommand(
                    Command::new("delete")
                        .about("deletee")
                        .arg_required_else_help(true)
                        .arg(
                            arg!(key: -k --key <KEY> "key")
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
            let query = sub_matches.get_one::<String>("query").unwrap().trim();
            let distro = sub_matches.get_one::<String>("distro").unwrap().trim();
            let shell = sub_matches.get_one::<String>("shell").unwrap().trim();

            match generate(query, distro, shell, false).await {
                Ok(output) => {
                    sp.stop_with_message("".to_string());
                    print_answer_and_context(output).unwrap();
                    exit(exitcode::OK)
                }
                Err(e) => {
                    sp.stop_with_message("".to_string());
                    print!("{}", e);
                    exit(exitcode::UNAVAILABLE);
                }
            }
        }
        Some(("debug", sub_matches)) => {
            let mut sp = Spinner::new(Spinners::Dots, "Thinking...".into());
            let distro = sub_matches.get_one::<String>("distro").unwrap().trim();
            let shell = sub_matches.get_one::<String>("shell").unwrap().trim();
            match generate("", distro, shell, true).await {
                Ok(output) => {
                    sp.stop_with_message("".to_string());
                    print_answer_and_context(output).unwrap();
                    exit(exitcode::OK)
                }
                Err(e) => {
                    sp.stop_with_message("".to_string());
                    print!("{}", e);
                    exit(exitcode::UNAVAILABLE);
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

                    let mut config: BottConfig = match BottConfig::load() {
                        Ok(c) => c,
                        Err(e) => {
                            print!("{}", e);
                            exit(exitcode::UNAVAILABLE);
                        }
                    };

                    if let Err(e) = config.set_key(key, value) {
                        print!("{}", e);
                        exit(exitcode::UNAVAILABLE);
                    }
                }
                ("get", sub_matches) => {
                    let key = sub_matches.get_one::<String>("key").unwrap().trim();

                    let mut config: BottConfig = match BottConfig::load() {
                        Ok(c) => c,
                        Err(e) => {
                            print!("{}", e);
                            exit(exitcode::UNAVAILABLE);
                        }
                    };
                    match config.get_key(key) {
                        Ok(k) => {
                            match k {
                                Some(_k) => {
                                    print!("{}", _k);
                                    exit(exitcode::OK);
                                }
                                None => {
                                    print!("key {} not found", key);
                                    exit(exitcode::UNAVAILABLE);
                                }
                            };
                        }
                        Err(e) => {
                            print!("{}", e);
                            exit(exitcode::UNAVAILABLE);
                        }
                    }
                }
                ("delete", sub_matches) => {
                    let key = sub_matches.get_one::<String>("key").unwrap().trim();

                    let mut config: BottConfig = match BottConfig::load() {
                        Ok(c) => c,
                        Err(e) => {
                            print!("{}", e);
                            exit(exitcode::UNAVAILABLE);
                        }
                    };
                    if let Err(e) = config.delete_key(key) {
                        print!("{}", e);
                        exit(exitcode::UNAVAILABLE);
                    }
                    exit(exitcode::OK);
                }
                _ => unreachable!(),
            }
        }
        _ => {
            if matches.args_present() && matches.get_flag("version") {
                print!("{}", env!("CARGO_PKG_VERSION"));
                return;
            }
            unreachable!();
        }
    }
}
