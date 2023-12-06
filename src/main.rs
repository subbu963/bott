use clap::{arg, Command};
use reqwest;
use serde::Deserialize;

fn cli() -> Command {
    Command::new("bott")
        .about("Your friendly terminal-hood chatbot")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("query")
                .about("Query")
                .arg_required_else_help(true)
                .arg(
                    arg!(<TEXT> "query text")
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
#[tokio::main]
async fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("query", sub_matches)) => {
            println!("sub_matches {:?}", sub_matches);
            let codellama_model = get_codellama_model()
                .await
                .expect("Ollama not running?")
                .expect("codellama not installed. Do `ollama pull codellama:13b-instruct`");
            let query = sub_matches.get_one::<String>("TEXT").unwrap().trim();
            println!("codellama_model is {:?} and {:?}", codellama_model, query);
        }
        _ => unreachable!(),
    }
}
