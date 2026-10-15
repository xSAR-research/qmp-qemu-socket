//! Worker image boundaries preserve original QMP bytes without saving ordinary play.

use std::{
    io::{BufRead, BufReader, Write},
    os::unix::{fs::DirBuilderExt, net::{UnixListener, UnixStream}},
    sync::atomic::AtomicU64,
};

use serde_json::{Value, json};

use super::*;


/// Original native Free Cell stages keep byte-preservation checks independent of encoders.
const ENTRY_PNG: &[u8] = include_bytes!("../../tests/fixtures/freecell-FC09.png");
/// Native Level Up pixels follow the saved reward entry.
const LEVEL_UP_PNG: &[u8] = include_bytes!("../../tests/fixtures/freecell-FC10.png");
/// A freshly classified gameplay frame contains an actionable Solver source.
const READY_PNG: &[u8] = include_bytes!("../../tests/fixtures/freecell-FC02.png");
/// Later ordinary gameplay pixels must remain outside the image recording scope.
const ORDINARY_PNG: &[u8] = include_bytes!("../../tests/fixtures/freecell-FC03.png");


/// Distinguish private directories when worker tests run concurrently.
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);


/// A private test root never touches the production image directory.
struct TestDirectory(
    /// Path created exclusively for this test and removed when its guard drops.
    PathBuf,
);


impl TestDirectory {
    /// Create a new private root without resetting any pre-existing path.
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("qmp-image-worker-{}-{}",
            std::process::id(), NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }

    /// Create the private destination supplied to the production recorder.
    fn images(&self) -> PathBuf {
        let path = self.0.join("images");
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        path
    }
}


impl Drop for TestDirectory {
    /// Remove only the temporary root owned by this test.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}


/// Restore the calling test thread's recorder even when an assertion unwinds.
struct DiagnosticsRestore(
    /// Previous worker state, restored after this test finishes or unwinds.
    Option<WorkerImageDiagnostics>,
);


impl DiagnosticsRestore {
    /// Install enabled diagnostics with an observable startup-relative clock.
    fn install(directory: PathBuf) -> Self {
        let config = ImageDiagnostics {
            started_at: Instant::now() - Duration::from_secs(5), directory: Some(directory),
        };
        Self(WORKER_IMAGE_DIAGNOSTICS.with(|slot|
            slot.replace(Some(WorkerImageDiagnostics::new(config)))))
    }
}


impl Drop for DiagnosticsRestore {
    /// Prevent one test's recorder or clock from leaking into another test.
    fn drop(&mut self) {
        WORKER_IMAGE_DIAGNOSTICS.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}


/// Capture normal production notifications without running the UI receiver.
fn sink() -> (WorkerEventSink, Receiver<ContextualWorkerEvent>) {
    let (events, receiver) = mpsc::channel();
    (WorkerEventSink {
        events, latest_frame: LatestFrameSlot::default(), capture_context: Mutex::new(None),
    }, receiver)
}


/// Sort recorded images by their padded capture time and sequence number.
fn png_paths(directory: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(directory).unwrap().map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "png")).collect();
    paths.sort();
    paths
}


/// Flush one newline-delimited QMP response with the original request identity.
fn packet(stream: &mut UnixStream, value: Value) {
    serde_json::to_writer(&mut *stream, &value).unwrap();
    stream.write_all(b"\n").unwrap();
    stream.flush().unwrap();
}


/// Serve only capture requests; unexpected input is a test failure.
fn serve_captures(listener: UnixListener, images: Vec<&'static [u8]>) -> usize {
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "mock QMP accept timed out");
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("mock QMP accept failed: {error}"),
        }
    };
    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    packet(&mut stream, json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}));
    let mut captures = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap() == 0 { break; }
        let request: Value = serde_json::from_str(&line).unwrap();
        match request["execute"].as_str().unwrap() {
            "qmp_capabilities" => {},
            "screendump" => {
                assert_eq!(request["arguments"]["format"], "png");
                let path = request["arguments"]["filename"].as_str().unwrap();
                fs::write(path, images.get(captures).expect("unexpected extra QMP capture")).unwrap();
                captures += 1;
            },
            command => panic!("image recording must not request input or other QMP work: {command}"),
        }
        packet(&mut stream, json!({"return": {}, "id": request["id"]}));
    }
    captures
}


/// Join a bounded mock server during unwind, after the later-created client closes.
struct ServerJoin(
    /// Live server thread returning the number of requests it served.
    Option<JoinHandle<usize>>,
);


impl ServerJoin {
    /// Surface server assertions after the client has closed its connection.
    fn finish(mut self) -> usize {
        self.0.take().unwrap().join().expect("mock capture server")
    }
}


impl Drop for ServerJoin {
    /// Reap the timeout-bounded server when an earlier assertion unwinds.
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() { let _ = handle.join(); }
    }
}


/// The real QMP acquisition boundary saves entry, transition and ready frames only.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn original_qmp_pngs_are_scoped_through_ready_then_ordinary_play_is_unsaved() {
    let _capture_backend = TestCaptureBackend::legacy_qmp();
    let root = TestDirectory::new();
    let images = root.images();
    let _restore = DiagnosticsRestore::install(images.clone());
    let (events, receiver) = sink();
    let socket = root.0.join("capture.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = ServerJoin(Some(thread::spawn(move ||
        serve_captures(listener, vec![ENTRY_PNG, LEVEL_UP_PNG, READY_PNG, ORDINARY_PNG]))));
    let mut qmp = QmpClient::connect(&socket).unwrap();

    capture_screen(&mut qmp, &socket, &events).unwrap();
    assert!(png_paths(&images).is_empty(), "ordinary acquisition only caches the possible entry");
    {
        let _outer = begin_game_win_images(&events).unwrap();
        assert!(game_win_images_active());
        {
            let _inner = begin_game_win_images(&events).unwrap();
            capture_screen(&mut qmp, &socket, &events).unwrap();
        }
        assert!(game_win_images_active(), "inner scope cannot stop the outer transition");
        let (ready, _) = capture_screen(&mut qmp, &socket, &events).unwrap();
        assert!(is_gameplay_scene_for_mode(&ready, GameMode::FreeCell).unwrap());
        assert!(matches!(crate::freecell::analyse(&ready).unwrap().prediction, PredictedAction::Action(_)));
    }
    assert!(!game_win_images_active());
    capture_screen(&mut qmp, &socket, &events).unwrap();
    drop(qmp);
    assert_eq!(server.finish(), 4, "recording adds no screendump requests");

    let paths = png_paths(&images);
    assert_eq!(paths.len(), 3);
    for (path, expected) in paths.iter().zip([ENTRY_PNG, LEVEL_UP_PNG, READY_PNG]) {
        assert_eq!(fs::read(path).unwrap(), expected, "original PNG bytes must remain unchanged");
    }
    let notifications: Vec<_> = receiver.try_iter().collect();
    assert!(notifications.iter().all(|event| event.emitted_at.is_some_and(|elapsed| elapsed >= Duration::from_secs(5))));
    let image_logs = notifications.iter().filter(|event| matches!(&event.event,
        WorkerEvent::Log(message) if message.starts_with("GAME WIN PNG ("))).count();
    assert_eq!(image_logs, 3);
}


/// A stopped controller closes its scope without removing evidence or saving later play.
#[test]
fn game_win_scope_closes_on_error_and_cancellation() {
    for cancelled in [false, true] {
        let root = TestDirectory::new();
        let images = root.images();
        let _restore = DiagnosticsRestore::install(images.clone());
        let (events, _receiver) = sink();
        record_game_win_png(ENTRY_PNG, Some(Duration::from_secs(4)), &events).unwrap();
        let stopped: Result<(), String> = (|| {
            let _scope = begin_game_win_images(&events)?;
            assert!(game_win_images_active());
            wait_or_stop(Duration::ZERO, &AtomicBool::new(cancelled), "injected STOP")?;
            Err("injected transition error".to_owned())
        })();
        assert_eq!(stopped.unwrap_err(), if cancelled { "injected STOP" } else { "injected transition error" });
        assert!(!game_win_images_active());
        record_game_win_png(ORDINARY_PNG, Some(Duration::from_secs(6)), &events).unwrap();
        let paths = png_paths(&images);
        assert_eq!(paths.len(), 1);
        assert_eq!(fs::read(&paths[0]).unwrap(), ENTRY_PNG);
    }
}


/// Cached capture time and queued emission time cannot be replaced by a later UI clock.
#[test]
fn entry_filename_and_worker_event_keep_their_original_timestamps() {
    let root = TestDirectory::new();
    let images = root.images();
    let _restore = DiagnosticsRestore::install(images.clone());
    let (events, receiver) = sink();
    record_game_win_png(ENTRY_PNG, Some(Duration::from_millis(4_321)), &events).unwrap();
    { let _finite_win = begin_game_win_images(&events).unwrap(); }
    assert!(!game_win_images_active());
    send_log(&events, "queued source-time marker".to_owned());

    WORKER_IMAGE_DIAGNOSTICS.with(|slot|
        slot.borrow_mut().as_mut().unwrap().started_at = Instant::now() - Duration::from_secs(100));
    let notifications: Vec<_> = receiver.try_iter().collect();
    let marker = notifications.iter().find(|event| matches!(&event.event,
        WorkerEvent::Log(message) if message == "queued source-time marker")).unwrap();
    let emitted = marker.emitted_at.unwrap();
    assert!(emitted >= Duration::from_secs(5) && emitted < Duration::from_secs(100));
    assert!(worker_elapsed().unwrap() >= Duration::from_secs(100));
    let paths = png_paths(&images);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].file_name().unwrap(), "000004.321_000001.png");
    assert!(notifications.iter().any(|event| matches!(&event.event,
        WorkerEvent::Log(message) if message.starts_with("GAME WIN PNG (entry): capture=+4.321s;"))));
}
