use crate::errors::BottError;
use crate::result::BottResult;
use directories::UserDirs;
use serde_derive::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaOptions {}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenaiOptions {}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BottConfig {
    version: String,
    llm: String,
    ollama_options: Option<OllamaOptions>,
    openai_options: Option<OpenaiOptions>,
}
impl Default for BottConfig {
    fn default() -> Self {
        Self {
            version: String::from("0.1.0"),
            llm: String::from("ollama"),
            ollama_options: None,
            openai_options: None,
        }
    }
}
impl BottConfig {
    fn get_path() -> BottResult<String> {
        let mut home = Path::new("");
        let mut _user_dirs: UserDirs;
        if let Some(user_dirs) = UserDirs::new() {
            _user_dirs = user_dirs.clone();
            home = _user_dirs.home_dir();
        }
        let bott_directory = Path::new(".bott-cli/config.yml");
        return match home.join(bott_directory).into_os_string().into_string() {
            Ok(s) => Ok(s),
            Err(_) => Err(BottError::ConfigPathErr),
        };
    }
    pub fn load() -> BottResult<BottConfig> {
        let bott_config_path = BottConfig::get_path()?;

        if let Ok(config) = confy::load_path::<BottConfig>(bott_config_path) {
            return Ok(config);
        }
        return Err(BottError::ConfigLoadErr);
    }
    pub fn save(&self) -> BottResult<()> {
        let bott_config_path = BottConfig::get_path()?;
        let _config = self.clone();
        return match confy::store_path(bott_config_path, _config) {
            Ok(_) => Ok(()),
            Err(e) => {
                print!("err is {:?}", e);
                Err(BottError::ConfigStoreErr)
            }
        };
    }
    pub fn set_key(&mut self, key: &str, value: &str) {
        match key {
            "llm" => {
                self.llm = String::from(value);
            }
            _ => unimplemented!(),
        };
    }
    pub fn get_key(&mut self, key: &str) -> String {
        return match key {
            "llm" => self.llm.clone(),
            _ => unimplemented!(),
        };
    }
    pub fn delete_key(&mut self, key: &str) -> String {
        return match key {
            _ => unimplemented!(),
        };
    }
}
