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
    use axum_client_ip::ClientIpSource;
    use bento::config::{Admin, CookieKey, LOCAL_CONF};
    #[cfg(feature = "rest-api")]
    use bento::server::ConcreteAuthStore;
    use bento::storage::AuthStore;
    use bento::storage::redbstore::RedbAuthStore;
    use bento::types::PasswordHash;
    use bento::webui;
    use bento::{
        config::{self, Secrets},
        server::AppState,
    };
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, file_and_error_handler, generate_route_list};
    use tower_http::{compression::CompressionLayer, decompression::RequestDecompressionLayer};
    use tracing::{debug, error, info, warn};

    const MAX_SESSIONS_PER_USER: usize = 5;

    /*
     * end static code
     */

    // set up tracing for logging
    let time_format =
        time::format_description::parse("[hour]:[minute]:[second].[subsecond digits:2]")
            .expect("Failed to parse time format description.");
    let local_offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(local_offset, time_format);

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_timer(timer)
        .with_file(false)
        .with_line_number(true)
        .with_target(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // initialize the auth store
    // let auth_store = Arc::new(MemoryAuthStore::new(MAX_SESSIONS_PER_USER));
    // create data directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all("data") {
        error!("Failed to create data directory: {e}");
        std::process::exit(1);
    }
    let auth_store = Arc::new(RedbAuthStore::new("data/auth.db", MAX_SESSIONS_PER_USER).unwrap());
    debug!("Authentication store initialized");

    // set up leptos webui
    let leptos_conf = get_configuration(None).unwrap();
    let leptos_routes = generate_route_list(webui::App);
    let leptos_options = leptos_conf.leptos_options;

    let mut local_secrets = Secrets::load_or_init().unwrap_or_else(|e| {
        error!("Failed to create secrets file (.bento_secrets): {e}");
        std::process::exit(1);
    });
    let CookieKey(cookie_key) = local_secrets.cookie_key.clone();
    let app_state = AppState {
        leptos_options,
        auth_store: auth_store.clone(),
        cookie_key,
    };
    unsafe {
        // zero out [Secrets] struct so keys don't hang around in memory:
        // &raw mut local_secrets could also be used, but these kinds of pointer calls don't
        // force rust's aliasing rules. we have no reason to bypass those here, as this
        // should be the only reference, mutable or not.
        std::ptr::write_volatile(&mut local_secrets as *mut _, config::Secrets::default());
    }

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

    // Register initial auth account
    let app_conf = LOCAL_CONF.as_ref();
    let Admin { username, password } = &app_conf.admin;
    let pass_hash: PasswordHash = match PasswordHash::try_from(password.as_str()) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to create password hash for admin user: {e}");
            return;
        }
    };

    if let Ok(user) = auth_store.create_admin(username, pass_hash).await {
        info!(username = %user.username.0, id = %user.id.0, "Admin user created successfully");
    } else {
        warn!("Admin user already exists, skipping creation");
    }

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
        .with_state(app_state)
        .layer(ClientIpSource::ConnectInfo.into_extension());

    // Start the server
    let server_addr = app_conf.server.socket_addr();
    info!("Binding to address: {}", server_addr);
    let listener = match tokio::net::TcpListener::bind(&server_addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to address {server_addr}: {e}");
            return;
        }
    };
    info!("Server started successfully!");
    if let Err(e) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        error!("Server failed to run: {e}");
        return;
    }
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
