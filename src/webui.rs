pub mod home;

use home::Home;

use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{ParentRoute, Route, Router, Routes},
    path,
};
use lucide_leptos::Bell;

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
        <Title text="Welcome to Leptos" />

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <ParentRoute path=path!("") view=TopBar>
                        <Route path=StaticSegment("/") view=Home />
                    </ParentRoute>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn TopBar() -> impl IntoView {
    view! {
        <div class="navbar bg-base-100 shadow-sm px-6">
            <div class="navbar-start gap-6">
                <img class="text-left" src="/bento-dark-64.webp" width=36 />
                <h1 class="text-xl font-bold">Bento</h1>
            </div>
            <div class="navbar-end gap-6">
                <a class="link link-hover" href="#">Documentation</a>
                <Bell />
                <a class="link link-hover" href="#">Logout</a>
            </div>
        </div>
    }
}
