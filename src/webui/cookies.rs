//! Cookie helper functions for session management

use axum::http::header::{HeaderValue, SET_COOKIE};
use axum_extra::extract::cookie::{Cookie, SameSite};
use leptos_axum::ResponseOptions;
use time::Duration;

/// Cookie name for session identification
pub const SESSION_COOKIE_NAME: &str = "session_id";

/// Builds a session cookie with the given value.
///
/// The cookie is configured with:
/// - `HttpOnly`: true (not accessible via JavaScript)
/// - `SameSite`: Lax (sent with top-level navigations)
/// - `Secure`: true in release builds only
/// - `Path`: "/" (available site-wide)
fn build_session_cookie(value: &str, max_age: Option<Duration>) -> Cookie<'static> {
    let builder = Cookie::build((SESSION_COOKIE_NAME, value.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    // Add max_age if provided (used for cookie expiration/deletion)
    let builder = if let Some(age) = max_age {
        builder.max_age(age)
    } else {
        builder
    };

    // Only set Secure flag in release builds
    #[cfg(not(debug_assertions))]
    let builder = builder.secure(true);

    builder.build()
}

/// Sets a session cookie on the response with the given session ID.
///
/// # Example
/// ```ignore
/// let response = expect_context::<ResponseOptions>();
/// set_session_cookie(&response, &session.id.0);
/// ```
pub fn set_session_cookie(response: &ResponseOptions, session_id: &str) {
    let cookie = build_session_cookie(session_id, None);

    if let Ok(header_value) = HeaderValue::from_str(&cookie.to_string()) {
        response.insert_header(SET_COOKIE, header_value);
    }
}

/// Clears the session cookie by setting it to empty with immediate expiration.
///
/// # Example
/// ```ignore
/// let response = expect_context::<ResponseOptions>();
/// clear_session_cookie(&response);
/// ```
pub fn clear_session_cookie(response: &ResponseOptions) {
    let cookie = build_session_cookie("", Some(Duration::seconds(0)));

    if let Ok(header_value) = HeaderValue::from_str(&cookie.to_string()) {
        response.insert_header(SET_COOKIE, header_value);
    }
}
