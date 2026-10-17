//! Exercise the shared capture boundary against explicit wire modes on local mock sockets.

use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    os::unix::{
        fs::DirBuilderExt,
        net::{UnixListener, UnixStream},
    },
    sync::atomic::AtomicU64,
};

use serde_json::{Value, json};
use xsar::tcp::protocol::v3;
use xsar::tcp::protocol::{FrameHeader, REQUEST_BYTES};

use super::*;

/// Keep concurrent capture tests in distinct, exclusively created private roots.
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

/// Own only the temporary files created by this test.
struct TestDirectory(PathBuf);

impl TestDirectory {
    /// Create a root without replacing any existing path.
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("qmp-tcp-worker-{}-{}",
            std::process::id(), NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }

    /// Provide a private destination for enabled GAME WIN diagnostics.
    fn images(&self) -> PathBuf {
        let path = self.0.join("images");
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Prevent diagnostics settings and cached entry images leaking between test cases.
struct DiagnosticsRestore(Option<WorkerImageDiagnostics>);

impl DiagnosticsRestore {
    fn install(directory: Option<PathBuf>) -> Self {
        let config = ImageDiagnostics {
            started_at: Instant::now() - Duration::from_secs(5), directory,
        };
        Self(WORKER_IMAGE_DIAGNOSTICS.with(|slot|
            slot.replace(Some(WorkerImageDiagnostics::new(config)))))
    }
}

impl Drop for DiagnosticsRestore {
    fn drop(&mut self) {
        WORKER_IMAGE_DIAGNOSTICS.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}

/// Join bounded socket servers after clients close, including during assertion unwinding.
struct ServerJoin(Option<JoinHandle<usize>>);

impl ServerJoin {
    fn finish(mut self) -> usize {
        self.0.take().unwrap().join().expect("capture mock server")
    }
}

impl Drop for ServerJoin {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() { let _ = handle.join(); }
    }
}

/// Capture notifications without running the UI or its frame consumer.
fn sink() -> (WorkerEventSink, Receiver<ContextualWorkerEvent>) {
    let (events, receiver) = mpsc::channel();
    (WorkerEventSink {
        events, latest_frame: LatestFrameSlot::default(), capture_context: Mutex::new(None),
    }, receiver)
}

/// QMP is connected for later input, but TCP acquisition must never issue a screendump.
fn qmp_pair(root: &TestDirectory) -> (QmpClient, ServerJoin) {
    let socket = root.0.join("qmp.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = ServerJoin(Some(thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "QMP handshake accept timed out");
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("QMP handshake accept failed: {error}"),
            }
        };
        stream.set_read_timeout(Some(Duration::from_secs(15))).unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        packet(&mut stream, json!({"QMP": {"version": {"qemu": {"major": 11, "minor": 0, "micro": 0}}, "capabilities": []}}));
        let mut requests = 0;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap() == 0 { break; }
            let request: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["execute"], "qmp_capabilities",
                "TCP captures must perform no QMP acquisition or guest input");
            requests += 1;
            packet(&mut stream, json!({"return": {}, "id": request["id"]}));
        }
        requests
    })));
    (QmpClient::connect(&socket).unwrap(), server)
}

/// Preserve request identity and flush every QMP handshake response.
fn packet(stream: &mut UnixStream, value: Value) {
    serde_json::to_writer(&mut *stream, &value).unwrap();
    stream.write_all(b"\n").unwrap();
    stream.flush().unwrap();
}

/// Emit valid captures or a deliberately interrupted response on separate connections.
enum Reply {
    Frame(CapturedFrame),
    Truncated,
}

/// Bound every connection wait rather than leaving failed tests blocked in accept.
fn accept_tcp(listener: &TcpListener) -> TcpStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((stream, _)) => return stream,
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "TCP capture accept timed out");
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("TCP capture accept failed: {error}"),
        }
    }
}

/// Use the production packed header and Windows BGRA byte order on the actual socket.
fn tcp_server(replies: Vec<Reply>) -> (String, ServerJoin) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    listener.set_nonblocking(true).unwrap();
    let server = ServerJoin(Some(thread::spawn(move || {
        let mut requests = 0;
        for reply in replies {
            let mut stream = accept_tcp(&listener);
            stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            stream.set_write_timeout(Some(Duration::from_secs(3))).unwrap();
            let mut request = [0; REQUEST_BYTES.len()];
            stream.read_exact(&mut request).unwrap();
            assert_eq!(request, REQUEST_BYTES);
            requests += 1;
            match reply {
                Reply::Frame(mut frame) => {
                    assert_eq!(frame.format, PixelFormat::Rgba8);
                    let header = FrameHeader::new(frame.width, frame.height,
                        u16::try_from(frame.stride).unwrap(), requests).unwrap();
                    for pixel in frame.pixels.as_chunks_mut::<4>().0 { pixel.swap(0, 2); }
                    stream.write_all(&header.to_bytes()).unwrap();
                    stream.write_all(&frame.pixels).unwrap();
                }
                Reply::Truncated => {
                    stream.write_all(&FrameHeader::new(1, 1, 4, requests).unwrap().to_bytes()).unwrap();
                    stream.write_all(&[1, 2]).unwrap();
                }
            }
            // The Windows server closes after one response, even for successive captures.
        }
        requests as usize
    })));
    (address, server)
}

/// Distinct channels and rows expose red/blue swaps, row inversion and stride mistakes.
fn coloured_frame() -> CapturedFrame {
    let stride = NOMINAL_FRAME_WIDTH as usize * 4;
    let mut pixels = vec![255; stride * NOMINAL_FRAME_HEIGHT as usize];
    pixels[..16].copy_from_slice(&[
        251, 3, 17, 255, 9, 127, 203, 255, 0, 0, 0, 255, 2, 2, 2, 255,
    ]);
    pixels[stride..stride + 8].copy_from_slice(&[41, 222, 5, 255, 199, 16, 83, 255]);
    CapturedFrame {
        width: NOMINAL_FRAME_WIDTH,
        height: NOMINAL_FRAME_HEIGHT,
        stride,
        format: PixelFormat::Rgba8,
        pixels,
    }
}

/// Compare the application-visible frame contract, including channel order and row layout.
fn assert_frame(actual: &CapturedFrame, expected: &CapturedFrame) {
    assert_eq!((actual.width, actual.height, actual.stride, actual.format),
        (expected.width, expected.height, expected.stride, expected.format));
    assert_eq!(actual.pixels, expected.pixels);
    assert!(actual.is_layout_valid());
}

/// PNG recording order follows capture sequence, independent of filesystem iteration order.
fn png_paths(directory: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(directory).unwrap().map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "png")).collect();
    paths.sort();
    paths
}

/// Ordinary acquisition needs no writable capture directory, PNG codec work or QMP request.
#[test]
fn ordinary_tcp_capture_preserves_rgba_without_png_or_temporary_file() {
    let root = TestDirectory::new();
    let _diagnostics = DiagnosticsRestore::install(None);
    let expected = coloured_frame();
    let (address, tcp) = tcp_server(vec![Reply::Frame(expected.clone())]);
    let _backend = TestCaptureBackend::tcp(address);
    let (mut qmp, qmp_server) = qmp_pair(&root);
    let (events, _receiver) = sink();
    let unavailable_path = root.0.join("not-created/qmp.sock");
    let (frame, timing) = capture_screen(&mut qmp, &unavailable_path, &events).unwrap();
    assert_frame(&frame, &expected);
    assert_eq!(timing.captures, 1);
    assert_eq!(timing.png_encode, Duration::ZERO);
    assert_eq!(timing.reserve, Duration::ZERO);
    assert!(!unavailable_path.parent().unwrap().exists());
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1, "only the QMP socket exists");
    drop(qmp);
    assert_eq!(qmp_server.finish(), 1, "only the QMP capabilities handshake is permitted");
    assert_eq!(tcp.finish(), 1);
}

/// Manual saving encodes the exact acquired pixels without capturing another frame.
#[test]
fn manual_tcp_png_contains_the_same_single_capture() {
    let root = TestDirectory::new();
    let _diagnostics = DiagnosticsRestore::install(None);
    let expected = coloured_frame();
    let (address, tcp) = tcp_server(vec![Reply::Frame(expected.clone())]);
    let _backend = TestCaptureBackend::tcp(address);
    let (mut qmp, qmp_server) = qmp_pair(&root);
    let (events, _receiver) = sink();
    let (frame, png, timing) = capture_screen_with_png(&mut qmp, &root.0.join("qmp.sock"), &events).unwrap();
    assert_frame(&frame, &expected);
    assert_frame(&decode_png(&png).unwrap(), &expected);
    assert_eq!(timing.captures, 1);
    assert!(timing.png_encode > Duration::ZERO);
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
    drop(qmp);
    assert_eq!(qmp_server.finish(), 1);
    assert_eq!(tcp.finish(), 1, "manual PNG generation adds no capture request");
}

/// Native transition fixtures exercise the latest-entry cache and the complete scope boundary.
#[test]
fn tcp_game_win_records_entry_transition_and_ready_without_extra_captures() {
    let root = TestDirectory::new();
    let images = root.images();
    let _diagnostics = DiagnosticsRestore::install(Some(images.clone()));
    let entry = decode_png(include_bytes!("../../tests/fixtures/freecell-FC09.png")).unwrap();
    let transition = decode_png(include_bytes!("../../tests/fixtures/freecell-FC10.png")).unwrap();
    let ready = decode_png(include_bytes!("../../tests/fixtures/freecell-FC02.png")).unwrap();
    let ordinary = decode_png(include_bytes!("../../tests/fixtures/freecell-FC03.png")).unwrap();
    let (address, tcp) = tcp_server(vec![Reply::Frame(ordinary.clone()), Reply::Frame(entry.clone()),
        Reply::Frame(transition.clone()), Reply::Frame(ready.clone()), Reply::Frame(ordinary)]);
    let _backend = TestCaptureBackend::tcp(address);
    let (mut qmp, qmp_server) = qmp_pair(&root);
    let (events, receiver) = sink();
    let socket = root.0.join("qmp.sock");
    capture_screen(&mut qmp, &socket, &events).unwrap();
    capture_screen(&mut qmp, &socket, &events).unwrap();
    assert!(png_paths(&images).is_empty(), "ordinary play only updates the entry candidate");
    {
        let _outer = begin_game_win_images(&events).unwrap();
        assert_eq!(png_paths(&images).len(), 1);
        {
            let _inner = begin_game_win_images(&events).unwrap();
            capture_screen(&mut qmp, &socket, &events).unwrap();
        }
        assert!(game_win_images_active());
        let (actual_ready, _) = capture_screen(&mut qmp, &socket, &events).unwrap();
        assert_frame(&actual_ready, &ready);
        assert!(is_gameplay_scene_for_mode(&actual_ready, GameMode::FreeCell).unwrap());
    }
    assert!(!game_win_images_active());
    assert!(begin_game_win_images(&events).is_err(), "a completed scope must not reuse a stale entry");
    capture_screen(&mut qmp, &socket, &events).unwrap();
    let paths = png_paths(&images);
    assert_eq!(paths.len(), 3, "save exactly entry, transition and readiness; exclude later play");
    for (path, expected) in paths.iter().zip([&entry, &transition, &ready]) {
        assert_frame(&decode_png(&fs::read(path).unwrap()).unwrap(), expected);
    }
    let notifications: Vec<_> = receiver.try_iter().collect();
    let image_logs = notifications.iter().filter(|event| matches!(&event.event,
        WorkerEvent::Log(message) if message.starts_with("GAME WIN PNG ("))).count();
    assert_eq!(image_logs, 3);
    drop(qmp);
    assert_eq!(qmp_server.finish(), 1);
    assert_eq!(tcp.finish(), 5, "recording adds no TCP requests");
}

/// An interrupted frame is reported, closes the scope, and cannot poison the next connection.
#[test]
fn failed_tcp_frame_keeps_entry_evidence_and_next_capture_recovers() {
    let root = TestDirectory::new();
    let images = root.images();
    let _diagnostics = DiagnosticsRestore::install(Some(images.clone()));
    let expected = coloured_frame();
    let (address, tcp) = tcp_server(vec![Reply::Frame(expected.clone()), Reply::Truncated,
        Reply::Frame(expected.clone())]);
    let _backend = TestCaptureBackend::tcp(address);
    let (mut qmp, qmp_server) = qmp_pair(&root);
    let (events, _receiver) = sink();
    let socket = root.0.join("qmp.sock");
    capture_screen(&mut qmp, &socket, &events).unwrap();
    let result = (|| {
        let _scope = begin_game_win_images(&events)?;
        capture_screen(&mut qmp, &socket, &events)
    })();
    assert!(result.unwrap_err().contains("TCP"));
    assert!(!game_win_images_active());
    let (recovered, _) = capture_screen(&mut qmp, &socket, &events).unwrap();
    assert_frame(&recovered, &expected);
    let paths = png_paths(&images);
    assert_eq!(paths.len(), 1, "failed capture and later ordinary play add no saved PNG");
    assert_frame(&decode_png(&fs::read(&paths[0]).unwrap()).unwrap(), &expected);
    drop(qmp);
    assert_eq!(qmp_server.finish(), 1, "TCP failure must not silently fall back to QMP");
    assert_eq!(tcp.finish(), 3);
}


/// Version-3 socket responses retain full native capture semantics in controlled tests.
enum V3Reply {
    /// Complete BGR8 response, owned independently from every preceding image.
    Frame(CapturedFrame),
    /// Valid header followed by too few payload bytes and a closed connection.
    Truncated,
    /// Invalid orientation metadata, with no payload permitted to reach the receiver.
    WrongOrientation,
    /// Valid but uncalibrated small full display; the app must reject it before PNG work.
    NonNative,
    /// Hold a partially sent payload until the receiving run requests STOP.
    WaitForStop(Arc<AtomicBool>),
}


/// Emit the public production version-3 codec on explicitly grouped persistent sessions.
fn v3_server(sessions: Vec<Vec<V3Reply>>) -> (String, ServerJoin) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    listener.set_nonblocking(true).unwrap();
    let server = ServerJoin(Some(thread::spawn(move || {
        let mut total_requests = 0usize;
        for (session_index, replies) in sessions.into_iter().enumerate() {
            let mut stream = accept_tcp(&listener);
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let session_id = session_index as u64 + 1;
            for (request_index, reply) in replies.into_iter().enumerate() {
                let mut request_bytes = [0; v3::REQUEST_LEN];
                stream.read_exact(&mut request_bytes).unwrap();
                let request = v3::Request::from_bytes(request_bytes).unwrap();
                assert_eq!(request.request_id(), request_index as u64 + 1);
                assert_eq!(
                    request.session_id(),
                    if request_index == 0 { 0 } else { session_id }
                );
                assert_eq!(
                    request.display_generation(),
                    if request_index == 0 { 0 } else { 7 }
                );
                total_requests += 1;
                let header = v3::ResponseHeader::success(
                    session_id,
                    request.request_id(),
                    total_requests as u64,
                    7,
                    NOMINAL_FRAME_WIDTH,
                    NOMINAL_FRAME_HEIGHT,
                )
                .unwrap();
                match reply {
                    V3Reply::Frame(frame) => {
                        for fragment in header.to_bytes().chunks(7) {
                            stream.write_all(fragment).unwrap();
                        }
                        let mut bgr = Vec::with_capacity(frame.pixels.len() / 4 * 3);
                        for pixel in frame.pixels.as_chunks::<4>().0 {
                            bgr.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
                        }
                        stream.write_all(&bgr).unwrap();
                    }
                    V3Reply::Truncated => {
                        stream.write_all(&header.to_bytes()).unwrap();
                        stream.write_all(&[1, 2]).unwrap();
                    }
                    V3Reply::WrongOrientation => {
                        let mut bytes = header.to_bytes();
                        bytes[7] = 9;
                        stream.write_all(&bytes).unwrap();
                    }
                    V3Reply::NonNative => {
                        let small = v3::ResponseHeader::success(
                            session_id,
                            request.request_id(),
                            total_requests as u64,
                            7,
                            1,
                            1,
                        )
                        .unwrap();
                        stream.write_all(&small.to_bytes()).unwrap();
                        stream.write_all(&[17, 3, 251]).unwrap();
                    }
                    V3Reply::WaitForStop(cancel) => {
                        stream.write_all(&header.to_bytes()).unwrap();
                        stream.write_all(&[1, 2]).unwrap();
                        cancel.store(true, Ordering::Release);
                        let mut byte = [0];
                        // Closing with unread response bytes can produce either
                        // EOF or a TCP reset. Both prove the session was discarded;
                        // a new request byte, timeout or other error remains a failure.
                        match stream.read(&mut byte) {
                            Ok(0) => {}
                            Err(error) if error.kind() == ErrorKind::ConnectionReset => {}
                            result => panic!(
                                "cancelled capture must discard its partially received session: {result:?}"
                            ),
                        }
                    }
                }
            }
        }
        total_requests
    })));
    (address, server)
}


/// Routine and manual acquisition share one connection and preserve independently owned frames.
#[test]
fn persistent_v3_reuses_connection_for_routine_and_manual_full_capture() {
    let root = TestDirectory::new();
    let _diagnostics = DiagnosticsRestore::install(None);
    let first = coloured_frame();
    let mut second = coloured_frame();
    second.pixels[0] = 19;
    let (address, tcp) = v3_server(vec![vec![
        V3Reply::Frame(first.clone()),
        V3Reply::Frame(second.clone()),
    ]]);
    let _backend = TestCaptureBackend::tcp_mode(address, CaptureMode::PersistentBgrV3);
    let (mut qmp, qmp_server) = qmp_pair(&root);
    let (events, receiver) = sink();
    let socket = root.0.join("qmp.sock");
    let (retained, _) = capture_screen(&mut qmp, &socket, &events).unwrap();
    let (manual, png, _) = capture_screen_with_png(&mut qmp, &socket, &events).unwrap();
    assert_frame(&retained, &first);
    assert_frame(&manual, &second);
    assert_frame(&decode_png(&png).unwrap(), &second);
    assert_eq!(&manual.pixels[8..16], &[0, 0, 0, 255, 2, 2, 2, 255]);
    assert_eq!(manual.pixels.len(), 8_294_400);
    assert!(
        manual
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| pixel[3] == 255)
    );
    let logs: Vec<_> = receiver
        .try_iter()
        .filter_map(|event| match event.event {
            WorkerEvent::Log(message) => Some(message),
            _ => None,
        })
        .collect();
    assert!(
        logs.iter()
            .any(|message| message.contains("session=1; request=1; frame=1;")
                && message.contains("reused=false"))
    );
    assert!(
        logs.iter()
            .any(|message| message.contains("session=1; request=2; frame=2;")
                && message.contains("reused=true"))
    );
    assert!(
        logs.iter()
            .any(|message| message.contains("payload-bytes=6220800"))
    );
    drop(qmp);
    assert_eq!(qmp_server.finish(), 1);
    assert_eq!(tcp.finish(), 2, "manual PNG adds no acquisition request");
}


/// Changing only the controller generation closes the old session before acquiring fresh evidence.
#[test]
fn persistent_v3_context_change_starts_a_new_session() {
    let root = TestDirectory::new();
    let _diagnostics = DiagnosticsRestore::install(None);
    let expected = coloured_frame();
    let (address, tcp) = v3_server(vec![
        vec![V3Reply::Frame(expected.clone())],
        vec![V3Reply::Frame(expected.clone())],
    ]);
    let _backend = TestCaptureBackend::tcp_mode(address, CaptureMode::PersistentBgrV3);
    let (mut qmp, qmp_server) = qmp_pair(&root);
    let (events, receiver) = sink();
    let socket = root.0.join("qmp.sock");
    let mut context = ControllerContext::new(
        socket.clone(),
        GameMode::FreeCell,
        SolvingStrategy::ComputerVision,
        1,
    );
    events.set_capture_context(Some(context.clone()));
    capture_screen(&mut qmp, &socket, &events).unwrap();
    context.generation += 1;
    events.set_capture_context(Some(context));
    let (fresh, _) = capture_screen(&mut qmp, &socket, &events).unwrap();
    assert_frame(&fresh, &expected);
    assert!(receiver.try_iter().any(|event| matches!(event.event,
        WorkerEvent::Log(message) if message.contains("session=2; request=1; frame=2;")
            && message.contains("reused=false"))));
    drop(qmp);
    assert_eq!(qmp_server.finish(), 1);
    assert_eq!(tcp.finish(), 2);
}


/// A failed persistent response is never retried; a later explicit capture may reconnect.
#[test]
fn persistent_v3_truncation_discards_session_and_later_capture_reconnects() {
    let root = TestDirectory::new();
    let _diagnostics = DiagnosticsRestore::install(None);
    let expected = coloured_frame();
    let (address, tcp) = v3_server(vec![
        vec![V3Reply::Frame(expected.clone()), V3Reply::Truncated],
        vec![V3Reply::Frame(expected.clone())],
    ]);
    let _backend = TestCaptureBackend::tcp_mode(address, CaptureMode::PersistentBgrV3);
    let (mut qmp, qmp_server) = qmp_pair(&root);
    let (events, receiver) = sink();
    let socket = root.0.join("qmp.sock");
    capture_screen(&mut qmp, &socket, &events).unwrap();
    assert!(
        capture_screen(&mut qmp, &socket, &events)
            .unwrap_err()
            .contains("TCP")
    );
    let (fresh, _) = capture_screen(&mut qmp, &socket, &events).unwrap();
    assert_frame(&fresh, &expected);
    tcp_capture::reset(Some(&events));
    assert!(receiver.try_iter().any(|event| matches!(event.event,
        WorkerEvent::Log(message) if message.contains("lifetime successes=2; failures=1"))));
    drop(qmp);
    assert_eq!(
        qmp_server.finish(),
        1,
        "capture reconnection never issues guest input"
    );
    assert_eq!(
        tcp.finish(),
        3,
        "one request per explicit capture; no hidden retry"
    );
}


/// Invalid wire metadata or non-native dimensions fail before PNG, preview or analysis.
#[test]
fn v3_invalid_response_never_reaches_png_or_analysis() {
    for reply in [V3Reply::WrongOrientation, V3Reply::NonNative] {
        let root = TestDirectory::new();
        let images = root.images();
        let _diagnostics = DiagnosticsRestore::install(Some(images.clone()));
        let (address, tcp) = v3_server(vec![vec![reply]]);
        let _backend = TestCaptureBackend::tcp_mode(address, CaptureMode::PersistentBgrV3);
        let (mut qmp, qmp_server) = qmp_pair(&root);
        let (events, _receiver) = sink();
        assert!(capture_screen_with_png(&mut qmp, &root.0.join("qmp.sock"), &events).is_err());
        assert!(events.latest_frame.take().is_none());
        assert!(png_paths(&images).is_empty());
        assert!(
            begin_game_win_images(&events).is_err(),
            "no partially valid entry PNG may be cached"
        );
        drop(qmp);
        assert_eq!(qmp_server.finish(), 1);
        assert_eq!(tcp.finish(), 1);
    }
}


/// STOP interrupts an incomplete payload and cannot poison a later explicit manual capture.
#[test]
fn v3_guarded_stop_discards_partial_payload_and_manual_capture_can_reconnect() {
    let root = TestDirectory::new();
    let _diagnostics = DiagnosticsRestore::install(None);
    let cancelled = Arc::new(AtomicBool::new(false));
    let expected = coloured_frame();
    let (address, tcp) = v3_server(vec![
        vec![V3Reply::WaitForStop(Arc::clone(&cancelled))],
        vec![V3Reply::Frame(expected.clone())],
    ]);
    let _backend = TestCaptureBackend::tcp_mode(address, CaptureMode::PersistentBgrV3);
    let (mut qmp, qmp_server) = qmp_pair(&root);
    let (events, _receiver) = sink();
    let socket = root.0.join("qmp.sock");
    {
        let _cancel_scope = tcp_capture::CancellationScope::install(Arc::clone(&cancelled));
        assert!(
            capture_screen(&mut qmp, &socket, &events)
                .unwrap_err()
                .contains("cancelled")
        );
    }
    let (fresh, png, _) = capture_screen_with_png(&mut qmp, &socket, &events).unwrap();
    assert!(
        cancelled.load(Ordering::Acquire),
        "manual capture does not clear the input STOP latch"
    );
    assert_frame(&fresh, &expected);
    assert_frame(&decode_png(&png).unwrap(), &expected);
    drop(qmp);
    assert_eq!(qmp_server.finish(), 1);
    assert_eq!(tcp.finish(), 2);
}


/// The explicit one-shot v3 mode retains BGR8 framing while reconnecting for each capture.
#[test]
fn one_shot_v3_opens_distinct_sessions() {
    let root = TestDirectory::new();
    let _diagnostics = DiagnosticsRestore::install(None);
    let expected = coloured_frame();
    let (address, tcp) = v3_server(vec![
        vec![V3Reply::Frame(expected.clone())],
        vec![V3Reply::Frame(expected.clone())],
    ]);
    let _backend = TestCaptureBackend::tcp_mode(address, CaptureMode::BgrV3);
    let (mut qmp, qmp_server) = qmp_pair(&root);
    let (events, _receiver) = sink();
    let socket = root.0.join("qmp.sock");
    for _ in 0..2 {
        let (actual, _) = capture_screen(&mut qmp, &socket, &events).unwrap();
        assert_frame(&actual, &expected);
    }
    drop(qmp);
    assert_eq!(qmp_server.finish(), 1);
    assert_eq!(tcp.finish(), 2);
}
