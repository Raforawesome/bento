#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    /*
     * Static code (placed here to only be compiled in server binary)
     */
    use std::{net::SocketAddr, sync::Arc};

    use axum::Router;
    #[cfg(feature = "rest-api")]
    use axum::routing::post;
    #[cfg(feature = "rest-api")]
    use axum_client_ip::ClientIpSource;
    use bento::server::AppState;
    #[cfg(feature = "rest-api")]
    use bento::server::ConcreteAuthStore;
    use bento::{storage::memstore::MemoryAuthStore, webui};
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, file_and_error_handler, generate_route_list};
    use tower_http::{compression::CompressionLayer, decompression::RequestDecompressionLayer};
    use tracing::{debug, info};

    const ADDR: &str = "0.0.0.0:8000"; // local address to run webserver on
    const MAX_SESSIONS_PER_USER: usize = 5;

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
    info!("Starting Bento BaaS server on {}", ADDR);

    // Initialize the auth store
    let auth_store = Arc::new(MemoryAuthStore::new(MAX_SESSIONS_PER_USER));
    debug!("Authentication store initialized");

    // Set up leptos webui
    let leptos_conf = get_configuration(None).unwrap();
    let leptos_routes = generate_route_list(webui::App);
    let leptos_options = leptos_conf.leptos_options;

    let app_state = AppState {
        leptos_options,
        auth_store: auth_store.clone(),
    };

    // define api sub-router for the server
    #[cfg(feature = "rest-api")]
    let api = Router::new()
        .route(
            "/api/v1/register",
            post(bento::api::auth::register::<ConcreteAuthStore>),
        )
        .route(
            "/api/v1/login",
            post(bento::api::auth::login::<ConcreteAuthStore>),
        );

    // define ssr'ed webui sub-router
    let ssr = Router::new().leptos_routes_with_context(
        &app_state,
        leptos_routes,
        {
            let app_state = app_state.clone();
            move || provide_context(app_state.clone())
        },
        {
            let opts = app_state.clone();
            move || webui::shell(opts.leptos_options.clone())
        },
    );

    // Unify both sub-routers under one
    #[cfg(feature = "rest-api")]
    let app: Router = Router::new()
        .merge(api)
        .merge(ssr)
        .fallback(file_and_error_handler::<AppState, _>(webui::shell)) // fallback for static files & 404s
        .layer(RequestDecompressionLayer::new().br(true).gzip(true))
        .layer(CompressionLayer::new().br(true).gzip(true))
        .with_state(app_state)
        .layer(ClientIpSource::ConnectInfo.into_extension());

    #[cfg(not(feature = "rest-api"))]
    let app: Router = Router::new()
        .merge(ssr)
        .fallback(file_and_error_handler::<AppState, _>(webui::shell)) // fallback for static files & 404s
        .layer(RequestDecompressionLayer::new().br(true).gzip(true))
        .layer(CompressionLayer::new().br(true).gzip(true))
        .with_state(app_state);

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
