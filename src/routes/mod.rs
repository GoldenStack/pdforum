pub mod browse;
pub mod creds;
pub mod publish;

use std::sync::{Arc, OnceLock};

use axum::{
    body::Body,
    http::{header::CONTENT_TYPE, HeaderName, Response, StatusCode},
    response::IntoResponse,
};
use ecow::EcoVec;
use parking_lot::{lock_api::MutexGuard, Mutex};
use serde::{Deserialize, Serialize};
use typst::diag::SourceDiagnostic;

use crate::render::PDF;

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
        self.lock
            .get_or_init(|| {
                let pdf = (self.function)();

                Arc::new(Mutex::new(pdf))
            })
            .lock()
    }
}

pub(crate) const ERROR_STR: &str = include_str!("../../templates/error.typ");
pub(crate) const COMMON_STR: &str = include_str!("../../templates/common.typ");
pub(crate) const KEYBOARD_STR: &str = include_str!("../../templates/keyboard.typ");

const TYPE_PDF: (HeaderName, &str) = (CONTENT_TYPE, "application/pdf");

pub const AUTH: &str = "auth";

#[derive(Debug, Default, Deserialize, Serialize)]
struct Auth {
    username: String,
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
        Vec::clone(error500),
    )
}

pub fn error400() -> impl IntoResponse {
    static ERROR: OnceLock<Vec<u8>> = OnceLock::new();
    let error400 = ERROR.get_or_init(|| {
        error("400", "bad request")
            .expect("Could not render fallback '400: bad request' page. Aborting program")
    });

    (StatusCode::BAD_REQUEST, [TYPE_PDF], Vec::clone(error400))
}

pub fn error404() -> (StatusCode, [(HeaderName, &'static str); 1], Vec<u8>) {
    let error404 = error("404", "not found")
        .expect("Could not render fallback '404: not found' page. Aborting program");

    (StatusCode::NOT_FOUND, [TYPE_PDF], error404)
}

pub fn error(code: &str, message: &str) -> Result<Vec<u8>, EcoVec<SourceDiagnostic>> {
    PDF::make("error.typ", [("error.typ", ERROR_STR)])
        .render_with_data(format!("{code}\n{message}"))
}

pub fn render_into<I: Into<Vec<u8>>>(pdf: &mut PDF, data: I) -> Response<Body> {
    match pdf.render_with_data(data) {
        Ok(buffer) => (StatusCode::OK, [TYPE_PDF], buffer).into_response(),
        Err(err) => {
            println!("{err:?}");
            error500().into_response()
        } 
    }
}
