use axum::{extract::{Path, RawQuery}, response::{IntoResponse, Redirect}, Extension};
use tower_sessions::Session;

use crate::{database, routes::{Auth, AUTH}, Context};

use super::Return;


pub async fn like(
    ctx: Extension<Context>,
    session: Session,
    Path(post_id): Path<String>,
    RawQuery(query): RawQuery
) -> Return {

    let Ok(Some(auth)) = session.get::<Auth>(AUTH).await else {
        return Ok(Redirect::temporary("/login").into_response());
    };

    let post_id: i32 = post_id.parse()?;

    let _ = database::like(&ctx.db, auth.id, post_id).await?;

    println!("{post_id}");
    
    Ok(Redirect::temporary(&format!("{}/{}", ctx.base_url, query.unwrap())).into_response())
}

pub async fn unlike(
    ctx: Extension<Context>,
    session: Session,
    Path(post_id): Path<String>,
    RawQuery(query): RawQuery
) -> Return {

    let Ok(Some(auth)) = session.get::<Auth>(AUTH).await else {
        return Ok(Redirect::temporary("/login").into_response());
    };

    let post_id: i32 = post_id.parse()?;

    let _ = database::unlike(&ctx.db, auth.id, post_id).await?;

    println!("{post_id}");
    
    Ok(Redirect::temporary(&format!("{}/{}", ctx.base_url, query.unwrap())).into_response())
}
