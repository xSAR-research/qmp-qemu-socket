//! Private, file-backed session history independent of the visible output panel.
//!
//! Files use exclusive creation and the configured Unix permissions. The UI owns
//! rollover and error reporting; clearing rendered lines never truncates this file.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::parameters::{
    SESSION_LOG_FILE_PREFIX, SESSION_LOG_FILE_SUFFIX, SESSION_LOG_MODE, SESSION_LOG_NAME_ATTEMPTS,
    session_log_directory,
};

/// Owns one uniquely reserved session file for append and complete-history reads.
pub(crate) struct SessionLog {
    /// Destination retained for status display and complete-history reads.
    path: PathBuf,
    /// Open write handle; reads use the pathname after flushing this handle.
    file: File,
}

impl SessionLog {
    /// Reserves a private session file in the configured log directory.
    ///
    /// Existing names are never overwritten. A bounded suffix search handles
    /// timestamp collisions; directory, clock and file-creation errors propagate.
    pub(crate) fn create() -> io::Result<Self> {
        let directory = session_log_directory();
        fs::create_dir_all(&directory)?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_millis();

        for attempt in 0..SESSION_LOG_NAME_ATTEMPTS {
            let collision_suffix = if attempt == 0 {
                String::new()
            } else {
                format!("-{attempt}")
            };
            let path = directory.join(format!(
                "{SESSION_LOG_FILE_PREFIX}{timestamp}{collision_suffix}{SESSION_LOG_FILE_SUFFIX}"
            ));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(SESSION_LOG_MODE)
                .open(&path)
            {
                Ok(file) => return Ok(Self { path, file }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not reserve a unique session log name",
        ))
    }

    /// Writes one complete timestamped line, including its terminating newline.
    ///
    /// Returns the underlying write error so the UI can retain visible output
    /// and stop relying on a failed backing file.
    pub(crate) fn append(&mut self, line: &str) -> io::Result<()> {
        writeln!(self.file, "{line}")
    }

    /// Flushes pending writes and reads the complete UTF-8 session history.
    ///
    /// Flush, read and decoding errors are returned to the caller; the caller
    /// decides whether the visible output is an adequate fallback.
    pub(crate) fn complete_text(&mut self) -> io::Result<String> {
        self.file.flush()?;
        fs::read_to_string(&self.path)
    }

    /// Returns the reserved pathname for status messages without exposing the file.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}
