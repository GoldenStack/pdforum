
use axum::{extract::Path, response::{IntoResponse, Redirect}, Extension};
use tower_sessions::Session;

use crate::{render::PDF, Context};

use super::{error500, render_into, Auth, Page, AUTH, COMMON_STR, KEYBOARD_STR};

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
) -> impl IntoResponse {
    let Ok(mut publish) = session
        .get::<String>(PUBLISHING)
        .await
        .map(Option::unwrap_or_default)
    else {
        return error500().into_response();
    };

    if suffix == "next" {
        todo!()
    } else if suffix.len() == 1 {
        publish.push_str(suffix.as_str());
    } else {
        publish = String::default();
    }

    session.insert(PUBLISHING, &publish).await.unwrap();

    let Ok(Some(auth)) = session
        .get::<Auth>(AUTH)
        .await
    else {
        return Redirect::temporary("/login").into_response();
    };

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



pub async fn publish_empty(ctx: Extension<Context>, session: Session) -> impl IntoResponse {
    publish(ctx, session, Path(String::new())).await
}