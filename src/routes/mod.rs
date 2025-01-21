pub mod browse;
pub mod creds;
pub mod post;
pub mod publish;
pub mod like;

use std::sync::{Arc, OnceLock};

use anyhow::anyhow;
use axum::{
    body::Body,
    http::{header::CONTENT_TYPE, HeaderName, Response, StatusCode},
    response::IntoResponse,
};
use ecow::EcoVec;
use parking_lot::{lock_api::MutexGuard, Mutex};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use typst::diag::SourceDiagnostic;

use crate::render::PDF;

pub type Return = Result<Response<Body>, Error>;

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

#[derive(Debug, Deserialize, Serialize)]
struct Auth {
    id: i32,
    username: String,
}

/// A post loaded from the database.
pub struct Post {
    pub id: i32,
    pub author: String,
    pub created_at: OffsetDateTime,
    pub content: String,
    pub likes: i32,
}

pub fn render_timestamp(time: OffsetDateTime) -> String {
    let duration = OffsetDateTime::now_utc() - time;
    let secs = duration.as_seconds_f64();

    const MINUTE: f64 = 1. * 60.;
    const HOUR: f64 = MINUTE * 60.;
    const DAY: f64 = HOUR * 24.;
    const MONTH: f64 = DAY * 30.;
    const YEAR: f64 = DAY * 365.;

    const PAIRS: [(&str, f64); 5] = [
        ("year", YEAR),
        ("month", MONTH),
        ("day", DAY),
        ("hour", HOUR),
        ("minute", MINUTE),
    ];

    for (name, unit) in PAIRS {
        if secs >= unit * 0.9 {
            let count = (secs / unit).round() as i64;

            return format!("{} {name}{} ago", count, if count == 1 { "" } else { "s" });
        }
    }

    return "just now".to_owned();
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

pub fn error404() -> (StatusCode, [(HeaderName, &'static str); 1], Vec<u8>) {
    let error404 = error("404", "not found")
        .expect("Could not render fallback '404: not found' page. Aborting program");

    (StatusCode::NOT_FOUND, [TYPE_PDF], error404)
}

pub fn error(code: &str, message: &str) -> Result<Vec<u8>, EcoVec<SourceDiagnostic>> {
    PDF::make("error.typ", [("error.typ", ERROR_STR)])
        .render_with_data(format!("{code}\n{message}"))
}

pub struct Error(anyhow::Error);

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        log::error!("500: {:?}", self.0);

        error500().into_response()
    }
}

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Error(value.into())
    }
}

pub fn render_into<I: Into<Vec<u8>>>(pdf: &mut PDF, data: I) -> Return {
    pdf.render_with_data(data)
        .map(|data| (StatusCode::OK, [TYPE_PDF], data).into_response())
        .map_err(|vec| anyhow!("{vec:?}").into())
}
