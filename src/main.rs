use std::{collections::HashMap, path::{Path, PathBuf}, time::Instant};

use chrono::{DateTime, Datelike, FixedOffset, Local, Utc};
use parking_lot::Mutex;
use typst::{diag::{eco_format, FileError, FileResult}, foundations::{Bytes, Datetime, Smart}, syntax::{FileId, Source, VirtualPath}, text::{Font, FontBook}, utils::LazyHash, Library, World};
use typst_kit::{download::{self, Downloader, ProgressSink}, fonts::{FontSlot, Fonts}, package::PackageStorage};
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

    /// Whether the file was accessed in the ongoing compilation.
    fn accessed(&self) -> bool {
        self.source.accessed() || self.file.accessed()
    }

    /// Marks the file as not yet accessed in preparation of the next
    /// compilation.
    fn reset(&mut self) {
        self.source.reset();
        self.file.reset();
    }

    /// Retrieve the source for this file.
    fn source(
        &mut self,
        read: &Read,
        project_root: &Path,
        package_storage: &PackageStorage,
    ) -> FileResult<Source> {
        self.source.get_or_init(
            || read(self.id, project_root, package_storage),
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
    fn file(
        &mut self,
        read: &Read,
        project_root: &Path,
        package_storage: &PackageStorage,
    ) -> FileResult<Bytes> {
        self.file.get_or_init(
            || read(self.id, project_root, package_storage),
            |data, _| Ok(data.into()),
        )
    }
}

/// Decode UTF-8 with an optional BOM.
fn decode_utf8(buf: &[u8]) -> FileResult<&str> {
    // Remove UTF-8 BOM.
    Ok(std::str::from_utf8(buf.strip_prefix(b"\xef\xbb\xbf").unwrap_or(buf))?)
}

fn read_from_disk(path: &Path) -> FileResult<Vec<u8>> {
    let f = |e| FileError::from_io(e, path);
    if std::fs::metadata(path).map_err(f)?.is_dir() {
        Err(FileError::IsDirectory)
    } else {
        std::fs::read(path).map_err(f)
    }
}

/// Resolves the path of a file id on the system, downloading a package if
/// necessary.
fn system_path(
    project_root: &Path,
    id: FileId,
    package_storage: &PackageStorage,
) -> FileResult<PathBuf> {
    // Determine the root path relative to which the file path
    // will be resolved.
    let buf;
    let mut root = project_root;
    if let Some(spec) = id.package() {
        buf = package_storage.prepare_package(spec, &mut ProgressSink)?;
        root = &buf;
    }

    // Join the path to the root. If it tries to escape, deny
    // access. Note: It can still escape via symlinks.
    id.vpath().resolve(root).ok_or(FileError::AccessDenied)
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

    /// Whether the cell was accessed in the ongoing compilation.
    fn accessed(&self) -> bool {
        self.accessed
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

/// A world that provides access to the operating system.
pub struct FWorld {
    /// The root relative to which absolute paths are resolved.
    root: PathBuf,
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
    /// Holds information about where packages are stored.
    package_storage: PackageStorage,
    /// The current datetime if requested.
    now: DateTime<Utc>,
    /// Function for reading files from the filesystem.
    read: Read,
}

pub type Read = fn(FileId, &Path, &PackageStorage) -> FileResult<Vec<u8>>;

impl FWorld {
    pub fn new(path: PathBuf) -> Self {
        let root: PathBuf = path.parent().unwrap_or(Path::new(".")).into();

        let main_path = VirtualPath::within_root(&path, &root).unwrap();

        let main = FileId::new(None, main_path);

        let fonts = Fonts::searcher()
            .include_system_fonts(true)
            .search();

        let read: Read = |id, project_root, package_storage| {
            println!("{:?}, {:?}", id.vpath(), id.vpath().as_rootless_path());
            if id.vpath().as_rootless_path() == Path::new("example.typ") {
                let mut s = r#"#import "@preview/yagenda:0.1.0": *
#show: agenda.with(
  name: "Pumpkins and peanuts committee", 
  date: "03/01/2000", 
  time: "2 pm",
  location: "baseball field", 
  invited: [Sally, Shroeder, Pig-pen, Marcie]
)

// load external yaml
// #let topics = yaml("agenda.yaml")

// alternative: embed yaml in-place

#let tmp = `
"admin":
  Topic: Misc. admin
  Time: 5 mins
  Lead: baptiste 
  Purpose: Inform, Admin
  Preparation: |
    - Anything to change on the agenda? 
    - Read minutes from last meeting
    - Review action points
  Process: |
    - Check if agenda needs to be adjusted
    - Check if everyone's happy with the minutes
    - List top priorities of meeting and items needing decisions
    
"snoopy update":
  Topic: Snoopy's latest 
  Time: 10 mins
  Lead: Charlie Brown
  Purpose: Share, Discuss
  Preparation: |
    - Bring your favorite comic strip
    - Reflect on Snoopy's recent escapades
  Process: |
    - Discuss Snoopy's ongoing "battle" with the Red Baron
    - Share anecdotes of Snoopy's fiercest war moves
    - Plan a group outing to visit Snoopy's doghouse

"woodstock sighting":
  Topic: Woodstock's whereabouts
  Time: 5 mins
  Lead: Linus
  Purpose: Locate, Update
  Preparation: |
    - Bring binoculars for bird-watching
    - Brush up on bird calls
  Process: |
    - Share any recent sightings or chirpings from Woodstock
    - Discuss strategies for bringing Woodstock back to the tunes
    - Plan an upside-down birdhouse-building workshop 

"great pumpkin plans":
  Topic: Prep for Great Pumpkin
  Time: 15 mins
  Lead: Linus
  Purpose: Plan, Excite
  Preparation: |
    - Bring your pumpkin-carving tools
    - Practice your most sincere pumpkin patch speech
  Process: |
    - Discuss tactics for maximizing sincerity in the pumpkin patch
    - Brainstorm new ways to attract the Great Pumpkin's attention
    - Assign roles for the annual pumpkin carving contest

"philosophical discussion":
  Topic: Philosophical Musings
  Time: 20 mins
  Lead: Lucy and Charles
  Purpose: Ponder, Reflect
  Preparation: |
    - Bring your favorite existentialist quotes
    - Contemplate the meaning of life #v(100%)
  Process: |
    - Engage in deep philosophical discussions under the night sky
    - Debate the nature of happiness, existence, and 5c psychiatry
    - Seek wisdom from Linus's trusty security blanket

"beethoven appreciation":
  Topic: Beethoven's Legacy
  Time: 10 mins
  Lead: Schroeder
  Purpose: Appreciate, Discuss
  Preparation: |
    - Bring your favorite Beethoven compositions
    - Practice your air piano skills
  Process: |
    - Listen to Schroeder perform select Beethoven pieces
    - Discuss the timeless appeal of Beethoven's music
    - Plan a Beethoven-themed recital for Christmas

"baseball game":
  Topic: Baseball Game
  Time: 30 mins
  Lead: Charlie Brown
  Purpose: Play, Bond
  Preparation: |
    - Bring your baseball glove and bat
    - Review the rules of baseball
  Process: |
    - Divide into teams for an exciting game of baseball
    - Cheer on Charlie Brown as he attempts to finally kick that football
    - Enjoy snacks and camaraderie under the shade of the old oak tree
    
"summer camp":
  Topic: Summer Camp Adventures
  Time: 15 mins
  Lead: Peppermint Patty
  Purpose: Plan, Excite
  Preparation: |
    - Bring ideas for summer camp activities!
    - Check availability of camping gear!
  Process: |
    - Brainstorm camping trip destinations and outdoor activities
    - Discuss potential guest speakers or counselors
    - Organize a talent show and marshmallow roasting competition

"marcie school update":
  Topic: Marcie's Academic Progress
  Time: 10 mins
  Lead: Peppermint Patty
  Purpose: Discuss, Support
  Preparation: |
    - Bring Marcie's recent report card
    - Reflect on Marcie's study habits and strengths
  Process: |
    - Share updates on Marcie's academic achievements and challenges
    - Discuss strategies to help Marcie excel in school
    - Offer encouragement and support to Marcie in her studies

"linus blanket workshop":
  Topic: Linus and his blanket
  Time: 20 mins
  Lead: Lucy
  Purpose: Understand, *Support*
  Preparation: |
    - Bring your own security blanket (optional)
    - Reflect on the significance of Linus's blanket
  Process: |
    - Discuss the history and symbolism of Linus's security blanket
    - Explore ways to help both Linus and Snoopy 
    - Release a blanket statement

"snoopy plane fights":
  Topic: Snoopy vs the Baron
  Time: 10 mins
  Lead: Woodstock
  Purpose: Support
  Preparation: |
    - Bring your favorite Snoopy flying ace memory
    - Reflect on Snoopy's aerial prowess
  Process: |
    - Share anecdotes from Snoopy's battles
    - Discuss the psychological implications of Snoopy's dogging of bullets
    `.text

#let topics = yaml.decode(tmp)

#agenda-table(topics)

#set page(flipped: false)

== Appendix

#lorem(120)

"#.to_string();

let r = format!("{:?}\n", Instant::now());
println!("R {}", r);
s.push_str(r.as_str());

return Ok(s.as_bytes().to_vec());
            };

            read_from_disk(&system_path(project_root, id, package_storage)?)
        };

        Self {
            root,
            main,
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            slots: Mutex::new(HashMap::new()),
            package_storage: PackageStorage::new(
                None,
                None,
                Downloader::new(concat!("typst/", env!("CARGO_PKG_VERSION"))),
            ),
            now: Utc::now(),
            read,
        }
    }

    /// Return all paths the last compilation depended on.
    pub fn dependencies(&mut self) -> impl Iterator<Item = PathBuf> + '_ {
        self.slots
            .get_mut()
            .values()
            .filter(|slot| slot.accessed())
            .filter_map(|slot| {
                system_path(&self.root, slot.id, &self.package_storage).ok()
            })
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
        self.slot(id, |slot| slot.source(&self.read, &self.root, &self.package_storage))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.slot(id, |slot| slot.file(&self.read, &self.root, &self.package_storage))
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