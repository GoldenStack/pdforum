use axum::{
    extract::Path,
    response::{IntoResponse, Redirect},
    Extension,
};
use tower_sessions::Session;

use crate::{database, render::PDF, Context};

use super::{render_into, Auth, Page, Return, AUTH, COMMON_STR, KEYBOARD_STR};

const PUBLISH_STR: &str = include_str!("../../templates/publish.typ");

static PUBLISH: Page = Page::new(|| {
    PDF::make(
        "publish.typ",
        [
            ("publish.typ", PUBLISH_STR),
            ("common.typ", COMMON_STR),
            ("keyboard.typ", KEYBOARD_STR),
        ],
    )
});

const PUBLISHING: &str = "publish";

pub async fn publish(
    ctx: Extension<Context>,
    session: Session,
    Path(suffix): Path<String>,
) -> Return {
    let mut publish = session
        .get::<String>(PUBLISHING)
        .await
        .map(Option::unwrap_or_default)?;

    let Ok(Some(auth)) = session.get::<Auth>(AUTH).await else {
        return Ok(Redirect::temporary("/login").into_response());
    };

    if suffix == "next" && !publish.is_empty() {
        let id = database::publish(&ctx.db, auth.id, publish.as_str()).await?;

        session.remove::<String>(PUBLISHING).await?;
        return Ok(format!("im the eeper... published post {id}").into_response());
    } else if suffix.len() == 1 {
        publish.push_str(suffix.as_str());
    } else {
        publish = String::default();
    }

    session.insert(PUBLISHING, &publish).await?;

    let mut page = PUBLISH.lock();

    let data = format!(
        r#"url: {}
auth: true
username: {}"#,
        ctx.base_url, auth.username
    );

    page.write("info.yml", data);

    render_into(&mut page, publish)
}

pub async fn publish_empty(ctx: Extension<Context>, session: Session) -> Return {
    publish(ctx, session, Path(String::new())).await
}
