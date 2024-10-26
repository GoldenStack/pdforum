use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc, time::Instant};

use chrono::{DateTime, Datelike, FixedOffset, Local, Utc};
use comemo::{track, Track, Validate};
use parking_lot::Mutex;
use typst::{diag::{FileError, FileResult, SourceResult}, engine::{Engine, Route, Sink, Traced}, eval::eval, foundations::{Bytes, Datetime, Smart, StyleChain}, introspection::Introspector, layout::layout_document, model::Document, syntax::{FileId, Source, VirtualPath}, text::{Font, FontBook}, utils::LazyHash, Library, World};
use typst_kit::fonts::{FontSlot, Fonts};
use typst_pdf::{PdfOptions, PdfStandards};

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

        if rootless == Path::new("example.typ") {
            let s = include_str!("../example.typ");

            return Ok(s.as_bytes().to_owned());
        }

        println!("NEW FILE: {:?}, \t{:?}, \t{:?}", id.vpath(), id.vpath().as_rootless_path(), id.package());
        Err(FileError::NotFound(id.vpath().as_rootless_path().to_path_buf()))
    };

    let world = FWorld::new("example.typ".into(), read);

    let lock = Arc::new(std::sync::Mutex::new(world));

    let homepage = |request: Request<Body>| async move {
        let mut world = lock.lock().unwrap();

        let start = Instant::now();
        let document: typst::model::Document = compile(&mut world).unwrap();
        let end = Instant::now();
        println!("[CMP] {:?}", end - start);


        let text = world.slots.get_mut().keys()
        .filter(|key| key.vpath().as_rootless_path() == Path::new("data.txt"))
        .next().unwrap().clone();
        world.slot(text.clone(), |s| s.reset());
    
        let options = PdfOptions {
            ident: Smart::Auto,
            timestamp: world.today(None),
            page_ranges: None,
            standards: PdfStandards::default()
        };

        let start = Instant::now();
        let buffer = typst_pdf::pdf(&document, &options).unwrap();
        let end: Instant = Instant::now();

        println!("[RNDR] {:?}", end - start);

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

/// Holds the processed data for a file ID.
///
/// Both fields can be populated if the file is both imported and read().
#[derive(Debug)]
struct FileSlot {
    /// The slot's file id.
    id: FileId,
    /// The lazily loaded and incrementally updated source file.
    source: SlotCell<Source>,
    /// The lazily loaded raw byte buffer.
    file: SlotCell<Bytes>,
}

impl FileSlot {
    /// Create a new file slot.
    fn new(id: FileId) -> Self {
        Self { id, file: SlotCell::new(), source: SlotCell::new() }
    }

    /// Marks the file as not yet accessed in preparation of the next
    /// compilation.
    fn reset(&mut self) {
        self.source.reset();
        self.file.reset();
    }

    /// Retrieve the source for this file.
    fn source(&mut self, read: &fn(FileId) -> FileResult<Vec<u8>>) -> FileResult<Source> {
        self.source.get_or_init(
            || read(self.id),
            |data, prev| {
                let text = decode_utf8(&data)?;
                if let Some(mut prev) = prev {
                    prev.replace(text);
                    Ok(prev)
                } else {
                    Ok(Source::new(self.id, text.into()))
                }
            },
        )
    }

    /// Retrieve the file's bytes.
    fn file(&mut self, read: &fn(FileId) -> FileResult<Vec<u8>>) -> FileResult<Bytes> {
        self.file.get_or_init(
            || read(self.id),
            |data, _| Ok(data.into()),
        )
    }
}

/// Decode UTF-8 with an optional BOM.
fn decode_utf8(buf: &[u8]) -> FileResult<&str> {
    // Remove UTF-8 BOM.
    Ok(std::str::from_utf8(buf.strip_prefix(b"\xef\xbb\xbf").unwrap_or(buf))?)
}

/// Lazily processes data for a file.
#[derive(Debug)]
struct SlotCell<T> {
    /// The processed data.
    data: Option<FileResult<T>>,
    /// A hash of the raw file contents / access error.
    fingerprint: u128,
    /// Whether the slot has been accessed in the current compilation.
    accessed: bool,
}

impl<T: Clone> SlotCell<T> {
    /// Creates a new, empty cell.
    fn new() -> Self {
        Self { data: None, fingerprint: 0, accessed: false }
    }

    /// Marks the cell as not yet accessed in preparation of the next
    /// compilation.
    fn reset(&mut self) {
        self.accessed = false;
    }

    /// Gets the contents of the cell or initialize them.
    fn get_or_init(
        &mut self,
        load: impl FnOnce() -> FileResult<Vec<u8>>,
        f: impl FnOnce(Vec<u8>, Option<T>) -> FileResult<T>,
    ) -> FileResult<T> {
        // If we accessed the file already in this compilation, retrieve it.
        if std::mem::replace(&mut self.accessed, true) {
            if let Some(data) = &self.data {
                return data.clone();
            }
        }

        // Read and hash the file.
        let result = load();
        let fingerprint = typst::utils::hash128(&result);

        // If the file contents didn't change, yield the old processed data.
        if std::mem::replace(&mut self.fingerprint, fingerprint) == fingerprint {
            if let Some(data) = &self.data {
                return data.clone();
            }
        }

        let prev = self.data.take().and_then(Result::ok);
        let value = result.and_then(|data| f(data, prev));
        self.data = Some(value.clone());

        value
    }
}

pub struct FWorld {
    /// The input path.
    main: FileId,
    /// Typst's standard library.
    library: LazyHash<Library>,
    /// Metadata about discovered fonts.
    book: LazyHash<FontBook>,
    /// Locations of and storage for lazily loaded fonts.
    fonts: Vec<FontSlot>,
    /// Maps file ids to source files and buffers.
    pub slots: Mutex<HashMap<FileId, FileSlot>>,
    /// The current datetime if requested.
    now: DateTime<Utc>,
    /// Function for reading files from the filesystem.
    read: fn(FileId) -> FileResult<Vec<u8>>,
}

impl FWorld {
    pub fn new(path: PathBuf, read: fn(FileId) -> FileResult<Vec<u8>>) -> Self {
        let root: PathBuf = path.parent().unwrap_or(Path::new(".")).into();

        let main_path = VirtualPath::within_root(&path, &root).unwrap();

        let main = FileId::new(None, main_path);

        let fonts = Fonts::searcher()
            .include_system_fonts(true)
            .search();

        Self {
            main,
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            slots: Mutex::new(HashMap::new()),
            now: Utc::now(),
            read,
        }
    }

    /// Reset the compilation state in preparation of a new compilation.
    pub fn reset(&mut self) {
        for slot in self.slots.get_mut().values_mut() {
            slot.reset();
        }
        self.now = Utc::now();
    }

    /// Access the canonical slot for the given file id.
    fn slot<F, T>(&self, id: FileId, f: F) -> T
    where
        F: FnOnce(&mut FileSlot) -> T,
    {
        let mut map = self.slots.lock();
        f(map.entry(id).or_insert_with(|| FileSlot::new(id)))
    }

}

#[track]
impl World for FWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.slot(id, |slot| slot.source(&self.read))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.slot(id, |slot| slot.file(&self.read))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts[index].get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        // The time with the specified UTC offset, or within the local time zone.
        let with_offset = match offset {
            None => self.now.with_timezone(&Local).fixed_offset(),
            Some(hours) => {
                let seconds = i32::try_from(hours).ok()?.checked_mul(3600)?;
                self.now.with_timezone(&FixedOffset::east_opt(seconds)?)
            }
        };

        Datetime::from_ymd(
            with_offset.year(),
            with_offset.month().try_into().ok()?,
            with_offset.day().try_into().ok()?,
        )
    }
}

fn compile(world: &mut FWorld) -> SourceResult<Document> {

    let w: &dyn World = world;

    let mut sink = Sink::new();

    let world = world.track();

    let traced = Traced::default();
    let traced = traced.track();
    let library = world.library();
    let styles = StyleChain::new(&library.styles);

    // Fetch the main source file once.
    let main = world.main();
    let main = world.source(main).unwrap();

    // First evaluate the main source file into a module.
    let content = eval(
        w.track(),
        traced,
        sink.track_mut(),
        Route::default().track(),
        &main,
    )?
    .content();

    let mut iter = 0;
    let mut subsink;
    let mut document = Document::default();

    // Relayout until all introspections stabilize.
    // If that doesn't happen within five attempts, we give up.
    loop {
        subsink = Sink::new();

        let constraint = <Introspector as Validate>::Constraint::new();
        let mut engine = Engine {
            world: w.track(),
            introspector: document.introspector.track_with(&constraint),
            traced,
            sink: subsink.track_mut(),
            route: Route::default(),
        };

        document = layout_document(&mut engine, &content, styles)?;
        iter += 1;

        if iter > 5 || document.introspector.validate(&constraint) {
            break;
        }
    }

    sink.extend_from_sink(subsink);

    // Promote delayed errors.
    let delayed = sink.delayed();
    if !delayed.is_empty() {
        return Err(delayed);
    }

    Ok(document)
}