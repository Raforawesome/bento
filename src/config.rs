use std::sync::LazyLock;

use axum_extra::extract::cookie;
use serde::{Deserialize, Serialize};
use toml::de;

pub static LOCAL_CONF: LazyLock<Config> =
    LazyLock::new(|| grab_config().expect("Failed to load configuration from bento.toml"));

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
    pub username: String,
    pub password: String,
}

pub fn grab_config() -> Result<Config, de::Error> {
    let config_str = std::fs::read_to_string("bento.toml").expect("Failed to read bento.toml");
    toml::from_str(&config_str)
}

/*
 * Secrets Manager
 */
use std::fs;
use std::path::Path;

pub struct CookieKey(cookie::Key);

impl CookieKey {
    pub fn generate() -> Self {
        CookieKey(cookie::Key::generate())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Secrets {
    pub cookie_key: CookieKey,
}

impl Secrets {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let secrets_path = ".bento_secrets";
        if Path::new(secrets_path).exists() {
            let secrets_str = fs::read_to_string(secrets_path)?;
            let secrets: Secrets = toml::from_str(&secrets_str).expect("valid secrets file");
            Ok(secrets)
        } else {
            // Generate a new cookie key if secrets file doesn't exist
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

// serde glue for saving cookie key
impl Serialize for CookieKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.0.master())
    }
}

impl<'de> Deserialize<'de> for CookieKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = <Vec<u8>>::deserialize(deserializer)?;
        Ok(CookieKey(cookie::Key::from(&bytes)))
    }
}
