#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    /*
     * Static code (placed here to only be compiled in server binary)
     */
    use std::{net::SocketAddr, sync::Arc};

    use axum::{
        Router,
        extract::FromRef,
        http::StatusCode,
        routing::{get, post},
    };
    use axum_client_ip::ClientIpSource;
    use foundry::{api, storage::memstore::MemoryAuthStore};
    use leptos::{config::LeptosOptions, prelude::*};
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tower_http::{compression::CompressionLayer, decompression::RequestDecompressionLayer};
    use tracing::{debug, info};

    const ADDR: &str = "0.0.0.0:8000"; // local address to run webserver on

    type ConcreteAuthStore = MemoryAuthStore; // declare which implementation of AuthStore to use

    // Unified AppState struct
    #[derive(Clone)]
    pub struct AppState {
        pub leptos_options: LeptosOptions,
        pub auth_store: Arc<ConcreteAuthStore>,
    }

    // Axum uses FromRef impls to clone "sub-state" into routers
    impl FromRef<AppState> for Arc<ConcreteAuthStore> {
        fn from_ref(state: &AppState) -> Self {
            state.auth_store.clone()
        }
    }

    impl FromRef<AppState> for LeptosOptions {
        fn from_ref(state: &AppState) -> Self {
            state.leptos_options.clone()
        }
    }
    /*
     * end static code
     */

    // Setup tracing for logging
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_file(false)
        .with_line_number(true)
        .with_target(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
    info!("Starting Foundry BaaS server on {}", ADDR);

    // Initialize the auth store
    let auth_store = Arc::new(MemoryAuthStore::new());
    debug!("Authentication store initialized");

    // Set up leptos webui
    let leptos_conf = get_configuration(None).unwrap();
    let leptos_routes = generate_route_list(foundry::webui::App);
    let leptos_options = leptos_conf.leptos_options;

    let app_state = AppState {
        leptos_options,
        auth_store: auth_store.clone(),
    };

    // define api sub-router for the server
    let api = Router::new()
        .route(
            "/api/v1/register",
            post(foundry::api::auth::register::<ConcreteAuthStore>),
        )
        .route(
            "/api/v1/login",
            post(foundry::api::auth::login::<ConcreteAuthStore>),
        );
    // .with_state(auth_store.clone());

    // define ssr'ed webui sub-router
    let ssr: Router<LeptosOptions> = Router::new()
        .leptos_routes(&app_state.leptos_options, leptos_routes, {
            let leptos_options = app_state.leptos_options.clone();
            move || foundry::webui::shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(foundry::webui::shell));

    let ssr_service = ssr.into_service();

    // Unify both sub-routers under one
    let app = Router::new()
        .merge(api)
        .merge(ssr)
        .layer(
            RequestDecompressionLayer::new()
                .br(true)
                .gzip(true)
                .pass_through_unaccepted(false),
        )
        .layer(CompressionLayer::new())
        .layer(ClientIpSource::ConnectInfo.into_extension());

    // Start the server
    info!("Binding to address: {}", ADDR);
    let listener = tokio::net::TcpListener::bind(ADDR).await.unwrap();
    info!("Server started successfully!");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
