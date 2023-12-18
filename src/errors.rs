use std::fmt;

#[derive(Debug)]
pub enum BottOllamaError {
    NotRunning,
    InvalidResponse,
    ModelUnavailable(String),
    UnableToGetResponse,
    UnknownError(String),
}
#[derive(Debug)]
pub enum BottError {
    ConfigPathErr,
    ConfigLoadErr,
    ConfigStoreErr,
    KeychainLoadErr,
    KeychainGetErr,
    KeychainSetErr,
    KeychainDeleteErr,
    OllamaErr(BottOllamaError),
}
impl fmt::Display for BottError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BottError::ConfigPathErr => write!(f, "Unable to get config path"),
            BottError::ConfigLoadErr => write!(f, "Unable to get config"),
            BottError::ConfigStoreErr => write!(f, "Unable to store config"),
            BottError::KeychainLoadErr => write!(f, "Unable to load keychain"),
            BottError::KeychainGetErr => write!(f, "Unable to get key from keychain"),
            BottError::KeychainSetErr => write!(f, "Unable to set key in keychain"),
            BottError::KeychainDeleteErr => write!(f, "Unable delete key from keychain"),
            BottError::OllamaErr(BottOllamaError::NotRunning) => write!(f, "Ollama not running?"),
            BottError::OllamaErr(BottOllamaError::InvalidResponse) => {
                write!(f, "Ollama sent invalid response")
            }
            BottError::OllamaErr(BottOllamaError::ModelUnavailable(s)) => {
                write!(f, "model not installed. Do `ollama pull {}`", s)
            }
            BottError::OllamaErr(BottOllamaError::UnableToGetResponse) => {
                write!(f, "Ollama sent invalid response")
            }
            BottError::OllamaErr(BottOllamaError::UnknownError(s)) => {
                write!(f, "Unexpected error: {}", s)
            }
        }
    }
}
