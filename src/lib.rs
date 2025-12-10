#![feature(impl_trait_in_bindings, bool_to_result)]
#[cfg(feature = "ssr")]
pub mod server {
    use axum::extract::FromRef;
    use axum_extra::extract::cookie::Key;
    use std::sync::Arc;
    // declare which implementation of AuthStore to use
    use super::storage::{redb_authstore::RedbAuthStore, redb_projectstore::RedbProjectStore};
    use leptos::config::LeptosOptions;

    // Unified AppState struct
    #[derive(Clone)]
    pub struct AppState {
        pub leptos_options: LeptosOptions,
        pub auth_store: Arc<RedbAuthStore>,
        pub project_store: Arc<RedbProjectStore>,
        pub cookie_key: Key,
    }

    // Axum uses FromRef impls to clone "sub-state" into routers
    impl FromRef<AppState> for Arc<RedbAuthStore> {
        fn from_ref(state: &AppState) -> Self {
            state.auth_store.clone()
        }
    }

    impl FromRef<AppState> for Arc<RedbProjectStore> {
        fn from_ref(state: &AppState) -> Self {
            state.project_store.clone()
        }
    }

    impl FromRef<AppState> for LeptosOptions {
        fn from_ref(state: &AppState) -> Self {
            state.leptos_options.clone()
        }
    }

    impl FromRef<AppState> for Key {
        fn from_ref(state: &AppState) -> Self {
            state.cookie_key.clone()
        }
    }
}

#[cfg(all(feature = "ssr", feature = "rest-api"))]
pub mod api;
#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod storage;

pub mod types;
pub mod webui;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::leptos_dom::logging::console_log("Hydrating client...");
    leptos::mount::hydrate_body(webui::App);
}
