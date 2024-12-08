use std::time::Instant;

use axum::{body::Body, http::Request, response::IntoResponse, Extension};
use tower_sessions::Session;

use crate::{render::PDF, Context};

use super::{render_into, Auth, Page, AUTH, HEADER_STR};

const BROWSE_STR: &str = include_str!("../../templates/browse.typ");

static BROWSE: Page = Page::new(|| {
    let mut pdf = PDF::main(BROWSE_STR);
    pdf.write_source("header.typ", HEADER_STR);
    pdf
});


pub async fn browse(ctx: Extension<Context>, session: Session, request: Request<Body>) -> impl IntoResponse {
    let data = r#"
AUTHOR HERE
1
1 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
2
2 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
3
5 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
4
8 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
1
1 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
2
2 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
3
5 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
4
8 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
1
1 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
2
2 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
3
5 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
AUTHOR HERE
4
8 min ago
This is a post! There is text here that is rendering on your screen right now. It's really incredible, isn't it??
"#.trim();
    
    let data = format!("{data}{:?}", Instant::now());

    let auth = session.get::<Auth>(AUTH).await.ok().flatten().is_some();

    let mut page = BROWSE.lock();

    page.write("info.yml", format!("url: \"{}\"\nauth: {auth}", ctx.base_url));

    render_into(&mut page, data)
}