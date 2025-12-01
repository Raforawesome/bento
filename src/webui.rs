#[cfg(feature = "ssr")]
pub mod cookies;
pub mod icons;
pub mod screen_home;
pub mod screen_login;

use screen_home::Home;

use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::{
    types::{AppError, Session},
    webui::screen_login::LoginScreen,
};

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
    let auth_user = Resource::new(|| (), |_| get_current_user());
    let fallback =
        || view! { <div class="min-h-screen flex items-center justify-center">"Loading..."</div> };

    view! {

        <Suspense fallback=fallback>
            {move || {
                auth_user.get().map(|result| {
                    match result {
                        Ok(Some(user)) => view! {
                            <Home user=user />
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
    use crate::webui::cookies::{SESSION_COOKIE_NAME, clear_session_cookie};
    use axum_extra::extract::CookieJar;
    use leptos_axum::ResponseOptions;
    use leptos_axum::extract;

    // extract the cookie jar from the request
    let jar: CookieJar = extract().await?;

    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
        let app_state: AppState = use_context().expect("Axum state in leptos context");
        let auth_store = app_state.auth_store.clone();
        let session_id = SessionId(cookie.value().to_string());

        // Revoke the session in the store
        let _ = auth_store.revoke_session(&session_id).await;
    }

    // Clear the cookie
    let response = expect_context::<ResponseOptions>();
    clear_session_cookie(&response);

    Ok(())
}
