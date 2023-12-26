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
pub enum BottOpenaiError {
    UnableToGetResponse,
}
#[derive(Debug)]
pub enum BottError {
    ConfigPath,
    ConfigLoad,
    ConfigStore,
    KeychainLoad,
    KeychainGet,
    KeychainSet,
    KeychainDelete,
    Ollama(BottOllamaError),
    Openai(BottOpenaiError),
}
impl fmt::Display for BottError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BottError::ConfigPath => write!(f, "Unable to get config path"),
            BottError::ConfigLoad => write!(f, "Unable to get config"),
            BottError::ConfigStore => write!(f, "Unable to store config"),
            BottError::KeychainLoad => write!(f, "Unable to load keychain"),
            BottError::KeychainGet => write!(f, "Unable to get key from keychain"),
            BottError::KeychainSet => write!(f, "Unable to set key in keychain"),
            BottError::KeychainDelete => write!(f, "Unable delete key from keychain"),
            // Ollama errors
            BottError::Ollama(BottOllamaError::NotRunning) => write!(f, "Ollama not running?"),
            BottError::Ollama(BottOllamaError::InvalidResponse) => {
                write!(f, "Ollama sent invalid response")
            }
            BottError::Ollama(BottOllamaError::ModelUnavailable(s)) => {
                write!(f, "model not installed. Do `ollama pull {}`", s)
            }
            BottError::Ollama(BottOllamaError::UnableToGetResponse) => {
                write!(f, "Ollama sent invalid response")
            }
            BottError::Ollama(BottOllamaError::UnknownError(s)) => {
                write!(f, "Unexpected error: {}", s)
            }
            // Openai errors
            BottError::Openai(BottOpenaiError::UnableToGetResponse) => {
                write!(f, "Openai sent invalid response")
            }
        }
    }
}
