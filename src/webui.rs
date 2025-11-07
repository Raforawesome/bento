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

enum ServerError {
    InvalidCreds,
    RequestError,
    Unknown,
}

#[server]
pub async fn login(username: String, password: String) -> Result<(), ServerFnError> {
    // place server-specific use statements within ssr-gated code
    use crate::server::AppState;
    use crate::storage::{AuthStore, PasswordHash, User, Username};
    use axum_client_ip::ClientIp;
    use axum_extra::extract::cookie::CookieJar;

    // unwrap used here because this is basic plumbing done at initialization
    let app_state: AppState = use_context().expect("Axum state in leptos context");
    let auth_store = app_state.auth_store.clone();
    let jar: CookieJar = leptos_axum::extract().await?;
    let ClientIp(client_ip) = leptos_axum::extract().await?;

    // Strong types for username and password
    let username = Username(username);
    let pass_hash =
        PasswordHash::try_from(password.as_str()).map_err(|_| ServerError::RequestError)?;

    let user: User = auth_store.get_user_by_username(&username).await?;

    Err(ServerError::InvalidCreds.into())
}

impl From<ServerError> for ServerFnError {
    fn from(err: ServerError) -> Self {
        match err {
            ServerError::InvalidCreds => ServerFnError::ServerError("Invalid credentials".into()),
            ServerError::RequestError => ServerFnError::ServerError("Client request error".into()),
            ServerError::Unknown => ServerFnError::ServerError("An unknown error occurred".into()),
        }
    }
}
