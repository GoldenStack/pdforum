pub mod render;
pub mod routes;
pub mod database;

use std::sync::Arc;

use anyhow::Result;
use axum::{response::Redirect, routing::get, Extension, Router};
use sqlx::postgres::PgPoolOptions;
use tower_sessions::{cookie::{time::Duration, SameSite}, Expiry, MemoryStore, SessionManagerLayer};

pub const URL: &str = "http://localhost:1473";

#[tokio::main]
async fn main() -> Result<()> {

    let db = PgPoolOptions::new()
        .max_connections(16)
        .connect(&"REDACTED") // NOT leaking this sorry
        .await?;

    let postgres_layer = Extension(Arc::new(db));

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::hours(6)));

    let app = Router::new()
        .route("/", get(routes::browse))
        .route("/register/:username", get(routes::register))
        .route("/register", get(routes::register_empty))
        .route("/register/", get(Redirect::permanent("/register")))
        .fallback(get(routes::error404()))
        .layer(session_layer)
        .layer(postgres_layer);

    // port 1473 is the port for my previous project plus one
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1473")
        .await
        .unwrap();

    // help me
    println!("unfortunately we are listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
