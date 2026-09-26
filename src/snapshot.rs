use std::{
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use crate::parameters::{SNAPSHOT_LABEL_MAX_CHARS, SNAPSHOT_MAX_PNG_BYTES, SNAPSHOT_NAME_ATTEMPTS};

pub struct SnapshotArtifact {
    path: PathBuf,
    file: Option<File>,
    committed: bool,
}

impl SnapshotArtifact {
    pub fn reserve(directory: &Path, label: &str) -> Result<Self, String> {
        if !directory.is_dir() {
            return Err(format!(
                "snapshot directory does not exist: {}",
                directory.display()
            ));
        }
        let timestamp = local_timestamp()?;
        let label = safe_label(label);
        for attempt in 0..SNAPSHOT_NAME_ATTEMPTS {
            let path = directory.join(snapshot_filename(&timestamp, &label, attempt));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
            {
                Ok(file) => {
                    return Ok(Self {
                        path,
                        file: Some(file),
                        committed: false,
                    });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "could not reserve a snapshot in {}: {error}",
                        directory.display()
                    ));
                }
            }
        }
        Err(format!(
            "could not reserve a unique snapshot name after {SNAPSHOT_NAME_ATTEMPTS} attempts"
        ))
    }

    pub fn save(mut self, png: &[u8]) -> Result<PathBuf, String> {
        if png.is_empty() || png.len() > SNAPSHOT_MAX_PNG_BYTES {
            return Err(format!(
                "captured PNG must be between 1 and {SNAPSHOT_MAX_PNG_BYTES} bytes"
            ));
        }
        let file = self
            .file
            .as_mut()
            .ok_or("snapshot reservation has no file")?;
        file.write_all(png)
            .and_then(|()| file.sync_all())
            .map_err(|error| {
                format!("could not finish snapshot {}: {error}", self.path.display())
            })?;
        self.committed = true;
        Ok(self.path.clone())
    }
}

impl Drop for SnapshotArtifact {
    fn drop(&mut self) {
        if !self.committed {
            drop(self.file.take());
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn safe_label(raw: &str) -> String {
    let mut label = String::new();
    for ch in raw.chars().take(SNAPSHOT_LABEL_MAX_CHARS) {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            label.push(ch);
        } else if !label.is_empty() && !label.ends_with('-') {
            label.push('-');
        }
    }
    label.trim_end_matches('-').to_owned()
}

fn snapshot_filename(timestamp: &str, label: &str, attempt: usize) -> String {
    let name = if label.is_empty() {
        format!("qmp-qemu-socket {timestamp}")
    } else {
        format!("qmp-qemu-socket {timestamp} {label}")
    };
    if attempt == 0 {
        format!("{name}.png")
    } else {
        format!("{name}-{attempt:02}.png")
    }
}

fn local_timestamp() -> Result<String, String> {
    // Use the host's configured local timezone; Rust's standard library has
    // no local civil-time formatter. No user text is passed to the process.
    let output = Command::new("/usr/bin/date")
        .arg("+%y%m%d %H%M%S")
        .output()
        .map_err(|error| format!("could not obtain the local date: {error}"))?;
    if !output.status.success() {
        return Err(format!("local date command failed: {}", output.status));
    }
    let timestamp = String::from_utf8(output.stdout)
        .map_err(|error| format!("local date command returned invalid UTF-8: {error}"))?
        .trim()
        .to_owned();
    let bytes = timestamp.as_bytes();
    if bytes.len() != 13
        || bytes[6] != b' '
        || !bytes[..6].iter().all(u8::is_ascii_digit)
        || !bytes[7..].iter().all(u8::is_ascii_digit)
    {
        return Err("local date command returned an unexpected timestamp".to_owned());
    }
    Ok(timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn names_are_bounded_and_collision_safe() {
        assert_eq!(safe_label("  B1 / Move, left  "), "B1-Move-left");
        assert!(
            safe_label("../\\danger")
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        );
        assert!(safe_label(&"a".repeat(100)).len() <= SNAPSHOT_LABEL_MAX_CHARS);
        assert_eq!(
            snapshot_filename("260924 223501", "B1-Move", 0),
            "qmp-qemu-socket 260924 223501 B1-Move.png"
        );
        assert_eq!(
            snapshot_filename("260924 223501", "B1-Move", 1),
            "qmp-qemu-socket 260924 223501 B1-Move-01.png"
        );
    }

    #[test]
    fn saved_png_bytes_are_unchanged_and_existing_names_are_never_replaced() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "qmp-snapshot-test-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();

        let first = SnapshotArtifact::reserve(&directory, "B1 Move").unwrap();
        let first_path = first.save(b"original QMP PNG bytes").unwrap();
        let second = SnapshotArtifact::reserve(&directory, "B1 Move").unwrap();
        let second_path = second.save(b"different PNG bytes").unwrap();
        assert_ne!(first_path, second_path);
        assert_eq!(fs::read(first_path).unwrap(), b"original QMP PNG bytes");
        assert_eq!(fs::read(second_path).unwrap(), b"different PNG bytes");

        let incomplete = SnapshotArtifact::reserve(&directory, "unused").unwrap();
        let incomplete_path = incomplete.path.clone();
        drop(incomplete);
        assert!(!incomplete_path.exists());
        fs::remove_dir_all(directory).unwrap();
    }
}
