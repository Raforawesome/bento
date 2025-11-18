pub mod home;
pub mod login_screen;

use home::Home;

use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
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
                                <TopBar />
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

/// server function to check if user is authenticated
#[server]
pub async fn check_auth() -> Result<bool, ServerFnError> {
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
