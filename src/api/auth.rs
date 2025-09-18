use axum::{
    extract::{Json, State},
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::storage::{AuthStore, Role};

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    user_id: String,
    username: String,
    token: String,
    role: Role,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    error: String,
}

pub async fn register<S: AuthStore>(
    State(store): State<S>,
    Json(req): Json<AuthRequest>,
) -> Response {
    todo!()
}

pub async fn login<S: AuthStore>(State(store): State<S>, Json(req): Json<AuthRequest>) -> Response {
    todo!()
}
