use crate::errors::BottError;
use crate::result::BottResult;
use keyring::Entry;
use users::{get_current_uid, get_user_by_uid};

pub struct Keychain {
    user: String,
    namespace: String,
}

enum KeychainOperation {
    Get,
    Set,
    Delete,
}
impl Keychain {
    pub fn load(namespace: &str) -> Self {
        let current_user = get_user_by_uid(get_current_uid()).unwrap();
        let user = current_user.name().to_string_lossy().to_string();
        let namespace = String::from(namespace);
        Self {
            user: user,
            namespace: namespace,
        }
    }
    fn operate(
        &self,
        key: &str,
        value: Option<&str>,
        operation: KeychainOperation,
    ) -> BottResult<Option<String>> {
        let entry = match Entry::new(
            format!("bott_cli_service:{}:{}", self.namespace, key).as_str(),
            self.user.as_ref(),
        ) {
            Ok(e) => e,
            Err(_) => return Err(BottError::KeychainLoadErr),
        };
        return match operation {
            KeychainOperation::Get => {
                let password = match entry.get_password() {
                    Ok(s) => s,
                    Err(_) => return Err(BottError::KeychainGetErr),
                };
                Ok(Some(password))
            }
            KeychainOperation::Set => {
                let val = value.unwrap();
                match entry.set_password(val) {
                    Ok(_) => return Ok(None),
                    Err(_) => return Err(BottError::KeychainSetErr),
                }
            }
            KeychainOperation::Delete => match entry.delete_password() {
                Ok(_) => return Ok(None),
                Err(_) => return Err(BottError::KeychainDeleteErr),
            },
        };
    }
    pub fn get(&self, key: &str) -> BottResult<Option<String>> {
        let password = self.operate(key, None, KeychainOperation::Get)?;
        Ok(password)
    }
    pub fn set(&self, key: &str, value: &str) -> BottResult<()> {
        self.operate(key, Some(value), KeychainOperation::Set)?;
        Ok(())
    }
    pub fn delete(&self, key: &str) -> BottResult<()> {
        self.operate(key, None, KeychainOperation::Delete)?;
        Ok(())
    }
}
