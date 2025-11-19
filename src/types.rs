use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use time::OffsetDateTime;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use argon2::{
    Argon2,
    password_hash::{
        PasswordHashString, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng as ArgonRng,
    },
};
#[cfg(feature = "ssr")]
use base64::{Engine as _, engine::general_purpose::URL_SAFE as Base64Url};
#[cfg(feature = "ssr")]
use rand::rngs::OsRng;

/*
 * Newtype wrappers for strong typing
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHash(
    #[cfg(feature = "ssr")] PasswordHashString,
    #[cfg(not(feature = "ssr"))] String,
);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionIp(pub IpAddr);

/// An enum to represent a user's permission level;
/// - Admins:
///   Can create other users
///
///   TODO: Add the ability to manage other user's workspaces
///
/// - Users:
///   Can manage their own workspaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Admin,
    User,
}

/// Main user abstraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub password_hash: PasswordHash,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub ip: SessionIp,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

/*
 * Implementations on newtype wrappers
 */
impl UserId {
    pub fn new() -> Self {
        UserId(Uuid::now_v7())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "ssr")]
impl SessionId {
    pub fn new() -> Self {
        use rand::TryRngCore as _;

        let mut buf = [0_u8; 120];
        if OsRng.try_fill_bytes(&mut buf).is_ok() {
            SessionId(Base64Url.encode(buf))
        } else {
            panic!("Failed to generate secure numbers from the operating system.");
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "ssr")]
impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "ssr")]
impl PasswordHash {
    pub fn verify<B: AsRef<[u8]>>(&self, password: B) -> bool {
        let pass_bytes: &[u8] = password.as_ref();

        Argon2::default()
            .verify_password(pass_bytes, &self.0.password_hash())
            .is_ok()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(feature = "ssr")]
impl TryFrom<&[u8]> for PasswordHash {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let pass_bytes: &[u8] = value;
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(pass_bytes, &salt)?.to_string();
        Ok(Self(PasswordHashString::new(&password_hash)?))
    }
}

#[cfg(feature = "ssr")]
impl TryFrom<&str> for PasswordHash {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let pass_bytes: &[u8] = value.as_bytes();
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(pass_bytes, &salt)?.to_string();
        Ok(Self(PasswordHashString::new(&password_hash)?))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Invalid credentials provided")]
    InvalidCreds,
    #[error("Client request error")]
    RequestError,
    #[error("An unknown error occurred")]
    Unknown,
}

impl serde::Serialize for PasswordHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[cfg(feature = "ssr")]
        {
            serializer.serialize_str(self.0.as_str())
        }
        #[cfg(not(feature = "ssr"))]
        {
            serializer.serialize_str(&self.0)
        }
    }
}

impl<'de> serde::Deserialize<'de> for PasswordHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        #[cfg(feature = "ssr")]
        {
            use argon2::password_hash::PasswordHashString;
            PasswordHashString::new(&s)
                .map(PasswordHash)
                .map_err(serde::de::Error::custom)
        }
        #[cfg(not(feature = "ssr"))]
        {
            Ok(PasswordHash(s))
        }
    }
}
