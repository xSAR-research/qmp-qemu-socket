//! Actual native-v23 normal entry, using isolated input-free QMP observations.

use super::*;


/// Load the original complete normal board refused by application 158.
fn retained() -> Vec<u8> {
    fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pyramid-recognition-159/normal-two.png"),
    )
    .unwrap()
}


/// The actual normal policy requires two complete fresh mock observations before entry.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_159_normal_entry_pairs_supported_current_board_and_header() {
    let episode = run(
        Operation::NormalColourEntry,
        vec![retained(); 2],
        Revocation::None,
    );
    assert_eq!(
        episode.result.unwrap(),
        Entry::Board {
            progress: Some(BoardCount {
                completed: 1,
                target: 3,
            }),
            standard_three: true,
        }
    );
    assert_eq!(episode.wire.captures, 2);
    assert_eq!(episode.wire.remaining, 0);
    assert!(episode.retained_frame && episode.owns_context && !episode.cancelled);
    assert!(
        episode
            .wire
            .commands
            .iter()
            .all(|command| matches!(command.as_str(), "qmp_capabilities" | "screendump"))
    );
}


/// STOP or a context round trip after the first capture revokes any matching partner.
#[test]
#[ignore = "requires local AF_UNIX sockets"]
fn native_159_normal_entry_cannot_outlive_stop_or_context() {
    for revocation in [Revocation::Stop, Revocation::ContextRoundTrip] {
        let episode = run(
            Operation::NormalColourEntry,
            vec![retained(); 2],
            revocation,
        );
        assert!(
            episode
                .result
                .expect_err("revoked first normal observation")
                .contains("STOP or changed context revoked")
        );
        assert_eq!(episode.wire.captures, 1);
        assert_eq!(episode.wire.remaining, 1);
        assert!(episode.retained_frame && episode.cancelled && !episode.owns_context);
        assert!(
            episode
                .wire
                .commands
                .iter()
                .all(|command| matches!(command.as_str(), "qmp_capabilities" | "screendump"))
        );
    }
}
