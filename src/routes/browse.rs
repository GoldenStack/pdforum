use axum::Extension;
use tower_sessions::Session;

use crate::{database, render::PDF, Context};

use super::{render_into, render_timestamp, Auth, Page, Return, AUTH, COMMENT_SVG, COMMON_STR, FILLED_HEART_SVG, HEART_SVG};

const BROWSE_STR: &str = include_str!("../../templates/browse.typ");

static BROWSE: Page = Page::new(|| {
    PDF::make(
        "browse.typ",
        [
            ("svg/heart.svg", HEART_SVG),
            ("svg/filled-heart.svg", FILLED_HEART_SVG),
            ("svg/comment.svg", COMMENT_SVG),
            ("browse.typ", BROWSE_STR),
            ("common.typ", COMMON_STR)
        ],
    )
});

pub async fn browse(ctx: Extension<Context>, session: Session) -> Return {
    let mut data = String::new();

    let auth = session.get::<Auth>(AUTH).await?;

    if let Some(auth) = auth {
        for (post, liked) in database::browse_as_user(&ctx.db, auth.id).await? {
            data.push_str(
                format!(
                    "{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}\u{0}",
                    post.id,
                    post.author,
                    post.likes,
                    0,
                    liked,
                    render_timestamp(post.created_at),
                    post.content
                )
                .as_str(),
            );
        }
    } else {
        for post in database::browse(&ctx.db).await? {
            data.push_str(
                format!(
                    "{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}\u{0}",
                    post.id,
                    post.author,
                    post.likes,
                    0,
                    false,
                    render_timestamp(post.created_at),
                    post.content
                )
                .as_str(),
            );
        }
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
