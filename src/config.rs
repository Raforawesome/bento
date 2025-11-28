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
    #[serde(default)]
    pub server: Server,
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

#[derive(Deserialize)]
pub struct Server {
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            address: default_address(),
            port: default_port(),
        }
    }
}

impl Server {
    /// Returns the full socket address string (e.g., "0.0.0.0:8000")
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

fn default_address() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8000
}

pub fn grab_config() -> Result<Config, de::Error> {
    let config_str = std::fs::read_to_string("bento.toml").expect("a file called ./bento.toml");
    toml::from_str(&config_str)
}

/*
 * Secrets Manager
 */
use std::fs;

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

// TODO: replace Box<dyn Error> with anyhow::Error
impl Secrets {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let secrets_path = ".bento_secrets";
        let secrets_str = fs::read_to_string(secrets_path)?;
        let secrets: Secrets = toml::from_str(&secrets_str)?;
        Ok(secrets)
    }

    pub fn load_or_init() -> Result<Self, Box<dyn std::error::Error>> {
        match Self::load() {
            Ok(secrets) => Ok(secrets),
            Err(_) => {
                tracing::info!("Generating default .bento_secrets file...");
                let secrets = Self::default();
                secrets.save()?;
                Ok(secrets)
            }
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
