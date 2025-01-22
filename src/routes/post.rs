use axum::{extract::Path, response::IntoResponse, Extension};
use tower_sessions::Session;

use crate::{database, render::PDF, Context};

use super::{error404, render_into, Auth, Page, Return, AUTH, COMMENT_SVG, COMMON_STR, FILLED_HEART_SVG, HEART_SVG};

const POST_STR: &str = include_str!("../../templates/post.typ");

static POST: Page = Page::new(|| {
    PDF::make(
        "post.typ",
        [
            ("svg/heart.svg", HEART_SVG),
            ("svg/filled-heart.svg", FILLED_HEART_SVG),
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
    
    let auth = session.get::<Auth>(AUTH).await?;

    let Some(post) = database::retrieve_post(&ctx.db, post_id, auth.as_ref().map(|auth| auth.id)).await? else {
        return Ok(error404().into_response());
    };

    let data = format!(
        r#"url: {}
auth: {}"#,
        ctx.base_url,
        auth.is_some()
    );

    let mut page = POST.lock();

    page.write("info.yml", data);

    render_into(&mut page, post.render())
}
