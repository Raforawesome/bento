use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_client_ip::ClientIp;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::storage::{AuthError, AuthStore, PasswordHash, Role, Session, SessionIp, Username};

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

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status: StatusCode = self.into();
        status.into_response()
    }
}

impl From<AuthError> for StatusCode {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::UserExists => StatusCode::BAD_REQUEST,
            AuthError::NotFound => StatusCode::UNAUTHORIZED,
            AuthError::InvalidSession => StatusCode::FORBIDDEN,
            AuthError::SessionLimitReached => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}

pub async fn register<S: AuthStore>(
    State(store): State<Arc<S>>,
    ClientIp(client_ip): ClientIp,
    Json(req): Json<AuthRequest>,
) -> Response {
    debug!("Registration attempt from IP: {}", client_ip);

    if (store.get_user_by_username(&req.username).await).is_ok() {
        debug!("Registration failed: username already exists");
        StatusCode::BAD_REQUEST.into_response()
    } else {
        let AuthRequest { username, password } = req;
        debug!("Creating new user");

        match store
            .create_standard_user(username, PasswordHash::from(&password))
            .await
        {
            Ok(user) => {
                debug!(user_id = %user.id.0, "User created successfully");
                // create token
                debug!("Issuing session for new user");
                match store
                    .issue_session(&user.id, SessionIp(client_ip.to_string()))
                    .await
                {
                    Ok(session) => {
                        debug!(
                            user_id = %user.id.0,
                            expires_at = %session.expires_at,
                            "Session created successfully"
                        );
                        let response = AuthResponse {
                            username: user.username.clone(),
                            role: user.role,
                            session,
                        };
                        (StatusCode::CREATED, Json(response)).into_response()
                    }
                    Err(err) => {
                        error!(user_id = %user.id.0, error = %err, "Failed to create session");
                        err.into_response()
                    }
                }
            }
            Err(err) => {
                error!(?err, "Failed to create user");
                err.into_response()
            }
        }
    }
}

pub async fn login<S: AuthStore>(
    State(store): State<Arc<S>>,
    ClientIp(client_ip): ClientIp,
    Json(req): Json<AuthRequest>,
) -> Response {
    match store.get_user_by_username(&req.username).await {
        Ok(user) => {
            if user.password_hash.verify(&req.password) {
                debug!(user_id = %user.id.0, "Password verified, issuing session");
                match store
                    .issue_session(&user.id, SessionIp(client_ip.to_string()))
                    .await
                {
                    Ok(session) => {
                        debug!(
                            user_id = %user.id.0,
                            expires_at = %session.expires_at,
                            "Session created successfully"
                        );
                        let response = AuthResponse {
                            username: user.username,
                            role: user.role,
                            session,
                        };
                        (StatusCode::OK, Json(response)).into_response()
                    }
                    Err(err) => {
                        error!(user_id = %user.id.0, error = %err, "Failed to create session");
                        err.into_response()
                    }
                }
            } else {
                debug!(user_id = %user.id.0, "Password verification failed");
                StatusCode::UNAUTHORIZED.into_response()
            }
        }
        Err(err) => err.into_response(),
    }
}
