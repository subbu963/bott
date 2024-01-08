use crate::errors::BottError;
use crate::keychain::Keychain;
use crate::result::BottResult;
use directories::UserDirs;
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaOptions {
    model: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenaiOptions {
    model: String,
}
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
            ollama_options: Some(OllamaOptions {
                model: String::from("codellama:7b-instruct"),
            }),
            openai_options: Some(OpenaiOptions {
                model: String::from("gpt-4"),
            }),
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
        let bott_dir_env = env::var("BOTT_DIR").unwrap_or(String::from(""));
        if !bott_dir_env.is_empty() {
            return match Path::new(bott_dir_env.as_str())
                .join(Path::new("config.yml"))
                .into_os_string()
                .into_string()
            {
                Ok(s) => Ok(s),
                Err(_) => Err(BottError::ConfigPathErr),
            };
        }
        let bott_directory = Path::new(".bott/config.yml");
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
    pub fn set_key(&mut self, key: &str, value: &str) -> BottResult<()> {
        match key {
            "llm" => {
                self.llm = String::from(value);
                self.save()?;
            }
            "ollama:model" => {
                self.ollama_options = Some(OllamaOptions {
                    model: value.to_string(),
                });
                self.save()?;
            }
            "openai:model" => {
                self.openai_options = Some(OpenaiOptions {
                    model: value.to_string(),
                });
                self.save()?;
            }
            "openai:api_key" => {
                let (namespace, key) = key.split_once(":").unwrap();
                let keychain = Keychain::load(namespace);
                keychain.set(key, value)?;
            }
            _ => unimplemented!(),
        };
        Ok(())
    }
    pub fn get_key(&mut self, key: &str) -> BottResult<Option<String>> {
        return match key {
            "llm" => Ok(Some(self.llm.clone())),
            "ollama:model" => {
                if let Some(options) = self.ollama_options.clone() {
                    return Ok(Some(options.model));
                }
                Ok(None)
            }
            "openai:model" => {
                if let Some(options) = self.openai_options.clone() {
                    return Ok(Some(options.model));
                }
                Ok(None)
            }
            "openai:api_key" => {
                let (namespace, key) = key.split_once(":").unwrap();
                let keychain = Keychain::load(namespace);
                Ok(keychain.get(key)?)
            }
            _ => unimplemented!(),
        };
    }
    pub fn delete_key(&mut self, key: &str) -> BottResult<()> {
        return match key {
            "ollama:model" => {
                self.ollama_options = None;
                self.save()?;
                Ok(())
            }
            "openai:model" => {
                self.openai_options = None;
                self.save()?;
                Ok(())
            }
            "openai:api_key" => {
                let (namespace, key) = key.split_once(":").unwrap();
                let keychain = Keychain::load(namespace);
                Ok(keychain.delete(key)?)
            }
            _ => unimplemented!(),
        };
    }
}
