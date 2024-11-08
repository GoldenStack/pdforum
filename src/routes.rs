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
        browse.write("header.typ", HEADER);
    
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
        register.write("header.typ", HEADER);
        register.write("keyboard.typ", KEYBOARD);
    
        Arc::new(Mutex::new(register))
    });

    let Ok(mut page) = lock.lock() else {
        return error500().into_response();
    };
    
    let auth = true;

    page.write("info.yml", format!("url: \"{URL}\"\nauth: {auth}"));

    if suffix.len() == 0 {
        let Ok(buffer) = page.render_with_data("") else {
            return error500().into_response();
        };
    
        return (
            StatusCode::OK,
            [TYPE_PDF, (SET_COOKIE, &format!("register=; path=/register;"))],
            buffer
        ).into_response();
    }

    let cookies = CookieJar::from_headers(request.headers());

    let prefix = cookies.get("register").map(Cookie::value).unwrap_or("");
    let username = format!("{prefix}{suffix}");

    let Ok(buffer) = page.render_with_data(username.as_bytes()) else {
        return error500().into_response();
    };

    (
        StatusCode::OK,
        [TYPE_PDF, (SET_COOKIE, &format!("register={username}; path=/register;"))],
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