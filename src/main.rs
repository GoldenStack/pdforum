pub mod database;
pub mod render;
pub mod routes;

use std::{env, sync::Arc};

use anyhow::Result;
use axum::{routing::get, Extension, Router};
use log::{info, LevelFilter};
use sqlx::PgPool;
use tower_sessions::{
    cookie::{time::Duration, SameSite},
    Expiry, MemoryStore, SessionManagerLayer,
};

#[derive(Clone, Debug)]
pub struct Context {
    pub base_url: Arc<String>,
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    env_logger::builder()
        .filter_module("tracing::span", LevelFilter::Warn)
        .filter_module("tower_sessions", LevelFilter::Warn)
        .filter_module("tower_sessions_core", LevelFilter::Warn)
        .try_init()?;

    let base_url = env::var("BASE_URL")?;

    let ctx = Context {
        base_url: Arc::new(base_url),
        db: database::open_connection().await?,
    };

    let postgres_layer = Extension(ctx);

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::hours(6)));

    let app = Router::new()
        .route("/", get(routes::browse::browse))
        .route("/register/:username", get(routes::creds::register))
        .route("/register", get(routes::creds::register_empty))
        .route("/login/:username", get(routes::creds::login))
        .route("/login", get(routes::creds::login_empty))
        .route("/publish/:suffix", get(routes::publish::publish))
        .route("/publish", get(routes::publish::publish_empty))
        .route("/post/:id", get(routes::post::post))
        .route("/like/:id", get(routes::like::like))
        .route("/unlike/:id", get(routes::like::unlike))
        .fallback(get(routes::error404()))
        .layer(session_layer)
        .layer(postgres_layer);

    // port 1473 is the port for my previous project plus one
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1473")
        .await
        .unwrap();

    // help me
    info!(
        "unfortunately we are listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
