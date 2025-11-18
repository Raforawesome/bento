use std::sync::LazyLock;

use crate::types::Username;
use axum_extra::extract::cookie::Key;
use serde::{Deserialize, Serialize};
use toml::de;
/*
 * Configuration Manager
 */
pub static LOCAL_CONF: LazyLock<Config> = LazyLock::new(|| grab_config().expect("bento.toml file"));

#[derive(Deserialize)]
pub struct Config {
    pub admin: Admin,
}

impl AsRef<Config> for Config {
    fn as_ref(&self) -> &Config {
        self
    }
}

#[derive(Deserialize)]
pub struct Admin {
    pub username: Username,
    pub password: String,
}

pub fn grab_config() -> Result<Config, de::Error> {
    let config_str = std::fs::read_to_string("bento.toml").expect("a file called ./bento.toml");
    toml::from_str(&config_str)
}

/*
 * Secrets Manager
 */
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize)]
pub struct Secrets {
    pub cookie_key: CookieKey,
}

#[derive(Clone)]
pub struct CookieKey(pub Key);

impl CookieKey {
    pub fn generate() -> Self {
        CookieKey(Key::generate())
    }
}

impl Secrets {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let secrets_path = ".bento_secrets";
        if Path::new(secrets_path).exists() {
            let secrets_str = fs::read_to_string(secrets_path)?;
            let secrets: Secrets = toml::from_str(&secrets_str).expect("valid secrets file");
            Ok(secrets)
        } else {
            // generate new secrets if secrets file doesn't exist
            let secrets = Secrets {
                cookie_key: CookieKey::generate(),
            };
            secrets.save()?;
            Ok(secrets)
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let secrets_toml = toml::to_string(self)?;
        fs::write(".bento_secrets", secrets_toml)?;
        Ok(())
    }
}

impl Default for Secrets {
    fn default() -> Self {
        Secrets {
            cookie_key: CookieKey(Key::from(&[0u8; 64])),
        }
    }
}

// serde glue for saving cookie key to disk
use base64::{Engine as _, engine::general_purpose::URL_SAFE as Base64Url};

impl Serialize for CookieKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let encoded: String = Base64Url.encode(self.0.master());
        serializer.serialize_str(&encoded)
    }
}

impl<'de> Deserialize<'de> for CookieKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let encoded = <String>::deserialize(deserializer)?;
        let bytes: Vec<u8> = Base64Url.decode(&encoded).expect("valid base64 cookie key");
        Ok(CookieKey(Key::from(&bytes)))
    }
}
