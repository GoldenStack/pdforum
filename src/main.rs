use std::{collections::HashMap, path::{Path, PathBuf}, time::Instant};

use chrono::{DateTime, Datelike, FixedOffset, Local, Utc};
use parking_lot::Mutex;
use typst::{diag::{FileError, FileResult}, foundations::{Bytes, Datetime, Smart}, syntax::{FileId, Source, VirtualPath}, text::{Font, FontBook}, utils::LazyHash, Library, World};
use typst_kit::{download::{Downloader, ProgressSink}, fonts::{FontSlot, Fonts}, package::PackageStorage};
use typst_pdf::{PdfOptions, PdfStandards};

fn main() {
    let start = Instant::now();

    let mut world = FWorld::new("example.typ".into());

    let end = Instant::now();
    println!("[WRLD] {:?}", end - start);
    let start = Instant::now();


    let document: typst::model::Document = typst::compile(&mut world).output.unwrap();
    world.slot(world.main, |s| s.reset());

    let end = Instant::now();
    println!("[COMP] {:?}", end - start);
    let start = Instant::now();

    let document: typst::model::Document = typst::compile(&mut world).output.unwrap();
    world.slot(world.main, |s| s.reset());

    let end = Instant::now();
    println!("[CMP2] {:?}", end - start);
    let start = Instant::now();

    let options = PdfOptions {
        ident: Smart::Auto,
        timestamp: world.today(None),
        page_ranges: None,
        standards: PdfStandards::default()
    };
    let buffer = typst_pdf::pdf(&document, &options).unwrap();
    let end = Instant::now();
    println!("[RNDR] {:?}", end - start);
    let start = Instant::now();

    std::fs::write(PathBuf::from("example.pdf"), &buffer).unwrap();

    let end = Instant::now();
    println!("[SAVE] {:?}", end - start);
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
    fn source(&mut self, read: &Read) -> FileResult<Source> {
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
    fn file(&mut self, read: &Read) -> FileResult<Bytes> {
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
    slots: Mutex<HashMap<FileId, FileSlot>>,
    /// The current datetime if requested.
    now: DateTime<Utc>,
    /// Function for reading files from the filesystem.
    read: fn(FileId) -> FileResult<Vec<u8>>,
}

pub type Read = fn(FileId) -> FileResult<Vec<u8>>;

impl FWorld {
    pub fn new(path: PathBuf) -> Self {
        let root: PathBuf = path.parent().unwrap_or(Path::new(".")).into();

        let main_path = VirtualPath::within_root(&path, &root).unwrap();

        let main = FileId::new(None, main_path);

        let fonts = Fonts::searcher()
            .include_system_fonts(true)
            .search();

        let read: Read = |id| {
            let rootless = id.vpath().as_rootless_path();

            if rootless == Path::new("yagenda.typ") {
                Ok(include_bytes!("../yagenda.typ").to_vec())
            } else if rootless == Path::new("example.typ") {
                let s = include_str!("../example.typ");

                let replace = format!("{:?}\n", Instant::now());
                println!("R {}", replace);

                Ok(s.replace("{{ PDForum Template }}", replace.as_str()).as_bytes().to_vec())
            } else {
                println!("NEW FILE: {:?}, \t{:?}, \t{:?}", id.vpath(), id.vpath().as_rootless_path(), id.package());
                Err(FileError::NotFound(id.vpath().as_rootless_path().to_path_buf()))
            }            
        };

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