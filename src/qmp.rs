//! Bounded-time QMP command transport and release-safe guest input over a Unix socket.
//!
//! Requests carry numeric identifiers; asynchronous events do not satisfy a
//! request. Failed input receives release-only recovery, never a replayed press.

use std::{
    io::{BufRead, BufReader, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use serde_json::{Value, json};
use thiserror::Error;

use crate::{
    geometry::{GeometryError, PixelPoint, pixel_to_qmp_axis},
    parameters::{
        KEY_HOLD, MOUSE_HOLD, POINTER_SETTLE_DELAY, QMP_IO_TIMEOUT, QMP_SCREENDUMP_TIMEOUT,
    },
};


/// Transport, protocol, input-recovery and capture-path failures.
#[derive(Debug, Error)]
pub enum QmpError {
    /// The Unix stream could not complete an I/O operation.
    #[error("QMP socket I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// A received packet is not valid JSON.
    #[error("invalid JSON from QMP: {0}")]
    Json(#[from] serde_json::Error),
    /// A packet or connection state violates the expected QMP exchange.
    #[error("invalid QMP protocol response: {0}")]
    Protocol(String),
    /// QEMU returned an explicit command error.
    #[error("QEMU rejected the command: {0}")]
    Command(String),
    /// The requested PNG path is not an absolute, UTF-8, NUL-free path.
    #[error("invalid screendump output path: {0}")]
    CapturePath(String),
    /// The input action failed and a fresh connection could not confirm release.
    #[error("{action}; a fresh QMP connection could not confirm the safety release: {recovery}")]
    InputRecovery {
        /// Original input failure, retained as the causal error.
        action: Box<QmpError>,
        /// Failure encountered while confirming a release on a fresh connection.
        recovery: Box<QmpError>,
    },
    /// The guest input coordinates cannot be mapped into the declared frame.
    #[error(transparent)]
    Geometry(#[from] GeometryError),
}


/// Raw status and pointer information obtained from QEMU before automation.
#[derive(Clone, Debug)]
pub struct QmpProbe {
    /// Response payload from query-status.
    status: Value,
    /// Response payload from query-mice.
    mice: Value,
}


impl QmpProbe {


    /// Returns QEMU's reported run-state label, or "unknown" when absent.
    pub fn run_state(&self) -> &str {
        self.status
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    }


    /// Reports whether QEMU explicitly says the virtual machine is running.
    pub fn is_running(&self) -> bool {
        self.status.get("running").and_then(Value::as_bool) == Some(true)
    }


    /// Finds the name of the active pointing device when it uses absolute input.
    pub fn current_absolute_pointer_name(&self) -> Option<&str> {
        self.mice.as_array()?.iter().find_map(|mouse| {
            let current = mouse.get("current").and_then(Value::as_bool) == Some(true);
            let absolute = mouse.get("absolute").and_then(Value::as_bool) == Some(true);

            (current && absolute)
                .then(|| mouse.get("name").and_then(Value::as_str))
                .flatten()
        })
    }


    /// Summarises the VM state and active absolute pointer for user-facing logs.
    pub fn summary(&self) -> String {


        match self.current_absolute_pointer_name() {
            Some(pointer) => format!(
                "QEMU state={} | active absolute pointer={pointer}",
                self.run_state()
            ),
            None => format!(
                "QEMU state={} | no active absolute pointer was reported",
                self.run_state()
            ),
        }
    }
}


/// Minimal newline-delimited JSON client for a local QEMU QMP Unix socket.
///
/// It intentionally exposes only the commands needed by this application.
pub struct QmpClient {
    /// Buffered response reader sharing the connected Unix socket.
    reader: BufReader<UnixStream>,
    /// Request writer sharing the connected Unix socket.
    writer: UnixStream,
    /// Socket location reused only for release recovery.
    socket_path: PathBuf,
    /// Next checked numeric request identifier.
    next_id: u64,
    /// Whether this connection is still permitted to send normal requests.
    usable: bool,
}


impl QmpClient {


    /// Opens a QMP socket, validates its greeting, and enables QMP commands.
    pub fn connect(path: &Path) -> Result<Self, QmpError> {
        let writer = UnixStream::connect(path)?;
        Self::from_connected_stream(path, writer)
    }


    /// Configures an already-connected stream, validates QMP, and enables commands.
    fn from_connected_stream(path: &Path, writer: UnixStream) -> Result<Self, QmpError> {
        writer.set_read_timeout(Some(QMP_IO_TIMEOUT))?;
        writer.set_write_timeout(Some(QMP_IO_TIMEOUT))?;

        let reader_stream = writer.try_clone()?;
        reader_stream.set_read_timeout(Some(QMP_IO_TIMEOUT))?;

        let mut client = Self {
            reader: BufReader::new(reader_stream),
            writer,
            socket_path: path.to_path_buf(),
            next_id: 1,
            usable: true,
        };

        let greeting = client.read_packet()?;


        if greeting.get("QMP").is_none() {
            return Err(QmpError::Protocol(
                "the socket did not begin with a QMP greeting".to_owned(),
            ));
        }

        client.execute("qmp_capabilities", None)?;
        Ok(client)
    }


    /// Queries the VM status and available pointing devices.
    pub fn probe(&mut self) -> Result<QmpProbe, QmpError> {
        let status = self.execute("query-status", None)?;
        let mice = self.execute("query-mice", None)?;
        Ok(QmpProbe { status, mice })
    }


    /// Writes the primary display's implicit head 0 to an absolute PNG path.
    ///
    /// This is a read-only guest operation. It sends no pointer or keyboard
    /// events and returns only after QEMU acknowledges the screendump command.
    pub fn screendump_png(&mut self, output_path: &Path) -> Result<(), QmpError> {
        let filename = qmp_capture_filename(output_path)?;
        self.reader
            .get_ref()
            .set_read_timeout(Some(QMP_SCREENDUMP_TIMEOUT))?;
        let capture_result = self
            .execute(
                "screendump",
                Some(json!({
                    "filename": filename,
                    "format": "png"
                })),
            )
            .map(|_| ());
        let restore_result = self.reader.get_ref().set_read_timeout(Some(QMP_IO_TIMEOUT));


        match capture_result {
            Err(error) => {
                let _ = restore_result;
                Err(error)
            }
            Ok(()) => {
                restore_result?;
                Ok(())
            }
        }
    }


    /// Sends a normal-duration absolute left click through QMP.
    pub fn click(
        &mut self,
        guest_point: PixelPoint,
        guest_width: u32,
        guest_height: u32,
    ) -> Result<(), QmpError> {
        self.click_with_hold(guest_point, guest_width, guest_height, MOUSE_HOLD)
    }


    /// Converts a guest pixel position and sends one absolute left click
    /// using the caller-selected button-down duration.
    pub fn click_with_hold(
        &mut self,
        guest_point: PixelPoint,
        guest_width: u32,
        guest_height: u32,
        hold: Duration,
    ) -> Result<(), QmpError> {
        let pixel_x = u32::try_from(guest_point.x).map_err(|_| {
            QmpError::Protocol(format!("negative guest X coordinate: {}", guest_point.x))
        })?;
        let pixel_y = u32::try_from(guest_point.y).map_err(|_| {
            QmpError::Protocol(format!("negative guest Y coordinate: {}", guest_point.y))
        })?;
        let x = pixel_to_qmp_axis(pixel_x, guest_width)?;
        let y = pixel_to_qmp_axis(pixel_y, guest_height)?;

        // Movement is its own synchronised QMP command. Waiting for its ACK
        // prevents the button transition from racing the absolute position.
        self.execute(
            "input-send-event",
            Some(json!({
                "events": [
                    { "type": "abs", "data": { "axis": "x", "value": x } },
                    { "type": "abs", "data": { "axis": "y", "value": y } }
                ]
            })),
        )?;
        thread::sleep(POINTER_SETTLE_DELAY);

        self.send_press_release(
            json!({
                "events": [
                    { "type": "btn", "data": { "button": "left", "down": true } }
                ]
            }),
            json!({
                "events": [
                    { "type": "btn", "data": { "button": "left", "down": false } }
                ]
            }),
            hold,
        )
    }


    /// Sends a held D-key press and release through QMP.
    pub fn press_draw_key(&mut self) -> Result<(), QmpError> {
        self.send_press_release(
            json!({
                "events": [
                    {
                        "type": "key",
                        "data": {
                            "down": true,
                            "key": { "type": "qcode", "data": "d" }
                        }
                    }
                ]
            }),
            json!({
                "events": [
                    {
                        "type": "key",
                        "data": {
                            "down": false,
                            "key": { "type": "qcode", "data": "d" }
                        }
                    }
                ]
            }),
            KEY_HOLD,
        )
    }


    /// Sends one identified QMP command and returns its matching response payload.
    fn execute(&mut self, command: &str, arguments: Option<Value>) -> Result<Value, QmpError> {
        let id = self.send_request(command, arguments)?;
        let mut responses = self.await_responses(&[id])?;
        Ok(responses.remove(0))
    }


    /// Writes one identified request without waiting for its response.
    fn send_request(&mut self, command: &str, arguments: Option<Value>) -> Result<u64, QmpError> {


        if !self.usable {
            return Err(QmpError::Protocol(
                "the QMP connection is no longer usable".to_owned(),
            ));
        }

        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| QmpError::Protocol("QMP request id exhausted".to_owned()))?;

        let mut packet = json!({
            "execute": command,
            "id": id,
        });


        if let Some(arguments) = arguments {
            packet["arguments"] = arguments;
        }

        serde_json::to_writer(&mut self.writer, &packet)?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()?;
        Ok(id)
    }


    /// Collects successful responses in request order. A command error is
    /// returned immediately so release recovery retains the causal failure.
    fn await_responses(&mut self, ids: &[u64]) -> Result<Vec<Value>, QmpError> {
        let mut responses = vec![None; ids.len()];


        while responses.iter().any(Option::is_none) {
            let response = self.read_packet()?;


            if response.get("event").is_some() && response.get("id").is_none() {
                // QMP events are asynchronous and are unrelated to these requests.
                continue;
            }

            let response_id = response.get("id").and_then(Value::as_u64).ok_or_else(|| {
                QmpError::Protocol("response has no numeric request id".to_owned())
            })?;
            let response_index = ids
                .iter()
                .position(|id| *id == response_id)
                .ok_or_else(|| {
                    QmpError::Protocol(format!("response has unexpected request id {response_id}"))
                })?;


            if let Some(error) = response.get("error") {
                let description = error
                    .get("desc")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown QMP command error");
                return Err(QmpError::Command(description.to_owned()));
            }
            let payload = response
                .get("return")
                .cloned()
                .ok_or_else(|| QmpError::Protocol("response has no return value".to_owned()))?;


            if responses[response_index].replace(payload).is_some() {
                return Err(QmpError::Protocol(format!(
                    "duplicate response for request id {response_id}"
                )));
            }
        }

        Ok(responses
            .into_iter()
            .map(|response| response.expect("all requested QMP responses were collected"))
            .collect())
    }


    /// Never wait for the down response before transmitting the matching up.
    fn send_press_release(
        &mut self,
        down_arguments: Value,
        up_arguments: Value,
        hold: Duration,
    ) -> Result<(), QmpError> {


        let down_id = match self.send_request("input-send-event", Some(down_arguments)) {
            Ok(id) => id,
            Err(error) => return Err(self.recover_release(up_arguments, error)),
        };

        thread::sleep(hold);


        let up_id = match self.send_request("input-send-event", Some(up_arguments.clone())) {
            Ok(id) => id,
            Err(error) => return Err(self.recover_release(up_arguments, error)),
        };


        match self.await_responses(&[down_id, up_id]) {
            Ok(_) => Ok(()),
            Err(error) => Err(self.recover_release(up_arguments, error)),
        }
    }


    /// A failed write or ACK leaves the guest's input state uncertain. Send
    /// only an idempotent release on the old stream, then close it and
    /// confirm the release once on a fresh QMP connection. Never retry down.
    fn recover_release(&mut self, up_arguments: Value, action: QmpError) -> QmpError {
        let _ = self.send_request("input-send-event", Some(up_arguments.clone()));
        self.usable = false;
        let _ = self.writer.shutdown(Shutdown::Both);
        let _ = self.reader.get_ref().shutdown(Shutdown::Both);

        let recovery = Self::connect(&self.socket_path).and_then(|mut fresh| {
            fresh
                .execute("input-send-event", Some(up_arguments))
                .map(|_| ())
        });


        match recovery {
            Ok(()) => action,
            Err(recovery) => QmpError::InputRecovery {
                action: Box::new(action),
                recovery: Box::new(recovery),
            },
        }
    }


    /// Reads and decodes one newline-delimited JSON packet from QMP.
    fn read_packet(&mut self) -> Result<Value, QmpError> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line)?;


        if bytes_read == 0 {
            return Err(QmpError::Protocol("QMP socket closed".to_owned()));
        }

        Ok(serde_json::from_str(&line)?)
    }
}


/// Validate a capture path before serialising it into a screendump request.
fn qmp_capture_filename(path: &Path) -> Result<&str, QmpError> {


    if !path.is_absolute() {
        return Err(QmpError::CapturePath(
            "the path must be absolute".to_owned(),
        ));
    }

    let filename = path
        .to_str()
        .ok_or_else(|| QmpError::CapturePath("the path must contain valid UTF-8".to_owned()))?;


    if filename.contains('\0') {
        return Err(QmpError::CapturePath(
            "the path must not contain a NUL byte".to_owned(),
        ));
    }

    Ok(filename)
}


#[cfg(test)]
mod tests {
    //! Mock transport checks for capture arguments and release-safe input ordering.
    use std::{
        ffi::OsString,
        fs,
        io::{ErrorKind, Read},
        os::unix::ffi::OsStringExt,
        os::unix::net::UnixListener,
        process,
        sync::atomic::{AtomicU64, Ordering},
        time::{Duration, Instant},
    };

    use super::*;


    /// Process-local suffix used to keep concurrent mock socket paths distinct.
    static NEXT_SOCKET_ID: AtomicU64 = AtomicU64::new(1);


    /// Filesystem pathname guard for a mock Unix socket.
    struct SocketFile(
        /// Socket path removed when the guard is dropped.
        PathBuf,
    );


    impl Drop for SocketFile {


        /// Remove the mock socket pathname when the test guard leaves scope.
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }


    /// Write and flush one newline-delimited JSON packet from the mock server.
    fn write_packet(stream: &mut UnixStream, packet: &Value) {
        serde_json::to_writer(&mut *stream, packet).expect("serialize mock QMP packet");
        stream.write_all(b"\n").expect("write mock QMP newline");
        stream.flush().expect("flush mock QMP packet");
    }


    /// Read and decode a complete request from the mock QMP client.
    fn read_request(reader: &mut BufReader<UnixStream>) -> Value {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line).expect("read mock QMP request");
        assert_ne!(bytes, 0, "mock QMP client closed unexpectedly");
        serde_json::from_str(&line).expect("decode mock QMP request")
    }


    /// Extract the numeric request identifier required by mock responses.
    fn request_id(request: &Value) -> u64 {
        request
            .get("id")
            .and_then(Value::as_u64)
            .expect("request has numeric id")
    }


    /// Send an empty successful response for the selected request.
    fn acknowledge(stream: &mut UnixStream, request: &Value) {
        write_packet(stream, &json!({ "return": {}, "id": request_id(request) }));
    }


    /// Exchange the greeting and capability negotiation with a test client.
    fn start_mock_qmp(stream: &mut UnixStream, reader: &mut BufReader<UnixStream>) {
        write_packet(
            stream,
            &json!({
                "QMP": {
                    "version": { "qemu": { "major": 11, "minor": 1, "micro": 1 } },
                    "capabilities": []
                }
            }),
        );
        let capabilities = read_request(reader);
        assert_eq!(capabilities["execute"], "qmp_capabilities");
        acknowledge(stream, &capabilities);
    }


    /// Accept a mock connection within a fixed deadline.
    fn accept_with_timeout(listener: &UnixListener) -> UnixStream {
        let deadline = Instant::now() + Duration::from_secs(2);


        loop {


            match listener.accept() {
                Ok((stream, _)) => return stream,
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "timed out waiting for QMP client"
                    );
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept mock QMP client: {error}"),
            }
        }
    }


    /// Require one input event per press or release request and return it.
    fn first_input_event(request: &Value) -> &Value {
        assert_eq!(request["execute"], "input-send-event");
        let events = request["arguments"]["events"]
            .as_array()
            .expect("input-send-event has an events array");
        assert_eq!(events.len(), 1, "down and up must use separate commands");
        &events[0]
    }


    /// Verifies that only an explicit running status passes the probe check.
    #[test]
    fn probe_requires_qemu_to_report_running() {
        let running = QmpProbe {
            status: json!({ "running": true, "status": "running" }),
            mice: json!([]),
        };
        let paused = QmpProbe {
            status: json!({ "running": false, "status": "paused" }),
            mice: json!([]),
        };

        assert!(running.is_running());
        assert!(!paused.is_running());
    }


    /// Verifies that the probe selects the current absolute pointing device.
    #[test]
    fn probe_accepts_only_the_current_absolute_pointer() {
        let probe = QmpProbe {
            status: json!({ "running": true, "status": "running" }),
            mice: json!([
                { "name": "relative", "current": false, "absolute": false },
                { "name": "tablet", "current": true, "absolute": true }
            ]),
        };

        assert_eq!(probe.current_absolute_pointer_name(), Some("tablet"));
    }


    /// Verify PNG command arguments and reject invalid paths before any request.
    #[test]
    fn screendump_uses_png_and_rejects_invalid_paths_before_sending() {
        let (client_stream, mut server_stream) = UnixStream::pair().expect("create socket pair");
        server_stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set mock server timeout");
        let server_reader = server_stream.try_clone().expect("clone mock server");

        let server = thread::spawn(move || {
            let mut reader = BufReader::new(server_reader);
            start_mock_qmp(&mut server_stream, &mut reader);

            // Invalid calls must not put requests on the wire, so the next
            // packet received here must be the one valid screendump request.
            let request = read_request(&mut reader);
            assert_eq!(request["execute"], "screendump");
            assert_eq!(
                request["arguments"],
                json!({
                    "filename": "/tmp/qmp-qemu-socket-frame.png",
                    "format": "png"
                })
            );
            acknowledge(&mut server_stream, &request);
        });

        let mut client = QmpClient::from_connected_stream(Path::new("mock-qmp"), client_stream)
            .expect("connect mock client");
        let relative = client
            .screendump_png(Path::new("frame.png"))
            .expect_err("relative output path must fail");
        assert!(matches!(relative, QmpError::CapturePath(_)));

        let non_utf8 = PathBuf::from(OsString::from_vec(vec![b'/', b't', b'm', b'p', b'/', 0xff]));
        let non_utf8 = client
            .screendump_png(&non_utf8)
            .expect_err("non-UTF-8 output path must fail");
        assert!(matches!(non_utf8, QmpError::CapturePath(_)));

        client
            .screendump_png(Path::new("/tmp/qmp-qemu-socket-frame.png"))
            .expect("send screendump command");
        drop(client);
        server.join().expect("mock QMP server completed");
    }


    /// Verify movement ordering and release delivery before either input acknowledgement.
    #[test]
    fn click_and_key_send_release_before_waiting_for_down_ack() {
        let (client_stream, mut server_stream) = UnixStream::pair().expect("create socket pair");
        server_stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set mock server timeout");
        let server_reader = server_stream.try_clone().expect("clone mock server");

        let server = thread::spawn(move || {
            let mut reader = BufReader::new(server_reader);
            start_mock_qmp(&mut server_stream, &mut reader);

            let movement = read_request(&mut reader);
            assert_eq!(movement["execute"], "input-send-event");
            assert_eq!(
                movement["arguments"]["events"],
                json!([
                    {
                        "type": "abs",
                        "data": {
                            "axis": "x",
                            "value": pixel_to_qmp_axis(400, 1920).unwrap()
                        }
                    },
                    {
                        "type": "abs",
                        "data": {
                            "axis": "y",
                            "value": pixel_to_qmp_axis(800, 1080).unwrap()
                        }
                    }
                ])
            );

            // Withhold the movement ACK for longer than the settle delay. A
            // client that does not wait for it will reveal a premature down.
            reader
                .get_ref()
                .set_read_timeout(Some(POINTER_SETTLE_DELAY + Duration::from_millis(30)))
                .expect("set movement gate timeout");
            let mut premature = String::new();
            let error = reader
                .read_line(&mut premature)
                .expect_err("button-down arrived before movement ACK");
            assert!(matches!(
                error.kind(),
                ErrorKind::WouldBlock | ErrorKind::TimedOut
            ));
            assert!(premature.is_empty());
            reader
                .get_ref()
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("restore mock server timeout");
            acknowledge(&mut server_stream, &movement);

            let button_down = read_request(&mut reader);
            assert_eq!(first_input_event(&button_down)["type"], "btn");
            assert_eq!(first_input_event(&button_down)["data"]["button"], "left");
            assert_eq!(first_input_event(&button_down)["data"]["down"], true);

            // No response is sent for down. Receiving up proves the client did
            // not block waiting for that ACK.
            let button_up = read_request(&mut reader);
            assert_eq!(first_input_event(&button_up)["type"], "btn");
            assert_eq!(first_input_event(&button_up)["data"]["button"], "left");
            assert_eq!(first_input_event(&button_up)["data"]["down"], false);

            // An asynchronous event and reverse response order must not make
            // the client discard either requested response.
            write_packet(
                &mut server_stream,
                &json!({ "event": "RESET", "timestamp": { "seconds": 0, "microseconds": 0 } }),
            );
            acknowledge(&mut server_stream, &button_up);
            acknowledge(&mut server_stream, &button_down);

            let key_down = read_request(&mut reader);
            assert_eq!(first_input_event(&key_down)["type"], "key");
            assert_eq!(first_input_event(&key_down)["data"]["down"], true);
            assert_eq!(
                first_input_event(&key_down)["data"]["key"],
                json!({ "type": "qcode", "data": "d" })
            );

            let key_up = read_request(&mut reader);
            assert_eq!(first_input_event(&key_up)["type"], "key");
            assert_eq!(first_input_event(&key_up)["data"]["down"], false);
            assert_eq!(
                first_input_event(&key_up)["data"]["key"],
                json!({ "type": "qcode", "data": "d" })
            );
            acknowledge(&mut server_stream, &key_down);
            acknowledge(&mut server_stream, &key_up);
        });

        let mut client = QmpClient::from_connected_stream(Path::new("mock-qmp"), client_stream)
            .expect("connect mock client");
        client
            .click(PixelPoint::new(400, 800), 1920, 1080)
            .expect("send click");
        client.press_draw_key().expect("send D key");
        drop(client);
        server.join().expect("mock QMP server completed");
    }


    /// Verify causal-error retention and release-only recovery without a second press.
    #[test]
    fn command_failure_releases_on_original_and_fresh_connections_without_retrying_down() {
        let socket_file = SocketFile(std::env::temp_dir().join(format!(
            "qmp-qemu-socket-test-{}-{}.sock",
            process::id(),
            NEXT_SOCKET_ID.fetch_add(1, Ordering::Relaxed)
        )));
        let _ = fs::remove_file(&socket_file.0);
        let listener = UnixListener::bind(&socket_file.0).expect("bind mock QMP socket");
        listener
            .set_nonblocking(true)
            .expect("make mock listener nonblocking");

        let server = thread::spawn(move || {
            let mut first_stream = accept_with_timeout(&listener);
            first_stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("set original connection timeout");
            let first_reader = first_stream.try_clone().expect("clone original stream");
            let mut first_reader = BufReader::new(first_reader);
            start_mock_qmp(&mut first_stream, &mut first_reader);

            let key_down = read_request(&mut first_reader);
            assert_eq!(first_input_event(&key_down)["data"]["down"], true);

            // Still withhold the down response until the release is present on
            // the wire. The action itself must occur exactly once.
            let key_up = read_request(&mut first_reader);
            assert_eq!(first_input_event(&key_up)["data"]["down"], false);
            write_packet(
                &mut first_stream,
                &json!({
                    "error": { "class": "GenericError", "desc": "injected down failure" },
                    "id": request_id(&key_down)
                }),
            );
            // Do not acknowledge up. The client must retain the causal down
            // error and proceed directly to release-only recovery.

            let original_release = read_request(&mut first_reader);
            assert_eq!(first_input_event(&original_release)["data"]["down"], false);

            // recover_release shuts the uncertain stream before connecting a
            // fresh client, so the original peer must now reach EOF.
            let mut trailing = Vec::new();
            first_reader
                .read_to_end(&mut trailing)
                .expect("read original connection EOF");
            assert!(trailing.is_empty());
            drop(first_reader);
            drop(first_stream);

            let mut fresh_stream = accept_with_timeout(&listener);
            fresh_stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("set fresh connection timeout");
            let fresh_reader = fresh_stream.try_clone().expect("clone fresh stream");
            let mut fresh_reader = BufReader::new(fresh_reader);
            start_mock_qmp(&mut fresh_stream, &mut fresh_reader);

            let fresh_release = read_request(&mut fresh_reader);
            assert_eq!(first_input_event(&fresh_release)["type"], "key");
            assert_eq!(first_input_event(&fresh_release)["data"]["down"], false);
            assert_eq!(
                first_input_event(&fresh_release)["data"]["key"],
                json!({ "type": "qcode", "data": "d" })
            );
            acknowledge(&mut fresh_stream, &fresh_release);
        });

        let mut client = QmpClient::connect(&socket_file.0).expect("connect original QMP client");
        let error = client
            .press_draw_key()
            .expect_err("injected down error must fail the action");
        assert!(matches!(
            error,
            QmpError::Command(ref message) if message == "injected down failure"
        ));

        server.join().expect("mock QMP recovery server completed");
    }
}
