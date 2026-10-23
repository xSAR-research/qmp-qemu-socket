//! Session-only Pyramid evidence names without filesystem creation or guest authority.
//!
//! Existing directories are selected with bounded metadata checks. Write bits are
//! only a preliminary check: the worker's private no-overwrite publication remains
//! authoritative for permissions, races and destination availability.

#![cfg_attr(
    not(test),
    deny(
        clippy::indexing_slicing,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic
    )
)]

use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{RECON_DEAL_ID_MAX_BYTES, reconnaissance_path_is_valid};


/// Maximum candidate metadata probes before generation reports a visible error.
const MAX_NAME_ATTEMPTS: usize = 64;


/// One unreserved destination and its matching explicit deal identity.
pub(super) struct GeneratedEvidence {
    /// Literal UTF-8 pathname, checked against the UI's pathname bounds.
    pub(super) output_path: String,
    /// Bounded ASCII identity containing time, process identity and session serial.
    pub(super) deal_id: String,
}


/// Monotonic session naming state; no generated name is a publication reservation.
pub(super) struct EvidenceDefaults {
    /// Existing real directory selected once without creating or modifying files.
    directory: PathBuf,
    /// Current process identity used to distinguish concurrently opened applications.
    process_id: u32,
    /// Last attempted serial, advanced with checked arithmetic and never reset.
    last_serial: u64,
}


impl EvidenceDefaults {
    /// Prefer an eligible runtime `$HOME/tmp`, then the runtime temporary directory.
    ///
    /// Returns an error when neither is an absolute real directory with write bits.
    /// ACLs, read-only mounts and later filesystem changes remain worker errors.
    pub(super) fn runtime() -> Result<Self, String> {
        let directory = choose_directory(
            std::env::var_os("HOME").map(|value| PathBuf::from(value).join("tmp")),
            std::env::temp_dir(),
            directory_is_eligible,
        )?;
        Ok(Self::in_directory(directory, std::process::id()))
    }


    /// Initialise checked session serials for the already selected directory and process.
    pub(super) fn in_directory(directory: PathBuf, process_id: u32) -> Self {
        Self {
            directory,
            process_id,
            last_serial: 0,
        }
    }


    /// Generate one unused-looking pathname using current UTC epoch milliseconds.
    ///
    /// Only bounded metadata reads occur. Errors cover the clock, exhausted serials,
    /// invalid path/token bounds, unreadable candidates and repeated collisions.
    pub(super) fn generate(&mut self) -> Result<GeneratedEvidence, String> {
        let epoch_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("Cannot generate a Pyramid evidence time: {error}"))?
            .as_millis();
        self.generate_at(epoch_millis, |path| match std::fs::symlink_metadata(path) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(format!(
                "Cannot inspect Pyramid evidence destination {}: {error}",
                path.display()
            )),
        })
    }


    /// Try bounded serials at an explicit time using the caller's existence probe.
    ///
    /// A candidate is returned only after bounds checks and an absent result. Failed
    /// or colliding candidates consume serials; exhaustion never wraps or reuses one.
    fn generate_at(
        &mut self,
        epoch_millis: u128,
        mut exists: impl FnMut(&Path) -> Result<bool, String>,
    ) -> Result<GeneratedEvidence, String> {
        for _ in 0..MAX_NAME_ATTEMPTS {
            self.last_serial = self
                .last_serial
                .checked_add(1)
                .ok_or("Pyramid evidence name serials are exhausted; restart the application.")?;
            let deal_id = format!(
                "pyramid-recon-{epoch_millis}-{}-{}",
                self.process_id, self.last_serial
            );
            if deal_id.len() > RECON_DEAL_ID_MAX_BYTES {
                return Err("Generated Pyramid deal identity exceeds its 64-byte bound.".to_owned());
            }
            let destination = self.directory.join(format!("{deal_id}.json"));
            let output_path = destination.to_str()
                .filter(|value| reconnaissance_path_is_valid(value))
                .ok_or("The automatic evidence directory cannot produce a valid UTF-8 pathname within 4096 bytes.")?
                .to_owned();
            if !exists(&destination)? {
                return Ok(GeneratedEvidence {
                    output_path,
                    deal_id,
                });
            }
        }
        Err("Automatic Pyramid evidence destinations repeatedly exist; choose an unused output path.".to_owned())
    }
}


/// Select the preferred eligible directory, then fallback, without creating either.
///
/// Returns an error if the supplied bounded eligibility checks reject both paths.
fn choose_directory(
    preferred: Option<PathBuf>,
    fallback: PathBuf,
    eligible: impl Fn(&Path) -> bool,
) -> Result<PathBuf, String> {
    if let Some(directory) = preferred.filter(|path| eligible(path)) {
        return Ok(directory);
    }
    if eligible(&fallback) {
        return Ok(fallback);
    }
    Err("No eligible existing evidence directory was found; choose an output path in a writable directory.".to_owned())
}


/// Reject relative paths, symlink directories and metadata without any write bits.
///
/// This preliminary check neither proves access nor reserves a destination.
fn directory_is_eligible(path: &Path) -> bool {
    path.is_absolute()
        && std::fs::symlink_metadata(path).is_ok_and(|metadata| {
            metadata.file_type().is_dir() && !metadata.permissions().readonly()
        })
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Build deterministic naming state without touching the filesystem or environment.
    fn defaults() -> EvidenceDefaults {
        EvidenceDefaults {
            directory: PathBuf::from("/tmp/evidence"),
            process_id: 42,
            last_serial: 0,
        }
    }


    /// Equal clock readings and existing candidates still produce different checked names.
    #[test]
    fn generated_names_are_distinct_and_collisions_consume_serials() {
        let mut defaults = defaults();
        let first = defaults
            .generate_at(1_800_000_000_000, |_| Ok(false))
            .unwrap();
        assert_eq!(first.deal_id, "pyramid-recon-1800000000000-42-1");
        assert!(
            first
                .output_path
                .ends_with("pyramid-recon-1800000000000-42-1.json")
        );
        let mut probes = 0;
        let next = defaults
            .generate_at(1_800_000_000_000, |_| {
                probes += 1;
                Ok(probes < 3)
            })
            .unwrap();
        assert_eq!(probes, 3);
        assert_eq!(next.deal_id, "pyramid-recon-1800000000000-42-4");
        assert_ne!(first.output_path, next.output_path);
    }


    /// Collision and identity exhaustion are bounded refusals, without wraparound.
    #[test]
    fn generated_names_refuse_bounded_collisions_and_serial_exhaustion() {
        let mut defaults = defaults();
        let mut probes = 0;
        assert!(
            defaults
                .generate_at(0, |_| {
                    probes += 1;
                    Ok(true)
                })
                .is_err()
        );
        assert_eq!(probes, MAX_NAME_ATTEMPTS);
        defaults.last_serial = u64::MAX;
        assert!(
            defaults
                .generate_at(0, |_| panic!("exhaustion must precede probing"))
                .is_err()
        );
        assert_eq!(defaults.last_serial, u64::MAX);
    }


    /// Runtime preference falls back explicitly and reports when neither directory qualifies.
    #[test]
    fn directory_selection_prefers_home_tmp_then_runtime_fallback() {
        let preferred = PathBuf::from("/home/operator/tmp");
        let fallback = PathBuf::from("/tmp");
        assert_eq!(
            choose_directory(Some(preferred.clone()), fallback.clone(), |_| true).unwrap(),
            preferred
        );
        assert_eq!(
            choose_directory(Some(preferred), fallback.clone(), |path| path == fallback).unwrap(),
            fallback
        );
        assert!(choose_directory(None, PathBuf::from("/missing"), |_| false).is_err());
    }


    /// Invalid pathname/token bounds fail before an existence probe can authorise a candidate.
    #[test]
    fn generated_names_enforce_path_and_token_bounds() {
        let mut defaults = defaults();
        defaults.directory = PathBuf::from(format!("/{}", "x".repeat(4096)));
        assert!(
            defaults
                .generate_at(0, |_| panic!("invalid path must precede probing"))
                .is_err()
        );
        defaults.directory = PathBuf::from("/tmp");
        defaults.last_serial = u64::MAX - 1;
        assert!(
            defaults
                .generate_at(u128::MAX, |_| panic!("invalid token must precede probing"))
                .is_err()
        );
    }
}
