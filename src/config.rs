use std::sync::LazyLock;

use serde::Deserialize;
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
