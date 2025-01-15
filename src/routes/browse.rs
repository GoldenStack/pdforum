use axum::{response::IntoResponse, Extension};
use sqlx::types::time::OffsetDateTime;
use time::Duration;
use tower_sessions::Session;

use crate::{database, render::PDF, Context};

use super::{render_into, Auth, Page, AUTH, COMMON_STR};

const BROWSE_STR: &str = include_str!("../../templates/browse.typ");

static BROWSE: Page = Page::new(|| {
    PDF::make(
        "browse.typ",
        [("browse.typ", BROWSE_STR), ("common.typ", COMMON_STR)],
    )
});

pub async fn browse(ctx: Extension<Context>, session: Session) -> impl IntoResponse {
    let mut data = String::new();

    match database::browse(&ctx.db).await {
        Ok(values) => {
            for value in values {
                let time_ago = format_duration_ago(OffsetDateTime::now_utc() - value.created_at);

                data.push_str(format!("{}\u{0}{}\u{0}{}\u{0}{}\u{0}", value.author, value.id, time_ago, value.content).as_str());
            }
        },
        Err(err) => todo!(),
    }
 
    // Chop off the last \u{0}
    let data = if data.ends_with("\u{0}") {
        &data[0..data.len()-1]
    } else { &data };

    let auth = session.get::<Auth>(AUTH).await.ok().flatten().is_some();

    let mut page = BROWSE.lock();

    page.write(
        "info.yml",
        format!("url: \"{}\"\nauth: {auth}", ctx.base_url),
    );

    render_into(&mut page, data)
}

fn format_duration_ago(dur: Duration) -> String {
    let secs = dur.as_seconds_f64();

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
        ("minute", MINUTE)
    ];

    for (name, unit) in PAIRS {
        if secs >= unit * 0.9 {
            let count = (secs / unit).round() as i64;

            return format!("{} {name}{} ago", count, if count == 1 { "" } else { "s" });
        }
    }

    return "just now".to_owned();
}