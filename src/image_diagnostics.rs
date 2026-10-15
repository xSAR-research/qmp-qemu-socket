//! Optional original-PNG recording for worker-owned GAME WIN sequences.
//!
//! Startup resets only the fixed diagnostics directory. The worker supplies the
//! existing capture bytes and their startup-relative capture-completion time;
//! this module never captures, decodes, modifies or authorises guest input.

use std::{
    ffi::OsString,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::{fs::OpenOptions, os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt}};


/// The sole production destination, reset once after startup arguments validate.
const IMAGE_DIRECTORY: &str = "/home/charlie/tmp/images";


/// Validated startup intent; parsing itself performs no filesystem operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StartupOptions {
    /// Start the application with diagnostics enabled or disabled.
    Run {
        /// Record existing captures only during GAME WIN sequences.
        images: bool,
    },
    /// Display help without resetting diagnostics or starting the application.
    Help,
}


/// Parse arguments excluding the executable name, validating every argument.
///
/// Help does not bypass invalid or duplicated arguments. Call startup reset
/// only after this function has returned a successful `Run` decision.
pub(crate) fn parse_args(
    args: impl IntoIterator<Item = OsString>,
) -> Result<StartupOptions, String> {
    let mut images = false;
    let mut help = false;
    for argument in args {
        match argument.to_str() {
            Some("--image") if !images => images = true,
            Some("--image") => return Err("Duplicate startup option --image.".to_owned()),
            Some("--help" | "-h") if !help => help = true,
            Some("--help" | "-h") => return Err("Duplicate startup help option.".to_owned()),
            _ => return Err(format!("Unsupported startup argument {argument:?}; use --help.")),
        }
    }
    Ok(if help { StartupOptions::Help } else { StartupOptions::Run { images } })
}


/// Shared startup origin and optional private diagnostics destination.
#[derive(Clone, Debug)]
pub(crate) struct ImageDiagnostics {
    /// Origin shared with worker log prefixes and capture-completion timestamps.
    pub started_at: Instant,
    /// Present only after an enabled startup has prepared the private directory.
    pub directory: Option<PathBuf>,
}


impl ImageDiagnostics {
    /// Reset the exact production directory, recreating it only when enabled.
    pub(crate) fn prepare(images: bool) -> io::Result<Self> {
        Self::prepare_in(Path::new(IMAGE_DIRECTORY), images)
    }


    /// Keep filesystem tests isolated from the production diagnostics path.
    fn prepare_in(root: &Path, images: bool) -> io::Result<Self> {
        let started_at = Instant::now();
        match fs::symlink_metadata(root) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Image diagnostics path is not a real directory: {}", root.display()),
                    ));
                }
                fs::remove_dir_all(root)?;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        let directory = if images {
            create_private_directory(root)?;
            Some(root.to_path_buf())
        } else {
            None
        };
        Ok(Self { started_at, directory })
    }
}


/// One retained image, with capture time distinct from its filesystem write time.
#[derive(Debug)]
pub(crate) struct RecordedImage {
    /// Unique destination of the original, unmodified PNG bytes.
    pub path: PathBuf,
    /// Startup-relative capture-completion time supplied by the worker.
    pub elapsed: Duration,
    /// Monotonic recording attempt number, shared across GAME WIN sequences.
    pub sequence: u64,
    /// Time spent creating and writing this image, excluding the QMP capture.
    pub write_duration: Duration,
}


/// Only the most recently observed original PNG is retained for sequence entry.
struct CachedImage {
    /// Owned original bytes, without decoding or re-encoding.
    png: Vec<u8>,
    /// Capture-completion timestamp associated with these exact bytes.
    elapsed: Duration,
}


/// Worker-owned recorder; the caller establishes and ends GAME WIN authority.
pub(crate) struct ImageRecorder {
    /// Immutable startup origin and optional destination.
    config: ImageDiagnostics,
    /// Latest enabled capture, used once when beginning a sequence.
    latest: Option<CachedImage>,
    /// Whether newly observed captures are currently recorded.
    active: bool,
    /// Last allocated attempt number; failures may leave gaps without reuse.
    sequence: u64,
}


impl ImageRecorder {
    /// Construct an inactive recorder without creating or resetting directories.
    pub(crate) fn new(config: ImageDiagnostics) -> Self {
        Self { config, latest: None, active: false, sequence: 0 }
    }


    /// Cache inactive enabled captures; write active captures directly from their bytes.
    ///
    /// The caller should invoke this before decoding so an active failed-decode
    /// capture is retained too. Disabled recording neither copies nor writes bytes.
    pub(crate) fn observe_png(
        &mut self,
        png: &[u8],
        elapsed: Duration,
    ) -> io::Result<Option<RecordedImage>> {
        if self.config.directory.is_none() {
            return Ok(None);
        }
        if self.active {
            return self.record(png, elapsed).map(Some);
        }
        if let Some(cached) = self.latest.as_mut() {
            let additional = png.len().saturating_sub(cached.png.len());
            cached.png.try_reserve_exact(additional).map_err(|error|
                io::Error::other(format!("Cannot cache the latest diagnostic PNG: {error}")))?;
            cached.png.clear();
            cached.png.extend_from_slice(png);
            cached.elapsed = elapsed;
        } else {
            let mut bytes = Vec::new();
            bytes.try_reserve_exact(png.len()).map_err(|error|
                io::Error::other(format!("Cannot cache the latest diagnostic PNG: {error}")))?;
            bytes.extend_from_slice(png);
            self.latest = Some(CachedImage { png: bytes, elapsed });
        }
        Ok(None)
    }


    /// Write the latest entry capture once, then record subsequent existing captures.
    ///
    /// Enabled entry without a cached capture and nested entry both fail without
    /// activation. The worker manages nesting and supplies its fresh entry image.
    pub(crate) fn begin(&mut self) -> io::Result<Option<RecordedImage>> {
        if self.config.directory.is_none() {
            return Ok(None);
        }
        if self.active {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Image diagnostics sequence is already active."));
        }
        let cached = self.latest.take().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidInput, "Image diagnostics entry has no cached capture."))?;
        match self.record(&cached.png, cached.elapsed) {
            Ok(recorded) => {
                self.active = true;
                Ok(Some(recorded))
            }
            Err(error) => {
                self.latest = Some(cached);
                Err(error)
            }
        }
    }


    /// End recording and discard its RAM cache, retaining every recorded file.
    ///
    /// A later enabled sequence must observe another capture before beginning;
    /// the previous sequence's final image cannot silently become a stale entry.
    pub(crate) fn end(&mut self) {
        self.active = false;
        self.latest = None;
    }


    /// Allocate a unique name and preserve all earlier evidence on failure.
    fn record(&mut self, png: &[u8], elapsed: Duration) -> io::Result<RecordedImage> {
        let directory = self.config.directory.as_ref().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidInput, "Image diagnostics recording is disabled."))?;
        let sequence = self.sequence.checked_add(1)
            .ok_or_else(|| io::Error::other("Image diagnostics sequence counter exhausted."))?;
        self.sequence = sequence;
        let write_started = Instant::now();
        let milliseconds = elapsed.as_millis();
        let filename = format!("{:06}.{:03}_{sequence:06}.png", milliseconds / 1_000, milliseconds % 1_000);
        let path = directory.join(filename);
        let metadata = fs::symlink_metadata(directory)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                format!("Image diagnostics destination is not a real directory: {}", directory.display())));
        }
        let file = create_private_file(&path)?;
        write_original_bytes(file, &path, png)?;
        Ok(RecordedImage { path, elapsed, sequence, write_duration: write_started.elapsed() })
    }
}


/// Create a private Unix directory atomically, without creating other paths.
#[cfg(unix)]
fn create_private_directory(path: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700).create(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}


/// Refuse enabled diagnostics where the requested Unix permissions are unavailable.
#[cfg(not(unix))]
fn create_private_directory(_path: &Path) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Unsupported,
        "Image diagnostics requires Unix directory and file permissions."))
}


/// Create a new owner-only file; never overwrite existing evidence or a symlink.
#[cfg(unix)]
fn create_private_file(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?;
    if let Err(error) = file.set_permissions(fs::Permissions::from_mode(0o600)) {
        drop(file);
        return Err(remove_failed_file(path, error));
    }
    Ok(file)
}


/// Match the enabled-diagnostics refusal on hosts without Unix permissions.
#[cfg(not(unix))]
fn create_private_file(_path: &Path) -> io::Result<File> {
    Err(io::Error::new(io::ErrorKind::Unsupported,
        "Image diagnostics requires Unix directory and file permissions."))
}


/// Write exact supplied bytes and remove only this newly created partial file on error.
///
/// Successful writes are closed without an additional filesystem durability flush.
/// The generic writer allows a real partial filesystem write failure to be tested.
fn write_original_bytes(mut writer: impl Write, path: &Path, png: &[u8]) -> io::Result<()> {
    if let Err(error) = writer.write_all(png) {
        drop(writer);
        return Err(remove_failed_file(path, error));
    }
    Ok(())
}


/// Preserve the write error and report a failed partial-file cleanup explicitly.
fn remove_failed_file(path: &Path, original: io::Error) -> io::Error {
    match fs::remove_file(path) {
        Ok(()) => original,
        Err(cleanup) if cleanup.kind() == io::ErrorKind::NotFound => original,
        Err(cleanup) => io::Error::new(original.kind(), format!(
            "{original}; cannot remove partial diagnostic image {}: {cleanup}", path.display())),
    }
}


#[cfg(all(test, unix))]
mod tests {
    //! Filesystem tests use private temporary roots, never the production directory.

    use std::{
        os::unix::{ffi::OsStringExt, fs::symlink},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;


    /// Distinguish parallel tests without sharing resettable directories.
    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);


    /// Opaque binary bytes exercise exact preservation without invoking a PNG decoder.
    const ENTRY_BYTES: &[u8] = b"\x89PNG\r\n\x1a\nentry\0original bytes";


    /// A distinct payload reveals a stale cache or omitted active capture.
    const NEXT_BYTES: &[u8] = b"\x89PNG\r\n\x1a\nnext\0original bytes";


    /// Private test container removed only when its owning test completes.
    struct TestRoot(
        /// Exclusive container for this test's files and deliberate symlinks.
        PathBuf,
    );


    impl TestRoot {
        /// Create a unique test root without resetting any pre-existing path.
        fn new() -> Self {
            let serial = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "qmp-qemu-image-diagnostics-{}-{serial}", std::process::id()));
            create_private_directory(&path).unwrap();
            Self(path)
        }


        /// Destination supplied to the private production preparation helper.
        fn images(&self) -> PathBuf {
            self.0.join("images")
        }
    }


    impl Drop for TestRoot {
        /// Remove this test's temporary container, including any deliberate symlink.
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }


    /// Construct platform-native argument vectors for the side-effect-free parser.
    fn arguments(values: &[&str]) -> Vec<OsString> {
        values.iter().map(|value| OsString::from(*value)).collect()
    }


    /// Defaults and supported help/image combinations produce explicit startup decisions.
    #[test]
    fn startup_options_accept_only_supported_intents() {
        assert_eq!(parse_args(arguments(&[])), Ok(StartupOptions::Run { images: false }));
        assert_eq!(parse_args(arguments(&["--image"])), Ok(StartupOptions::Run { images: true }));
        for flags in [["--help", "--image"], ["--image", "-h"]] {
            assert_eq!(parse_args(arguments(&flags)), Ok(StartupOptions::Help));
        }
        assert_eq!(parse_args(arguments(&["-h"])), Ok(StartupOptions::Help));
    }


    /// Duplicates, positional arguments and invalid native strings cannot reach reset.
    #[test]
    fn startup_options_reject_invalid_arguments_even_after_help() {
        for flags in [
            vec!["--image", "--image"], vec!["--help", "-h"],
            vec!["--help", "--unknown"], vec!["--images"], vec!["image"],
        ] {
            assert!(parse_args(arguments(&flags)).is_err(), "{flags:?}");
        }
        assert!(parse_args([OsString::from_vec(vec![0xff])]).is_err());
    }


    /// Disabled startup removes previous nested evidence without recreating its destination.
    #[test]
    fn disabled_startup_resets_existing_directory_without_recreation() {
        let root = TestRoot::new();
        let images = root.images();
        fs::create_dir_all(images.join("previous-sequence")).unwrap();
        fs::write(images.join("previous-sequence/frame.png"), ENTRY_BYTES).unwrap();
        let config = ImageDiagnostics::prepare_in(&images, false).unwrap();
        assert!(config.directory.is_none());
        assert!(!images.exists());
        assert!(ImageDiagnostics::prepare_in(&images, false).unwrap().directory.is_none());
        assert!(!images.exists());
    }


    /// Enabled startup clears old images and creates an empty owner-only directory.
    #[test]
    fn enabled_startup_resets_and_recreates_private_directory() {
        let root = TestRoot::new();
        let images = root.images();
        fs::create_dir(&images).unwrap();
        fs::write(images.join("old.png"), ENTRY_BYTES).unwrap();
        let config = ImageDiagnostics::prepare_in(&images, true).unwrap();
        assert_eq!(config.directory.as_ref(), Some(&images));
        assert_eq!(fs::read_dir(&images).unwrap().count(), 0);
        assert_eq!(fs::metadata(&images).unwrap().permissions().mode() & 0o777, 0o700);
    }


    /// A reset path symlink is refused and its foreign contents remain untouched.
    #[test]
    fn symlink_reset_refusal_preserves_foreign_target() {
        let root = TestRoot::new();
        let foreign = root.0.join("foreign");
        fs::create_dir(&foreign).unwrap();
        fs::write(foreign.join("keep.png"), ENTRY_BYTES).unwrap();
        let images = root.images();
        symlink(&foreign, &images).unwrap();
        for enabled in [false, true] {
            assert_eq!(ImageDiagnostics::prepare_in(&images, enabled).unwrap_err().kind(),
                io::ErrorKind::InvalidInput);
            assert!(fs::symlink_metadata(&images).unwrap().file_type().is_symlink());
            assert_eq!(fs::read(foreign.join("keep.png")).unwrap(), ENTRY_BYTES);
        }
    }


    /// An unexpected regular file at the reset path is preserved and diagnosed.
    #[test]
    fn non_directory_reset_refusal_preserves_file() {
        let root = TestRoot::new();
        let images = root.images();
        fs::write(&images, ENTRY_BYTES).unwrap();
        assert_eq!(ImageDiagnostics::prepare_in(&images, true).unwrap_err().kind(),
            io::ErrorKind::InvalidInput);
        assert_eq!(fs::read(&images).unwrap(), ENTRY_BYTES);
    }


    /// Disabled recording creates no cache, activation, file or destination directory.
    #[test]
    fn disabled_recorder_does_not_cache_or_write() {
        let root = TestRoot::new();
        let images = root.images();
        let mut recorder = ImageRecorder::new(ImageDiagnostics::prepare_in(&images, false).unwrap());
        assert!(recorder.observe_png(ENTRY_BYTES, Duration::ZERO).unwrap().is_none());
        assert!(recorder.begin().unwrap().is_none());
        assert!(recorder.latest.is_none());
        assert!(!recorder.active);
        assert!(!images.exists());
    }


    /// Inactive captures replace the cache; entry saves only the latest original bytes.
    #[test]
    fn entry_records_latest_inactive_png_once_with_capture_time_and_permissions() {
        let root = TestRoot::new();
        let images = root.images();
        let mut recorder = ImageRecorder::new(ImageDiagnostics::prepare_in(&images, true).unwrap());
        recorder.observe_png(ENTRY_BYTES, Duration::from_secs(1)).unwrap();
        let elapsed = Duration::from_micros(113_310_999);
        recorder.observe_png(NEXT_BYTES, elapsed).unwrap();
        assert_eq!(fs::read_dir(&images).unwrap().count(), 0);
        let recorded = recorder.begin().unwrap().unwrap();
        assert_eq!(recorded.path.file_name().unwrap(), "000113.310_000001.png");
        assert_eq!(recorded.elapsed, elapsed);
        assert_eq!(recorded.sequence, 1);
        assert_eq!(fs::read(&recorded.path).unwrap(), NEXT_BYTES);
        assert_eq!(fs::metadata(&recorded.path).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(recorder.begin().unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(fs::read_dir(&images).unwrap().count(), 1);
    }


    /// Enabled entry without a cached frame fails without silently entering recording.
    #[test]
    fn missing_entry_capture_does_not_activate() {
        let root = TestRoot::new();
        let images = root.images();
        let mut recorder = ImageRecorder::new(ImageDiagnostics::prepare_in(&images, true).unwrap());
        assert_eq!(recorder.begin().unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert!(!recorder.active);
        assert_eq!(fs::read_dir(&images).unwrap().count(), 0);
    }


    /// Every active capture is retained, while later scopes require a new entry frame.
    #[test]
    fn active_frames_and_later_sequences_are_retained_until_next_startup() {
        let root = TestRoot::new();
        let images = root.images();
        let mut recorder = ImageRecorder::new(ImageDiagnostics::prepare_in(&images, true).unwrap());
        recorder.observe_png(ENTRY_BYTES, Duration::from_millis(1)).unwrap();
        let first = recorder.begin().unwrap().unwrap();
        let second = recorder.observe_png(NEXT_BYTES, Duration::from_millis(2)).unwrap().unwrap();
        let third = recorder.observe_png(ENTRY_BYTES, Duration::from_millis(3)).unwrap().unwrap();
        assert!(recorder.latest.is_none(), "active recording retains no extra full-PNG copy");
        recorder.end();
        assert_eq!(recorder.begin().unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert!(recorder.observe_png(NEXT_BYTES, Duration::from_millis(4)).unwrap().is_none());
        assert_eq!(fs::read_dir(&images).unwrap().count(), 3);
        let fourth = recorder.begin().unwrap().unwrap();
        assert_eq!(fourth.sequence, 4);
        for (record, expected) in [(&first, ENTRY_BYTES), (&second, NEXT_BYTES),
            (&third, ENTRY_BYTES), (&fourth, NEXT_BYTES)]
        {
            assert_eq!(fs::read(&record.path).unwrap(), expected);
        }
        recorder.end();
        assert_eq!(fs::read_dir(&images).unwrap().count(), 4);
    }


    /// Equal elapsed milliseconds receive unique monotonically numbered filenames.
    #[test]
    fn duplicate_capture_times_cannot_overwrite_previous_images() {
        let root = TestRoot::new();
        let images = root.images();
        let mut recorder = ImageRecorder::new(ImageDiagnostics::prepare_in(&images, true).unwrap());
        let elapsed = Duration::from_millis(7);
        recorder.observe_png(ENTRY_BYTES, elapsed).unwrap();
        let first = recorder.begin().unwrap().unwrap();
        let second = recorder.observe_png(NEXT_BYTES, elapsed).unwrap().unwrap();
        assert_ne!(first.path, second.path);
        assert_eq!(first.sequence, 1);
        assert_eq!(second.sequence, 2);
        assert_eq!(fs::read(first.path).unwrap(), ENTRY_BYTES);
        assert_eq!(fs::read(second.path).unwrap(), NEXT_BYTES);
    }


    /// A failed active write surfaces an error while prior evidence remains intact.
    #[test]
    fn write_failure_preserves_previous_recorded_evidence() {
        let root = TestRoot::new();
        let images = root.images();
        let mut recorder = ImageRecorder::new(ImageDiagnostics::prepare_in(&images, true).unwrap());
        recorder.observe_png(ENTRY_BYTES, Duration::ZERO).unwrap();
        let first = recorder.begin().unwrap().unwrap();
        let retained = root.0.join("retained");
        fs::rename(&images, &retained).unwrap();
        fs::write(&images, b"not a directory").unwrap();
        assert!(recorder.observe_png(NEXT_BYTES, Duration::from_secs(1)).is_err());
        assert_eq!(fs::read(retained.join(first.path.file_name().unwrap())).unwrap(), ENTRY_BYTES);
        assert_eq!(fs::read(&images).unwrap(), b"not a directory");
    }


    /// A destination replaced by a symlink cannot receive active diagnostic writes.
    #[test]
    fn changed_destination_symlink_is_refused_without_following_target() {
        let root = TestRoot::new();
        let images = root.images();
        let mut recorder = ImageRecorder::new(ImageDiagnostics::prepare_in(&images, true).unwrap());
        recorder.observe_png(ENTRY_BYTES, Duration::ZERO).unwrap();
        let first = recorder.begin().unwrap().unwrap();
        let retained = root.0.join("retained");
        let foreign = root.0.join("foreign");
        fs::rename(&images, &retained).unwrap();
        fs::create_dir(&foreign).unwrap();
        symlink(&foreign, &images).unwrap();
        assert_eq!(recorder.observe_png(NEXT_BYTES, Duration::from_secs(1)).unwrap_err().kind(),
            io::ErrorKind::InvalidInput);
        assert_eq!(fs::read_dir(&foreign).unwrap().count(), 0);
        assert_eq!(fs::read(retained.join(first.path.file_name().unwrap())).unwrap(), ENTRY_BYTES);
    }


    /// A destination collision never overwrites bytes from another successful write.
    #[test]
    fn create_new_collision_preserves_existing_file_and_cached_entry() {
        let root = TestRoot::new();
        let images = root.images();
        let mut recorder = ImageRecorder::new(ImageDiagnostics::prepare_in(&images, true).unwrap());
        recorder.observe_png(ENTRY_BYTES, Duration::ZERO).unwrap();
        let existing = images.join("000000.000_000001.png");
        fs::write(&existing, NEXT_BYTES).unwrap();
        assert_eq!(recorder.begin().unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert!(!recorder.active);
        assert_eq!(fs::read(&existing).unwrap(), NEXT_BYTES);
        let recovered = recorder.begin().unwrap().unwrap();
        assert_eq!(recovered.sequence, 2);
        assert_eq!(fs::read(recovered.path).unwrap(), ENTRY_BYTES);
    }


    /// A real file writer deliberately fails after committing a short binary prefix.
    struct FailAfterPrefix {
        /// Real newly created destination, closed before partial-file cleanup.
        file: File,
        /// Remaining successful bytes before the injected I/O failure.
        remaining: usize,
    }


    impl Write for FailAfterPrefix {
        /// Write a bounded prefix, then report a filesystem-style write failure.
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.remaining == 0 {
                return Err(io::Error::other("Injected partial write failure."));
            }
            let count = self.file.write(&bytes[..bytes.len().min(self.remaining)])?;
            self.remaining -= count;
            Ok(count)
        }


        /// Preserve the underlying file writer's flush behaviour.
        fn flush(&mut self) -> io::Result<()> {
            self.file.flush()
        }
    }


    /// Failed partial writes are removed without touching earlier successful evidence.
    #[test]
    fn partial_write_cleanup_removes_only_the_failed_new_file() {
        let root = TestRoot::new();
        let previous = root.0.join("previous.png");
        fs::write(&previous, NEXT_BYTES).unwrap();
        let partial = root.0.join("partial.png");
        let writer = FailAfterPrefix { file: create_private_file(&partial).unwrap(), remaining: 8 };
        assert!(write_original_bytes(writer, &partial, ENTRY_BYTES).is_err());
        assert!(!partial.exists());
        assert_eq!(fs::read(previous).unwrap(), NEXT_BYTES);
    }
}
