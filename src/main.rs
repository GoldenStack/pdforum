pub mod render;

use std::{path::Path, sync::Arc, time::Instant};

use render::PDF;
use typst::{diag::{FileError, FileResult}, syntax::FileId};

use anyhow::Result;
use axum::{body::Body, http::{header::{CONTENT_TYPE, SET_COOKIE}, Request}, routing::get, Router};

#[tokio::main]
async fn main() -> Result<()> {

    let read: fn(FileId) -> FileResult<Vec<u8>> = |id| {
        let rootless = id.vpath().as_rootless_path();

        if rootless == Path::new("data.txt") {
            let str = r#"
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
            return Ok(format!("{str}{:?}", Instant::now()).into_bytes());
        }

        if rootless == Path::new("homepage.typ") {
            let s = include_str!("../templates/homepage.typ");

            return Ok(s.as_bytes().to_owned());
        }

        println!("NEW FILE: {:?}, \t{:?}, \t{:?}", id.vpath(), id.vpath().as_rootless_path(), id.package());
        Err(FileError::NotFound(id.vpath().as_rootless_path().to_path_buf()))
    };

    let main = PDF::new("homepage.typ", read);

    let lock = Arc::new(std::sync::Mutex::new(main));

    let homepage = |request: Request<Body>| async move {
        let mut world = lock.lock().unwrap();

        // TODO: Replace unwrapping with returning a prerendered 404 file.
        let buffer = world.render().unwrap();

        // println!("COOKIES: {:?}", request.headers().get(COOKIE));

        ([(CONTENT_TYPE, "application/pdf"), (SET_COOKIE, "cat=2;")], buffer)
    };

    let app = Router::new().route("/", get(homepage));

    // port 1473 is the port for my previous project plus one
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1473")
        .await
        .unwrap();

    // help me
    println!("unfortunately we are listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

