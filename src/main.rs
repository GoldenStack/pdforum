pub mod render;
pub mod routes;

use anyhow::Result;
use axum::{response::Redirect, routing::get, Router};

pub const URL: &str = "http://localhost:1473";

#[tokio::main]
async fn main() -> Result<()> {

    let app = Router::new()
        .route("/", get(routes::browse))
        .route("/register/:username", get(routes::register))
        .route("/register", get(routes::register_empty))
        .route("/register/", get(Redirect::permanent("/register")))
        .fallback(get(routes::error404()));

    // port 1473 is the port for my previous project plus one
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1473")
        .await
        .unwrap();

    // help me
    println!("unfortunately we are listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
