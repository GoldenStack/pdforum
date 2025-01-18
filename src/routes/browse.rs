use axum::Extension;
use tower_sessions::Session;

use crate::{database, render::PDF, Context};

use super::{render_into, render_timestamp, Auth, Page, Return, AUTH, COMMON_STR};

const BROWSE_STR: &str = include_str!("../../templates/browse.typ");

static BROWSE: Page = Page::new(|| {
    PDF::make(
        "browse.typ",
        [("browse.typ", BROWSE_STR), ("common.typ", COMMON_STR)],
    )
});

pub async fn browse(ctx: Extension<Context>, session: Session) -> Return {
    let mut data = String::new();

    let values = database::browse(&ctx.db).await?;
    for value in values {
        data.push_str(
            format!(
                "{}\u{0}{}\u{0}{}\u{0}{}\u{0}",
                value.author,
                value.id,
                render_timestamp(value.created_at),
                value.content
            )
            .as_str(),
        );
    }

    // Chop off the last \u{0}
    let data = if data.ends_with("\u{0}") {
        &data[0..data.len() - 1]
    } else {
        &data
    };

    let auth = session.get::<Auth>(AUTH).await.ok().flatten().is_some();

    let mut page = BROWSE.lock();

    page.write(
        "info.yml",
        format!("url: \"{}\"\nauth: {auth}", ctx.base_url),
    );

    render_into(&mut page, data)
}
