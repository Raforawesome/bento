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

#[server]
pub async fn login(username: String, password: String) -> Result<(), ServerFnError> {
    // place uses within ssr-gated code
    use crate::config::Admin;

    let app_cfg = crate::config::LOCAL_CONF.as_ref();
    let Admin {
        username: admin_username, // destructure the username field and assign to local var `admin_username`
        password: admin_password, // since we're destructuring a ref these local vars are also references
    } = &app_cfg.admin;

    println!(
        "{username} {password} compared to: {} {}",
        admin_username, admin_password
    );

    if admin_username == &username && admin_password == &password {
        Ok(())
    } else {
        Err(ServerFnError::ServerError("Invalid credentials".into()))
    }
}
