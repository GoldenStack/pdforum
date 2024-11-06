pub mod render;

use std::{sync::{Arc, Mutex, OnceLock}, time::Instant};

use axum_extra::extract::{cookie::Cookie, CookieJar};
use ecow::EcoVec;
use render::PDF;

use anyhow::Result;
use axum::{body::Body, extract::Path, http::{header::{CONTENT_TYPE, COOKIE, SET_COOKIE}, Request, StatusCode}, response::IntoResponse, routing::get, Router};
use typst::diag::SourceDiagnostic;

#[tokio::main]
async fn main() -> Result<()> {

    let error404 = error("404", "not found")
        .expect("Could not render fallback '404: not found' page. Aborting program");

    let error404 = (
        StatusCode::NOT_FOUND,
        [(CONTENT_TYPE, "application/pdf")],
        error404
    );

    let header = include_str!("../templates/header.typ");
    let keyboard = include_str!("../templates/keyboard.typ");

    let mut main = PDF::main(include_str!("../templates/browse.typ"));
    main.write("header.typ", header);

    let mut register = PDF::main(include_str!("../templates/register.typ"));
    register.write("header.typ", header);
    register.write("keyboard.typ", keyboard);

    let main = Arc::new(Mutex::new(main));
    let register = Arc::new(Mutex::new(register));

    let register2 = register.clone();

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

        let auth = false;

        let Ok(mut page) = main.lock() else {
            return error500().into_response();
        };

        page.write("info.yml", format!("url: \"http://localhost:1473\"\nauth: {auth}"));

        let Ok(buffer) = page.render_with_data(data) else {
            return error500().into_response();
        };

        (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/pdf")],
            buffer
        ).into_response()
    };

    let register_c = |Path(appended): Path<String>, request: Request<Body>| async move {
        let Ok(mut page) = register.lock() else {
            return error500().into_response();
        };
        
        let auth = true;

        page.write("info.yml", format!("url: \"http://localhost:1473\"\nauth: {auth}"));

        let cookies = CookieJar::from_headers(request.headers());

        let mut username = cookies.get("register").map(Cookie::value).unwrap_or("").to_owned();
        username.push_str(appended.as_str());

        let Ok(buffer) = page.render_with_data(username.as_bytes()) else {
            return error500().into_response();
        };

        (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/pdf"), (SET_COOKIE, &format!("register={username}; path=/register;"))],
            buffer
        ).into_response()
    };

    let register2 = || async move {
        let Ok(mut page) = register2.lock() else {
            return error500().into_response();
        };

        let auth = true;

        page.write("info.yml", format!("url: \"http://localhost:1473\"\nauth: {auth}"));

        let Ok(buffer) = page.render_with_data("") else {
            return error500().into_response();
        };

        (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/pdf"), (SET_COOKIE, &format!("register=; path=/register;"))],
            buffer
        ).into_response()
    };

    let app = Router::new()
        .route("/", get(homepage))
        .route("/register/:username", get(register_c))
        .route("/register", get(register2))
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

fn error500() -> impl IntoResponse {
    static ERROR: OnceLock<Vec<u8>> = OnceLock::new();
    let error500 = ERROR.get_or_init(|| {
        error("500", "internal server error")
            .expect("Could not render fallback '500: internal server error' page. Aborting program")
    });

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        [(CONTENT_TYPE, "application/pdf")],
        Vec::clone(error500)
    )
}

fn error(code: &str, message: &str) -> Result<Vec<u8>, EcoVec<SourceDiagnostic>> {
    let mut main = PDF::main(include_str!("../templates/error.typ"));
    main.render_with_data(format!("{code}\n{message}"))
}