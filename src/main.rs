use std::sync::{Arc, Mutex};

use anyhow::Result;
use axum::{body::Body, http::{header::{CONTENT_TYPE, COOKIE, SET_COOKIE}, Request}, routing::get, Router};

use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};

#[tokio::main]
async fn main() -> Result<()> {

    let n: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    let homepage = |request: Request<Body>| async move {
        let mut pdf = Pdf::new();

        let catalog_id = Ref::new(1);
        let page_tree_id = Ref::new(2);
        let page_id = Ref::new(3);
        let font_id = Ref::new(4);
        let content_id = Ref::new(5);
        let font_name = Name(b"F1");

        pdf.catalog(catalog_id).pages(page_tree_id);

        pdf.pages(page_tree_id).kids([page_id]).count(1);

        let mut page = pdf.page(page_id);

        page.media_box(Rect::new(0.0, 0.0, 595.0, 842.0));
        page.parent(page_tree_id);
        page.contents(content_id);

        page.resources().fonts().pair(font_name, font_id);
        page.finish();
        
        pdf.type1_font(font_id).base_font(Name(b"Helvetica"));

        let mut content = Content::new();
        content.begin_text();
        content.set_font(font_name, 14.0);
        content.next_line(108.0, 734.0);

        let v = {
            let mut x = n.lock().unwrap();
            *x += 1;
            *x
        };
        let txt = format!("This is website load #{}", v);

        content.show(Str(txt.as_bytes()));
        content.end_text();
        pdf.stream(content_id, &content.finish());

        println!("COOKIES: {:?}", request.headers().get(COOKIE));

        ([(CONTENT_TYPE, "application/pdf"), (SET_COOKIE, "cat=2;")], pdf.finish())
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
