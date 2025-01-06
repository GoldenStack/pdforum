use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Instant,
};

use log::{trace, Level::Trace};

use chrono::{DateTime, Datelike, FixedOffset, Local, Utc};
use comemo::track;
use ecow::EcoVec;
use log::log_enabled;
use typst::{
    compile,
    diag::{FileError, FileResult, SourceDiagnostic},
    foundations::{Bytes, Datetime, Smart},
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, World,
};
use typst_kit::fonts::{FontSlot, Fonts};

use anyhow::Result;
use typst_pdf::{PdfOptions, PdfStandards};

/// A PDF to be rendered with PDForum.
///
/// This is a Typst [World], except it implements a fake filesystem which is
/// internally just an in-memory [HashMap]. This simplifies the overall
/// workflow, makes the entire rendering process faster, and prevents, in the
/// worst case (i.e. Typst injection, which, now that I'm writing it, sounds
/// very funny), arbitrary filesystem reads.
pub struct PDF {
    /// The input path.
    main: FileId,
    /// Typst's standard library.
    library: LazyHash<Library>,
    /// Metadata about discovered fonts.
    book: LazyHash<FontBook>,
    /// Locations of and storage for lazily loaded fonts.
    fonts: Vec<FontSlot>,
    /// The current datetime if requested.
    now: DateTime<Utc>,
    /// Fake isolated filesystem.
    files: HashMap<FileId, Bytes>,
    /// Fake isolated filesystem but exclusively storing source files.
    sources: HashMap<FileId, Source>,
}

impl PDF {
    /// Makes a PDF from the given main path and list of sources.
    /// This is intended to allow creating a PDF with a single expression.
    pub fn make<const C: usize, M: Into<PathBuf>, I: Into<String>>(
        main: M,
        items: [(M, I); C],
    ) -> Self {
        let mut pdf = PDF::new(main);
        for (path, data) in items {
            pdf.write_source(path, data);
        }

        pdf
    }

    /// Creates a new PDF.
    /// This won't really do much; remember to [PDF::write] / [PDF::write_source]
    /// files to this 'filesystem'.
    pub fn new<M: Into<PathBuf>>(main: M) -> Self {
        let path = main.into();
        let root: PathBuf = path.parent().unwrap_or(Path::new(".")).into();

        let main_path = VirtualPath::within_root(&path, &root).unwrap();

        let main = FileId::new(None, main_path);

        let fonts = Fonts::searcher().include_system_fonts(true).search();

        Self {
            main,
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            now: Utc::now(),
            files: HashMap::new(),
            sources: HashMap::new(),
        }
    }

    /// Writes a binary file to this filesystem.
    /// Binary files may or may not be valid UTF-8.
    pub fn write<M: Into<PathBuf>, I: Into<Vec<u8>>>(&mut self, path: M, data: I) {
        let vpath = VirtualPath::new(path.into());

        let id = FileId::new(None, vpath);

        self.files.insert(id, data.into().into());
    }

    /// Writes a source file to this filesystem. It must be valid UTF-8.
    /// This will allow it to be treated both as a source file and a binary
    /// file, but only source files can be actually loaded.
    ///
    /// Generally, source files will be written to once when creating the PDF,
    /// while non-source files will be written to before each render.
    pub fn write_source<M: Into<PathBuf>, I: Into<String>>(&mut self, path: M, data: I) {
        let vpath = VirtualPath::new(path.into());

        let id = FileId::new(None, vpath);

        let data = data.into();

        self.files.insert(id, data.as_bytes().into());
        self.sources.insert(id, Source::new(id, data));
    }

    /// Renders this PDF into bytes representing the actual PDF.
    /// This will [compile] it and then [export](typst_pdf::pdf) it.
    pub fn render(&mut self) -> Result<Vec<u8>, EcoVec<SourceDiagnostic>> {
        let document = compile(self).output?;

        let options = PdfOptions {
            ident: Smart::Auto,
            timestamp: self.today(None),
            page_ranges: None,
            standards: PdfStandards::default(),
        };

        typst_pdf::pdf(&document, &options)
    }

    /// Renders this PDF into bytes representing the actual PDF.
    /// Additionally, this will write the given data to the `data.txt` path,
    /// which (in PDForum, at least) is the standardized location of data,
    /// primarily unsanitized user input data.
    pub fn render_with_data<I: Into<Vec<u8>>>(
        &mut self,
        data: I,
    ) -> Result<Vec<u8>, EcoVec<SourceDiagnostic>> {
        // If trace logging is enabled, time it!
        if log_enabled!(Trace) {
            let start = Instant::now();

            self.write("data.txt", data);
            let result = self.render();

            trace!("Rendered {:?} in {:?}", self.main, start.elapsed());

            return result;
        }

        // Otherwise, just do everything normally
        self.write("data.txt", data);
        self.render()
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
        self.sources
            .get(&id)
            .cloned()
            .ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().to_path_buf()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files
            .get(&id)
            .cloned()
            .ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().to_path_buf()))
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
