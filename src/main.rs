pub mod render;

use std::{sync::Arc, time::Instant};

use render::PDF;

use anyhow::Result;
use axum::{body::Body, http::{header::{CONTENT_TYPE, SET_COOKIE}, Request}, routing::get, Router};

#[tokio::main]
async fn main() -> Result<()> {

    let main = PDF::main(include_str!("../templates/homepage.typ"));

    let lock = Arc::new(std::sync::Mutex::new(main));

    let homepage = |request: Request<Body>| async move {
        let mut world = lock.lock().unwrap();

        let data = r#"
AUTHOR HERE
1
1 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
2
2 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
3
5 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
4
8 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
"#.trim();

        world.write("data.txt", format!("{data}{:?}", Instant::now()).into_bytes());

        // TODO: Replace unwrapping with returning a prerendered 404 file.
        let buffer = world.render().unwrap();

        // println!("COOKIES: {:?}", request.headers().get(COOKIE));

        ([(CONTENT_TYPE, "application/pdf"), (SET_COOKIE, "cat=2;")], buffer)
    };

    let app = Router::new().route("/", get(homepage));

    // port 1473 is the port for my previous project plus one
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1473")
        .await
        .unwrap();

    // help me
    println!("unfortunately we are listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

