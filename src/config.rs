use crate::errors::BottError;
use crate::result::BottResult;
use directories::UserDirs;
use serde_derive::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct BottConfig {
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
pub fn get_bott_config_path() -> BottResult<String> {
    let mut home = Path::new("");
    let mut _user_dirs: UserDirs;
    if let Some(user_dirs) = UserDirs::new() {
        _user_dirs = user_dirs.clone();
        home = _user_dirs.home_dir();
    }
    let bott_directory = Path::new(".bott-cli");
    return match home.join(bott_directory).into_os_string().into_string() {
        Ok(s) => Ok(s),
        Err(_) => Err(BottError::ConfigPathErr),
    };
}
pub fn get_bott_config() -> BottResult<BottConfig> {
    let bott_config_path = get_bott_config_path()?;

    if let Ok(cfg) = confy::load_path::<BottConfig>(bott_config_path) {
        return Ok(cfg);
    }
    return Err(BottError::ConfigLoadErr);
}
