#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    /*
     * Static code (placed here to only be compiled in server binary)
     */
    use std::{net::SocketAddr, sync::Arc};

    use axum::{
        http::StatusCode,
        routing::{get, post},
    };
    use axum_client_ip::ClientIpSource;
    use foundry::{api, storage::memstore::MemoryAuthStore};
    use tower_http::{compression::CompressionLayer, decompression::RequestDecompressionLayer};
    use tracing::{debug, info};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    const ADDR: &str = "0.0.0.0:8000"; // local address to run webserver on

    type ConcreteAuthStore = MemoryAuthStore; // declare which implementation of AuthStore to use
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
    let auth_store = MemoryAuthStore::new();
    debug!("Authentication store initialized");

    // Set up leptos webui
    let leptos_conf = get_configuration(None).unwrap();
    let leptos_options = leptos_conf.leptos_options;
    let routes = generate_route_list(foundry::webui::App);

    // Create the router with all API routes
    let app = axum::Router::new()
        .route("/", get(async || "Welcome to Foundry BaaS!"))
        .route(
            "/api/v1/register",
            post(api::auth::register::<ConcreteAuthStore>),
        )
        .route("/api/v1/login", post(api::auth::login::<ConcreteAuthStore>))
        .fallback(async || StatusCode::NOT_FOUND)
        .layer(
            RequestDecompressionLayer::new()
                .br(true)
                .gzip(true)
                .pass_through_unaccepted(false),
        ) // decompress incoming requests
        .layer(CompressionLayer::new()) // compress responses (auto negotiates)
        .layer(ClientIpSource::ConnectInfo.into_extension()) // provide client ip extractors
        .with_state(Arc::new(auth_store)); // put auth store in global state

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
