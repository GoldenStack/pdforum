use std::{sync::{Arc, OnceLock}, time::Instant};

use axum::{body::Body, extract::Path, http::{header::CONTENT_TYPE, HeaderName, Request, Response, StatusCode}, response::{IntoResponse, Redirect}, Extension};
use ecow::EcoVec;
use parking_lot::{lock_api::MutexGuard, Mutex};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_sessions::Session;
use typst::diag::SourceDiagnostic;

use crate::{database, URL};
use crate::render::PDF;

const ERROR_STR: &str = include_str!("../templates/error.typ");
const BROWSE_STR: &str = include_str!("../templates/browse.typ");
const HEADER_STR: &str = include_str!("../templates/header.typ");
const KEYBOARD_STR: &str = include_str!("../templates/keyboard.typ");
const REGISTER_STR: &str = include_str!("../templates/register.typ");

const TYPE_PDF: (HeaderName, &str) = (CONTENT_TYPE, "application/pdf");

/// A static PDF, stored with an initializer function.
/// 
/// This is a simple wrapper over a OnceLock that moves initialization
/// and locking to a central location.
pub struct Page {
    lock: OnceLock<Arc<Mutex<PDF>>>,
    function: fn() -> PDF,
}

impl Page {
    pub const fn new(function: fn() -> PDF) -> Self {
        Page {
            lock: OnceLock::new(),
            function,
        }
    }

    /// Initializes the page PDF if it has not been initialized yet,
    /// and then blocks to acquire the mutex.
    pub fn lock(&self) -> MutexGuard<'_, parking_lot::RawMutex, PDF> {
        self.lock.get_or_init(|| {
            let pdf = (self.function)();

            Arc::new(Mutex::new(pdf))
        }).lock()
    }
}

static BROWSE: Page = Page::new(|| {
    let mut pdf = PDF::main(BROWSE_STR);
    pdf.write_source("header.typ", HEADER_STR);
    pdf
});

static REGISTER: Page = Page::new(|| {
    let mut pdf = PDF::main(REGISTER_STR);
    pdf.write_source("header.typ", HEADER_STR);
    pdf.write_source("keyboard.typ", KEYBOARD_STR);

    pdf
});


const REGISTRATION: &str = "register";

#[derive(Debug, Default, Deserialize, Serialize)]
struct Register {
    field: RegisterField,
    username: String,
    password: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
enum RegisterField {
    #[default]
    Username,
    Password
}

const AUTH: &str = "auth";

#[derive(Debug, Default, Deserialize, Serialize)]
struct Auth {
    username: String,
}

pub async fn browse(session: Session, request: Request<Body>) -> impl IntoResponse {
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

    let auth = session.get::<Auth>(AUTH).await.ok().flatten().is_some();

    let mut page = BROWSE.lock();

    page.write("info.yml", format!("url: \"{URL}\"\nauth: {auth}"));

    render_into(&mut page, data)
}

pub async fn register(pg: Extension<Arc<PgPool>>, session: Session, Path(suffix): Path<String>, request: Request<Body>) -> impl IntoResponse {
    let Ok(mut register) = session.get::<Register>(REGISTRATION).await.map(Option::unwrap_or_default) else {
        return error500().into_response();
    };

    match suffix.as_str() {
        "" => register = Register::default(),
        "next" => {
            match register.field {
                RegisterField::Username => register.field = RegisterField::Password,
                RegisterField::Password => {
                    match database::register(&pg, &register.username, &register.password).await {
                        Ok(true) => {
                            let Ok(register) = session.remove::<Register>(REGISTRATION).await.map(Option::unwrap_or_default) else {
                                return error500().into_response();
                            };
                        
                            session.insert(AUTH, Auth { username: register.username }).await.unwrap();
                            return Redirect::temporary("/").into_response();
                        }
                        Ok(false) | Err(_) => {
                            todo!()
                        }
                    }
                },
            }
        }
        c if c.len() == 1 => {
            match register.field {
                RegisterField::Username => register.username = format!("{}{}", register.username, c),
                RegisterField::Password => register.password = format!("{}{}", register.password, c),
            }
        }
        _ => return error400().into_response(),
    }

    session.insert(REGISTRATION, &register).await.unwrap();

    let auth = session.get::<Auth>(AUTH).await.ok().flatten().is_some();

    let value = match register.field {
        RegisterField::Username => ("username", register.username),
        RegisterField::Password => ("password", register.password),
    };

    let mut page = REGISTER.lock();

    page.write("info.yml", format!("url: \"{URL}\"\nauth: {auth}\nfield: \"{}\"", value.0));

    render_into(&mut page, value.1)
}

pub async fn register_empty(postgres: Extension<Arc<PgPool>>, session: Session, request: Request<Body>) -> impl IntoResponse {
    register(postgres, session, Path(String::new()), request).await.into_response()
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
    PDF::main(ERROR_STR).render_with_data(format!("{code}\n{message}"))
}

pub fn render_into<I: Into<Vec<u8>>>(pdf: &mut PDF, data: I) -> Response<Body> {
    match pdf.render_with_data(data) {
        Ok(buffer) => (
            StatusCode::OK,
            [TYPE_PDF],
            buffer
        ).into_response(),
        Err(_) => error500().into_response()
    }
}