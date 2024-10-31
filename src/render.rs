
use std::{collections::HashMap, path::{Path, PathBuf}};

use chrono::{DateTime, Datelike, FixedOffset, Local, Utc};
use comemo::track;
use ecow::EcoVec;
use parking_lot::Mutex;
use typst::{compile, diag::{FileError, FileResult, SourceDiagnostic}, foundations::{Bytes, Datetime, Smart}, syntax::{FileId, Source, VirtualPath}, text::{Font, FontBook}, utils::LazyHash, Library, World};
use typst_kit::fonts::{FontSlot, Fonts};

use anyhow::Result;
use typst_pdf::{PdfOptions, PdfStandards};

/// Holds the processed data for a file ID.
///
/// Both fields can be populated if the file is both imported and read().
#[derive(Debug)]
struct FileSlot {
    /// The lazily loaded and incrementally updated source file.
    source: SlotCell<Source>,
    /// The lazily loaded raw byte buffer.
    file: SlotCell<Bytes>,
}

impl FileSlot {
    /// Create a new file slot.
    fn new() -> Self {
        Self { file: SlotCell::new(), source: SlotCell::new() }
    }

    /// Marks the file as not yet accessed in preparation of the next
    /// compilation.
    fn reset(&mut self) {
        self.source.reset();
        self.file.reset();
    }

    /// Retrieve the source for this file.
    fn source<F: Fn(FileId) -> FileResult<Vec<u8>>>(&mut self, id: FileId, read: F) -> FileResult<Source> {
        self.source.get_or_init(
            || read(id),
            |data, prev| {
                let text = decode_utf8(&data)?;
                if let Some(mut prev) = prev {
                    prev.replace(text);
                    Ok(prev)
                } else {
                    Ok(Source::new(id, text.into()))
                }
            },
        )
    }

    /// Retrieve the file's bytes.
    fn file<F: Fn(FileId) -> FileResult<Vec<u8>>>(&mut self, id: FileId, read: F) -> FileResult<Bytes> {
        self.file.get_or_init(
            || read(id),
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

pub struct PDF {
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
    read: HashMap<VirtualPath, Vec<u8>>,
}

impl PDF {

    pub fn main<I: Into<Vec<u8>>>(data: I) -> Self {
        let mut pdf = Self::new("main.typ");
        pdf.write("main.typ", data);
        pdf
    }

    pub fn new<M: Into<PathBuf>>(main: M) -> Self {
        let path = main.into();
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
            read: HashMap::new(),
        }
    }

    pub fn write<M: Into<PathBuf>, I: Into<Vec<u8>>>(&mut self, path: M, data: I) {
        let vpath = VirtualPath::new(path.into());

        for (key, value) in self.slots.get_mut() {
            if key.vpath() == &vpath {
                value.reset();
            }
        }

        self.read.insert(vpath, data.into());
    }

    pub fn render(&mut self) -> Result<Vec<u8>, EcoVec<SourceDiagnostic>> {
        let document = compile(self).output?;
    
        let options = PdfOptions {
            ident: Smart::Auto,
            timestamp: self.today(None),
            page_ranges: None,
            standards: PdfStandards::default()
        };

        typst_pdf::pdf(&document, &options)
    }

    pub fn render_with_data<I: Into<Vec<u8>>>(&mut self, data: I) -> Result<Vec<u8>, EcoVec<SourceDiagnostic>> {
        self.write("data.txt", data);
        self.render()
    }

    /// Reset the compilation state in preparation of a new compilation.
    pub fn reset(&mut self) {
        self.slots.get_mut().values_mut().for_each(FileSlot::reset);
        self.now = Utc::now();
    }

    /// Access the canonical slot for the given file id.
    fn slot<F, T>(&self, id: FileId, f: F) -> T
    where
        F: FnOnce(&mut FileSlot) -> T,
    {
        let mut map = self.slots.lock();
        f(map.entry(id).or_insert_with(FileSlot::new))
    }

    fn try_read(&self, id: FileId) -> FileResult<Vec<u8>> {
        self.read.get(id.vpath()).cloned().ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().to_path_buf()))
    }

}

#[track]
impl World for PDF {
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
        self.slot(id, |slot| slot.source(id, |id| self.try_read(id)))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.slot(id, |slot| slot.file(id, |id| self.try_read(id)))
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
