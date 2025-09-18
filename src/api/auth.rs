use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_client_ip::ClientIp;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::storage::{
    AuthStore, PasswordHash, Role, Session, SessionIp, SessionToken, User, UserId, Username,
};

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    username: Username,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    username: Username,
    role: Role,
    session: Session,
}

pub async fn register<S: AuthStore>(
    State(store): State<Arc<S>>,
    ClientIp(client_ip): ClientIp,
    Json(req): Json<AuthRequest>,
) -> Response {
    todo!()
}

pub async fn login<S: AuthStore>(State(store): State<S>, Json(req): Json<AuthRequest>) -> Response {
    todo!()
}
