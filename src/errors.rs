#[derive(Debug)]
pub enum BottOllamaError {
    NotRunning,
    InvalidResponse,
    CodeLlamaUnavailable,
    UnableToGetResponse,
}
#[derive(Debug)]
pub enum BottError {
    ConfigPathErr,
    ConfigLoadErr,
    ConfigStoreErr,
    KeychainOperateErr,
    KeychainGetErr,
    KeychainSetErr,
    KeychainDeleteErr,
    OllamaErr(BottOllamaError),
}
