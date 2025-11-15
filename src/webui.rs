pub mod home;
pub mod login_screen;

use home::Home;

use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path,
};
use lucide_leptos::Bell;
use thiserror::Error;

use crate::webui::login_screen::LoginScreen;

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
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/bento.css" />

        // sets the document title
        <Title text="Bento: Backend Toolbox" />

        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                <Route path=path!("/") view=LoginScreen />
                <ParentRoute path=path!("") view=TopBar>
                    <Route path=path!("/home") view=Home />
                </ParentRoute>
            </Routes>
        </Router>
    }
}

#[component]
pub fn TopBar() -> impl IntoView {
    view! {
        <>
            <div class="navbar bg-base-100 px-6">
                <div class="navbar-start gap-6">
                    <img class="text-left" src="/bento-dark-64.webp" width=36 />
                    <h1 class="text-xl font-bold">Bento</h1>
                </div>
                <div class="navbar-end gap-6">
                    <a class="link link-hover" href="#">Documentation</a>
                    <button class="btn btn-ghost" on:click=|_| println!("clicked!")>
                        <Bell />
                    </button>
                    <a class="link link-hover" href="#">Logout</a>
                </div>
            </div>
            <hr class="border-white/15" />
        </>
    }
}

#[derive(Debug, Error)]
enum ServerError {
    #[error("Invalid credentials provided")]
    InvalidCreds,
    #[error("Client request error")]
    RequestError,
    #[error("An unknown error occurred")]
    Unknown,
}

use crate::types::{Session, SessionIp};

#[server]
pub async fn login(username: String, password: String) -> Result<Session, ServerFnError> {
    // place server-specific use statements within ssr-gated code
    use crate::server::AppState;
    use crate::storage::AuthStore;
    use crate::types::{User, Username};
    use axum_client_ip::ClientIp;
    use tower_cookies::{Cookie, Cookies, cookie::SameSite};

    // unwrap used here because this is basic plumbing done at initialization
    let app_state: AppState = use_context().expect("Axum state in leptos context");
    let auth_store = app_state.auth_store.clone();
    let cookies: Cookies = leptos_axum::extract().await?;
    let ClientIp(client_ip) = leptos_axum::extract().await?;

    // Strong types for username and password
    let username = Username(username);
    let pass_hash =
        PasswordHash::try_from(password.as_str()).map_err(|_| ServerError::RequestError)?;

    let user: User = auth_store.get_user_by_username(&username).await?;

    if user.password_hash.verify(pass_hash.as_str()) {
        let session_ip = SessionIp(client_ip);
        let session = auth_store.issue_session(&user.id, session_ip).await?;

        // Set the auth cookie
        #[cfg(not(debug_assertions))]
        let cookie = Cookie::build(("session_id", session.id.as_str().to_string()))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .build();

        #[cfg(debug_assertions)] // in debug mode we probably wont have https
        let cookie = Cookie::build(("session_id", session.id.as_str().to_string()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .build();

        cookies.add(cookie);

        Ok(session)
    } else {
        Err(ServerError::InvalidCreds.into())
    }
}
