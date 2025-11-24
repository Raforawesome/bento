pub mod home;
pub mod icons;
pub mod login_screen;

use home::Home;

use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use serde::{Deserialize, Serialize};
use server_fn::{codec::JsonEncoding, error::ServerFnErrorErr};

use crate::{types::Session, webui::login_screen::LoginScreen};

/// Universal error type that automatically converts from any error.
///
/// This type implements `FromServerFnError` and uses downcasting
/// to provide user-friendly error messages for known error types, while
/// gracefully handling unknown errors.
///
/// No manual `From` implementations are needed - any `std::error::Error` can be
/// automatically converted using the `?` operator.
///
/// ## Example
/// ```rust
/// #[server]
/// pub async fn my_function() -> Result<Data, AppError> {
///     let user = auth_store.get_user(&id).await?;  // AuthError → AppError
///     let data = fetch_data().await?;              // Any error → AppError
///     Ok(data)
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError(String);

impl AppError {
    /// Create a new AppError with a custom message
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        Self::new(format!("Server function error: {:?}", value))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Universal error conversion using downcasting for user-friendly messages
#[cfg(feature = "ssr")]
impl<E> From<E> for AppError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        use std::any::Any;

        // Try to downcast to known error types for better messages
        let err_any: &dyn Any = &err;

        // Check for AuthError
        if let Some(auth_err) = err_any.downcast_ref::<crate::storage::AuthError>() {
            use crate::storage::AuthError;
            return Self::new(match auth_err {
                AuthError::NotFound => "User not found",
                AuthError::InvalidSession => "Your session has expired. Please log in again.",
                AuthError::UserExists => "A user with this username already exists",
                AuthError::SessionLimitReached => {
                    "Maximum number of active sessions reached. Please log out of another device."
                }
            });
        }

        // Check for ServerError
        if let Some(server_err) = err_any.downcast_ref::<crate::types::ServerError>() {
            use crate::types::ServerError;
            return Self::new(match server_err {
                ServerError::InvalidCreds => "Invalid username or password",
                ServerError::RequestError => "Request error occurred",
                ServerError::Unknown => "An unknown error occurred",
            });
        }

        // Default: convert to string
        Self::new(err.to_string())
    }
}

// Client-side: can't use the generic From impl, so handle ServerFnErrorErr specifically
#[cfg(not(feature = "ssr"))]
impl From<ServerFnErrorErr> for AppError {
    fn from(err: ServerFnErrorErr) -> Self {
        Self::new(format!("Server function error: {:?}", err))
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="night">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link href="https://cdn.jsdelivr.net/npm/daisyui@5" rel="stylesheet" type="text/css" />
                <link href="https://cdn.jsdelivr.net/npm/daisyui@5/themes.css" rel="stylesheet" type="text/css" />
                <script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" />
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/bento.css" />

        // sets the document title
        <Title text="Bento: Backend Toolbox" />

        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                <Route path=path!("/") view=RootView />
            </Routes>
        </Router>
    }
}

/// root view that dynamically renders LoginScreen or Home based on auth state
#[component]
pub fn RootView() -> impl IntoView {
    let check_auth = Resource::new(|| (), |_| check_auth());
    let fallback =
        || view! { <div class="min-h-screen flex items-center justify-center">"Loading..."</div> };

    view! {

        <Suspense fallback=fallback>
            {move || {
                check_auth.get().map(|result| {
                    match result {
                        Ok(true) => view! {
                            <>
                                <Home />
                            </>
                        }.into_any(),
                        _ => view! { <LoginScreen /> }.into_any(),
                    }
                })
            }}
        </Suspense>
    }
}

#[component]
pub fn LogoSvg(size: i32, #[prop(optional)] class: Option<&'static str>) -> impl IntoView {
    view! {
        <img
            class={format!("h-{size} w-{size} {}", class.unwrap_or_default())}
            src="/bento-dark.svg"
            alt="Bento logo"
        />
    }
}

/// server function to check if user is authenticated
#[server]
pub async fn check_auth() -> Result<bool, AppError> {
    use crate::server::AppState;
    use crate::storage::AuthStore;
    use crate::types::SessionId;
    use axum_extra::extract::CookieJar;
    use leptos_axum::extract;

    // extract the cookie jar from the request
    let jar: CookieJar = extract().await?;

    if let Some(cookie) = jar.get("session_id") {
        let app_state: AppState = use_context().expect("Axum state in leptos context");
        let auth_store = app_state.auth_store.clone();
        let session_id = SessionId(cookie.value().to_string());
        Ok(auth_store.fetch_session(&session_id).await.is_ok())
    } else {
        Ok(false)
    }
}

/// Helper function to authenticate a user and issue a session.
///
/// Returns the issued session if authentication succeeds.
#[cfg(feature = "ssr")]
async fn authenticate_user(
    username: &str,
    password: &str,
    client_ip: std::net::IpAddr,
) -> Result<Session, AppError> {
    use crate::server::AppState;
    use crate::storage::AuthStore;
    use crate::types::{SessionIp, Username};

    let app_state: AppState = use_context().expect("Axum state in leptos context");
    let auth_store = app_state.auth_store.clone();

    // strong type for username
    let username = Username(username.to_string());

    // Get user by username
    let user = auth_store.get_user_by_username(&username).await?;

    // Verify password
    if user.password_hash.verify(password) {
        let session_ip = SessionIp(client_ip);
        let session = auth_store.issue_session(&user.id, session_ip).await?;
        Ok(session)
    } else {
        Err(AppError::new("Invalid username or password"))
    }
}

/// Server function to fetch the current user's session from the cookie.
///
/// Returns `Some(Session)` if a valid session exists, `None` otherwise.
/// This is a low-level function - consider using `get_current_user()` for user info.
#[server]
pub async fn fetch_session() -> Result<Option<Session>, AppError> {
    use crate::server::AppState;
    use crate::storage::AuthStore;
    use crate::types::SessionId;
    use axum_extra::extract::CookieJar;
    use leptos_axum::extract;

    // extract the cookie jar from the request
    let jar: CookieJar = extract().await?;

    if let Some(cookie) = jar.get("session_id") {
        let app_state: AppState = use_context().expect("Axum state in leptos context");
        let auth_store = app_state.auth_store.clone();
        let session_id = SessionId(cookie.value().to_string());

        match auth_store.fetch_session(&session_id).await {
            Ok(session) => Ok(Some(session)),
            Err(_) => Ok(None),
        }
    } else {
        Ok(None)
    }
}

/// User information returned by `get_current_user`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CurrentUser {
    pub username: String,
    pub role: crate::types::Role,
    pub user_id: String,
}

/// Server function to get the current authenticated user's information.
///
/// Returns `Ok(Some(CurrentUser))` with username and role if authenticated, `Ok(None)` otherwise.
/// Returns `Err(_)` in the case of an error with the server function call.
///
/// # Example Usage
///
/// ```rust
/// let user = Resource::new(|| (), |_| get_current_user());
///
/// view! {
///     <Suspense fallback=|| view! { <p>"Loading..."</p> }>
///         {move || {
///             user.get().map(|result| {
///                 match result {
///                     Ok(Some(user)) => view! {
///                         <p>"Welcome, " {&user.username}</p>
///                     },
///                     _ => view! { <p>"Not logged in"</p> },
///                 }
///             })
///         }}
///     </Suspense>
/// }
/// ```
#[server]
pub async fn get_current_user() -> Result<Option<CurrentUser>, AppError> {
    use crate::server::AppState;
    use crate::storage::AuthStore;

    // Use the fetch_session function to get the session
    let session = fetch_session().await?;

    if let Some(session) = session {
        let app_state: AppState = use_context().expect("Axum state in leptos context");
        let auth_store = app_state.auth_store.clone();

        // Fetch the user details
        match auth_store.get_user_by_id(&session.user_id).await {
            Ok(user) => Ok(Some(CurrentUser {
                username: user.username.0,
                role: user.role,
                user_id: user.id.0.to_string(),
            })),
            Err(_) => Ok(None),
        }
    } else {
        Ok(None)
    }
}

#[server]
pub async fn logout() -> Result<(), AppError> {
    use crate::server::AppState;
    use crate::storage::AuthStore;
    use crate::types::SessionId;
    use axum::http::header::{HeaderValue, SET_COOKIE};
    use axum_extra::extract::CookieJar;
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use leptos_axum::ResponseOptions;
    use leptos_axum::extract;

    // extract the cookie jar from the request
    let jar: CookieJar = extract().await?;

    if let Some(cookie) = jar.get("session_id") {
        let app_state: AppState = use_context().expect("Axum state in leptos context");
        let auth_store = app_state.auth_store.clone();
        let session_id = SessionId(cookie.value().to_string());

        // Revoke the session in the store
        let _ = auth_store.revoke_session(&session_id).await;
    }

    // Clear the cookie
    let response = expect_context::<ResponseOptions>();
    let cookie = Cookie::build(("session_id", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(0));

    #[cfg(not(debug_assertions))]
    let cookie = cookie.secure(true);

    let cookie = cookie.build();

    if let Ok(header_value) = HeaderValue::from_str(&cookie.to_string()) {
        response.insert_header(SET_COOKIE, header_value);
    }

    Ok(())
}
