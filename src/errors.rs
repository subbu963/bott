#[derive(Debug)]
pub enum BottError {
    ConfigPathErr,
    ConfigLoadErr,
    KeychainOperateErr,
    KeychainGetErr,
    KeychainSetErr,
    KeychainDeleteErr,
}
