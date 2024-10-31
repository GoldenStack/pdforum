pub mod render;

use std::{sync::Arc, time::Instant};

use ecow::EcoVec;
use render::PDF;

use anyhow::Result;
use axum::{body::Body, http::{header::{CONTENT_TYPE, SET_COOKIE}, Request, StatusCode}, response::IntoResponse, routing::get, Router};
use typst::diag::SourceDiagnostic;

#[tokio::main]
async fn main() -> Result<()> {

    let error500 = error("500", "internal server error")
        .map(Arc::new)
        .expect("Could not render fallback '500: internal server error' page. Aborting program");

    let error404 = error("404", "not found")
        .unwrap_or_else(|_| Vec::clone(error500.as_ref()));

    let error404 = (
        StatusCode::NOT_FOUND,
        [(CONTENT_TYPE, "application/pdf")],
        error404
    );

    let main = PDF::main(include_str!("../templates/homepage.typ"));

    let lock = Arc::new(std::sync::Mutex::new(main));

    let homepage = |request: Request<Body>| async move {
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
        
        let data = format!("{data}{:?}", Instant::now());

        let buffer = lock.lock().ok()
            .and_then(|mut main| main.render_with_data(data).ok());

        if let Some(content) = buffer {
            (
                StatusCode::OK,
                [(CONTENT_TYPE, "application/pdf"), (SET_COOKIE, "cat=2;")],
                content
            ).into_response()
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(CONTENT_TYPE, "application/pdf")],
                Vec::clone(error500.as_ref())
            ).into_response()
        }

        // println!("COOKIES: {:?}", request.headers().get(COOKIE));
    };
    

    let app = Router::new()
        .route("/", get(homepage))
        .fallback(get(error404));

    // port 1473 is the port for my previous project plus one
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1473")
        .await
        .unwrap();

    // help me
    println!("unfortunately we are listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn error(code: &str, message: &str) -> Result<Vec<u8>, EcoVec<SourceDiagnostic>> {
    let mut main = PDF::main(include_str!("../templates/error.typ"));
    main.render_with_data(format!("{code}\n{message}"))
}