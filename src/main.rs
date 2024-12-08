pub mod database;
pub mod render;
pub mod routes;

use std::{env, sync::Arc};

use anyhow::Result;
use axum::{routing::get, Extension, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};
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

    let base_url = env::var("BASE_URL")?;
    let database_url = env::var("DATABASE_URL")?;

    let db = PgPoolOptions::new()
        .max_connections(16)
        .connect(&database_url)
        .await?;

    let ctx = Context {
        base_url: Arc::new(base_url),
        db,
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
        .fallback(get(routes::error404()))
        .layer(session_layer)
        .layer(postgres_layer);

    // port 1473 is the port for my previous project plus one
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1473")
        .await
        .unwrap();

    // help me
    println!(
        "unfortunately we are listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
