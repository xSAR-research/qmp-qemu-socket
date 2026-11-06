//! Exact frame/identity retention and private persistence failure boundaries.

use super::*;
use crate::{capture::decode_png, parameters::AnimationSettleDelays};
use std::os::unix::fs::PermissionsExt;

/// Isolated root owned only by this test, never the application's image cache.
struct Root(PathBuf);

impl Root {
    /// Reserve one private directory without replacing existing entries.
    fn new() -> Self {
        let serial = RUN_SERIAL.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "klondike-telemetry-test-{}-{serial}",
            std::process::id()
        ));
        DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Supply independently distinguishable validated response identities.
fn timing(frame_id: u64) -> CaptureTiming {
    CaptureTiming {
        captures: 1,
        identity: Some(super::super::super::CaptureIdentity {
            protocol_version: 3,
            session_id: 41,
            request_id: frame_id + 100,
            frame_id,
            display_generation: 7,
            reused_connection: frame_id > 1,
        }),
        ..CaptureTiming::default()
    }
}

/// Native source shared by all synthetic diagnostic derivatives below.
fn native() -> CapturedFrame {
    decode_png(include_bytes!(
        "../../../../tests/fixtures/klondike/K48.png"
    ))
    .unwrap()
}

/// First rejection is immutable; later raw pixels retain their own identity and no classification.
#[test]
fn telemetry_301_packet_retains_first_and_terminal_pixels_with_exact_identities() {
    let root = Root::new();
    let mut first = native();
    let offset = 710 * first.stride + 874 * 4;
    first.pixels[offset..offset + 3].fill(0);
    let mut last = native();
    last.pixels[..3].copy_from_slice(&[17, 23, 29]);
    let mut trace = RunDiagnostics::new();
    trace.capture(&native(), timing(1), Some(true));
    trace.attempt("Gameplay SUIT 3 source click".to_owned());
    trace.acknowledged(true);
    trace.capture(&first, timing(2), Some(false));
    trace.capture(&last, timing(3), Some(false));
    trace.capture(&native(), timing(4), Some(true));
    trace.capture(&last, timing(5), None);
    trace.attempt("Terminal attempt uncertain".to_owned());
    trace.acknowledged(false);
    let path = trace
        .persist_in(
            &root.0,
            Some(&last),
            None,
            StepRunSettings::new(AnimationSettleDelays::default(), 1),
            None,
            "original scene refusal",
        )
        .unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o700
    );
    for name in ["first-rejected.png", "last-observed.png", "state.json"] {
        assert_eq!(
            fs::metadata(path.join(name)).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    assert_eq!(
        decode_png(&fs::read(path.join("first-rejected.png")).unwrap())
            .unwrap()
            .pixels,
        first.pixels
    );
    assert_eq!(
        decode_png(&fs::read(path.join("last-observed.png")).unwrap())
            .unwrap()
            .pixels,
        last.pixels
    );
    let state: Value = serde_json::from_slice(&fs::read(path.join("state.json")).unwrap()).unwrap();
    assert_eq!(state["original_error"], "original scene refusal");
    assert_eq!(
        state["first_rejected"]["capture"]["transport"]["frame_id"],
        2
    );
    assert_eq!(
        state["last_observed"]["capture"]["transport"]["frame_id"],
        5
    );
    assert_eq!(
        state["last_observed"]["capture"]["classified_gameplay_scene"],
        Value::Null
    );
    assert_eq!(
        state["first_rejected"]["capture"]["last_input"]["acknowledged"],
        true
    );
    assert_eq!(state["last_input"]["acknowledged"], false);
    assert_eq!(
        state["first_rejected"]["scene_probes"]["probes"]
            .as_array()
            .unwrap()
            .len(),
        15
    );
    assert_eq!(state["persistence_warnings"], json!([]));
}

/// Legacy/raw captures never invent TCP identities or a scene rejection.
#[test]
fn telemetry_301_raw_legacy_and_ordinal_exhaustion_do_not_mislabel_pixels() {
    let frame = native();
    let mut trace = RunDiagnostics::new();
    trace.capture(&frame, CaptureTiming::default(), None);
    assert!(trace.first.is_none());
    assert_eq!(
        trace.latest.as_ref().unwrap().value()["transport"],
        Value::Null
    );
    trace.ordinal = u64::MAX;
    trace.capture(&frame, timing(9), Some(false));
    assert!(
        trace.latest.is_none(),
        "new pixels cannot inherit stale identity on overflow"
    );
    assert!(trace.first.is_none());
}

/// Persistence never overwrites a file and bounded failures preserve the original evidence.
#[test]
fn telemetry_301_exclusive_writes_and_partial_packet_failures_are_visible() {
    let root = Root::new();
    let path = root.0.join("already-present");
    fs::write(&path, b"keep").unwrap();
    assert_eq!(
        write_private(&path, b"replace", 20).unwrap_err().kind(),
        io::ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read(&path).unwrap(), b"keep");
    assert!(write_private(&root.0.join("oversized"), b"too large", 2).is_err());
    assert!(!root.0.join("oversized").exists());
    let settings = StepRunSettings::new(AnimationSettleDelays::default(), 1);
    assert!(
        RunDiagnostics::new()
            .persist_in(&path, None, None, settings, None, "original")
            .is_err()
    );
    let mut invalid = native();
    invalid.pixels.clear();
    let mut trace = RunDiagnostics::new();
    trace.capture(&invalid, timing(1), Some(false));
    let error = trace
        .persist_in(
            &root.0,
            Some(&invalid),
            None,
            settings,
            None,
            "original refusal",
        )
        .unwrap_err();
    assert!(error.contains("packet retained"));
    let packet = fs::read_dir(&root.0)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|p| p.is_dir())
        .unwrap();
    let state: Value =
        serde_json::from_slice(&fs::read(packet.join("state.json")).unwrap()).unwrap();
    assert_eq!(state["original_error"], "original refusal");
    assert_eq!(state["persistence_warnings"].as_array().unwrap().len(), 2);
    assert!(!packet.join("first-rejected.png").exists());
}
