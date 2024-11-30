use std::{sync::{Arc, Mutex, OnceLock}, time::Instant};

use axum::{body::Body, extract::Path, http::{header::{CONTENT_TYPE, SET_COOKIE}, HeaderName, Request, StatusCode}, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use ecow::EcoVec;
use typst::diag::SourceDiagnostic;

use crate::URL;
use crate::render::PDF;

const ERROR: &str = include_str!("../templates/error.typ");
const BROWSE: &str = include_str!("../templates/browse.typ");
const HEADER: &str = include_str!("../templates/header.typ");
const KEYBOARD: &str = include_str!("../templates/keyboard.typ");
const REGISTER: &str = include_str!("../templates/register.typ");

const TYPE_PDF: (HeaderName, &str) = (CONTENT_TYPE, "application/pdf");

pub async fn browse(request: Request<Body>) -> impl IntoResponse {
    static PDF: OnceLock<Arc<Mutex<PDF>>> = OnceLock::new();
    let lock = PDF.get_or_init(|| {
        let mut browse = PDF::main(BROWSE);
        browse.write_source("header.typ", HEADER);
    
        Arc::new(Mutex::new(browse))
    });

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

    let Ok(mut page) = lock.lock() else {
        return error500().into_response();
    };

    page.write("info.yml", format!("url: \"{URL}\"\nauth: {auth}"));

    let Ok(buffer) = page.render_with_data(data) else {
        return error500().into_response();
    };

    (
        StatusCode::OK,
        [TYPE_PDF],
        buffer
    ).into_response()
}

pub async fn register(Path(suffix): Path<String>, request: Request<Body>) -> impl IntoResponse {
    static PDF: OnceLock<Arc<Mutex<PDF>>> = OnceLock::new();
    let lock = PDF.get_or_init(|| {
        let mut register = PDF::main(REGISTER);
        register.write_source("header.typ", HEADER);
        register.write_source("keyboard.typ", KEYBOARD);
    
        Arc::new(Mutex::new(register))
    });

    let Ok(mut page) = lock.lock() else {
        return error500().into_response();
    };

    fn register_empty(page: &mut PDF, request: Request<Body>) -> impl IntoResponse {
        let Ok(buffer) = page.render_with_data("") else {
            return error500().into_response();
        };
    
        return (
            StatusCode::OK,
            [TYPE_PDF, (SET_COOKIE, &format!("field=username; username=; path=/register; password=; path=/register;"))],
            buffer
        ).into_response();
    }
    
    let auth = false;

    let cookies = CookieJar::from_headers(request.headers());
    let field = cookies.get("field").map(Cookie::value).unwrap_or("");
    let username = cookies.get("username").map(Cookie::value).unwrap_or("");
    let password = cookies.get("password").map(Cookie::value).unwrap_or("");

    if suffix.len() == 0 { // Reset to start
        page.write("info.yml", format!("url: \"{URL}\"\nauth: {auth}\nfield: \"username\""));
        return register_empty(&mut page, request).into_response();
    }

    let username = format!("{username}{suffix}");

    page.write("info.yml", format!("url: \"{URL}\"\nauth: {auth}\nfield: \"username\""));

    let Ok(buffer) = page.render_with_data(username.as_bytes()) else {
        return error500().into_response();
    };

    (
        StatusCode::OK,
        [TYPE_PDF, (SET_COOKIE, &format!("username={username}; path=/register;"))],
        buffer
    ).into_response()
}

pub async fn register_empty(request: Request<Body>) -> impl IntoResponse {
    register(Path(String::new()), request).await.into_response()
}

pub fn error500() -> impl IntoResponse {
    static ERROR: OnceLock<Vec<u8>> = OnceLock::new();
    let error500 = ERROR.get_or_init(|| {
        error("500", "internal server error")
            .expect("Could not render fallback '500: internal server error' page. Aborting program")
    });

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        [TYPE_PDF],
        Vec::clone(error500)
    )
}

pub fn error400() -> impl IntoResponse {
    static ERROR: OnceLock<Vec<u8>> = OnceLock::new();
    let error400 = ERROR.get_or_init(|| {
        error("400", "bad request")
            .expect("Could not render fallback '400: bad request' page. Aborting program")
    });

    (
        StatusCode::BAD_REQUEST,
        [TYPE_PDF],
        Vec::clone(error400)
    )
}

pub fn error404() -> (StatusCode, [(HeaderName, &'static str); 1], Vec<u8>) {
    let error404 = error("404", "not found")
        .expect("Could not render fallback '404: not found' page. Aborting program");

    (
        StatusCode::NOT_FOUND,
        [TYPE_PDF],
        error404
    )
}

pub fn error(code: &str, message: &str) -> Result<Vec<u8>, EcoVec<SourceDiagnostic>> {
    PDF::main(ERROR).render_with_data(format!("{code}\n{message}"))
}