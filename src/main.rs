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

    // Create the router with all API routes
    let app = axum::Router::new()
        .route("/", get(async || "Welcome to Foundry BaaS!"))
        .route("/api", get(async || StatusCode::SERVICE_UNAVAILABLE))
        .route("/api/v1", get(async || StatusCode::SERVICE_UNAVAILABLE))
        .route(
            "/api/v1/register",
            post(api::auth::register::<ConcreteAuthStore>),
        )
        .route("/api/v1/login", post(api::auth::login::<ConcreteAuthStore>))
        .layer(RequestDecompressionLayer::new()) // decompress incoming requests
        .layer(CompressionLayer::new()) // compress responses (auto negotaties)
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
