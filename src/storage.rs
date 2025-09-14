use serde::{Deserialize, Serialize};
use uuid::Uuid;

/*
 * Newtype wrappers for strong typing
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
}

pub struct User;

pub trait UserStore: Send {
    fn test() -> impl Future<Output = ()> + Send;
    fn create_user() -> impl Future<Output = User> + Send;
}
