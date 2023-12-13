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
    pub fn new(&mut self, namespace: &str) {
        let current_user = get_user_by_uid(get_current_uid()).unwrap();
        self.user = current_user.name().to_string_lossy().to_string();
        self.namespace = String::from(namespace);
    }
    fn operate(
        &self,
        key: &str,
        value: Option<&str>,
        operation: KeychainOperation,
    ) -> Result<Option<String>, keyring::Error> {
        let entry = Entry::new(
            format!("bott_cli_service:{}:{}", self.namespace, key).as_str(),
            self.user.as_ref(),
        )?;
        return match operation {
            KeychainOperation::Get => {
                let password = entry.get_password()?;
                Ok(Some(password))
            }
            KeychainOperation::Set => {
                let val = value.unwrap();
                entry.set_password(val)?;
                Ok(None)
            }
            KeychainOperation::Delete => {
                entry.delete_password()?;
                Ok(None)
            }
        };
    }
    pub fn get(&self, key: &str) -> Result<Option<String>, keyring::Error> {
        let password = self.operate(key, None, KeychainOperation::Get)?;
        Ok(password)
    }
    pub fn set(&self, key: &str, value: &str) -> Result<(), keyring::Error> {
        self.operate(key, Some(value), KeychainOperation::Set)?;
        Ok(())
    }
    pub fn delete(&self, key: &str) -> Result<(), keyring::Error> {
        self.operate(key, None, KeychainOperation::Delete)?;
        Ok(())
    }
}
