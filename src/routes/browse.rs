use axum::Extension;
use tower_sessions::Session;

use crate::{database, render::PDF, Context};

use super::{render_into, Auth, Page, Post, Return, AUTH, COMMENT_SVG, COMMON_STR, FILLED_HEART_SVG, HEART_SVG};

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
    let auth = session.get::<Auth>(AUTH).await?;

    let posts = database::browse(&ctx.db, auth.map(|auth| auth.id)).await?;

    let data = posts.into_iter().map(|post| post.render())
        .fold(String::new(), |mut acc, str| {
            if !acc.is_empty() {
                acc.push_str("\u{0}");
            }
            acc.push_str(&str);
            acc
        });

    println!("{}", data.replace("\u{0}", "[NULL]"));

    let auth = session.get::<Auth>(AUTH).await.ok().flatten().is_some();

    let mut page = BROWSE.lock();

    page.write(
        "info.yml",
        format!("url: \"{}\"\nauth: {auth}", ctx.base_url),
    );

    render_into(&mut page, data)
}
