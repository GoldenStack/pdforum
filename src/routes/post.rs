use axum::{extract::Path, response::IntoResponse, Extension};
use tower_sessions::Session;

use crate::{database, render::PDF, Context};

use super::{error404, render_into, render_timestamp, Auth, Page, Return, AUTH, COMMON_STR};

const POST_STR: &str = include_str!("../../templates/post.typ");

const HEART_SVG: &str = include_str!("../../templates/svg/heart.svg");
const COMMENT_SVG: &str = include_str!("../../templates/svg/comment.svg");

static POST: Page = Page::new(|| {
    PDF::make(
        "post.typ",
        [
            ("svg/heart.svg", HEART_SVG),
            ("svg/comment.svg", COMMENT_SVG),
            ("post.typ", POST_STR),
            ("common.typ", COMMON_STR),
        ],
    )
});

pub async fn post(
    ctx: Extension<Context>,
    session: Session,
    Path(post_id): Path<String>,
) -> Return {
    let post_id: i32 = post_id.parse()?;

    let Some(post) = database::retrieve_post(&ctx.db, post_id).await? else {
        return Ok(error404().into_response());
    };

    let auth = session.get::<Auth>(AUTH).await?;

    let mut page = POST.lock();

    let data = format!(
        r#"url: {}
auth: {}"#,
        ctx.base_url,
        auth.is_some()
    );

    page.write("info.yml", data);

    let post_data = format!(
        "{}\u{0}{}\u{0}{}\u{0}{}\u{0}{}",
        post.author,
        10,
        25,
        render_timestamp(post.created_at),
        post.content
    );

    render_into(&mut page, post_data)
}
