use axum::{http::StatusCode, routing::get};
use foundry::storage::memstore::MemoryAuthStore;

const ADDR: &str = "0.0.0.0:8000";

#[tokio::main]
async fn main() {
    println!("Starting Foundry BaaS server on {}", ADDR);

    // Initialize the auth store
    let auth_store = MemoryAuthStore::new();

    // Create the router with all API routes
    let app = axum::Router::new()
        .route("/", get(async || "Welcome to Foundry BaaS!"))
        .route("/api", get(async || StatusCode::BAD_GATEWAY))
        .with_state(auth_store);

    // Start the server
    let listener = tokio::net::TcpListener::bind(ADDR).await.unwrap();
    println!("Server started successfully!");
    axum::serve(listener, app).await.unwrap();
}
