//! Signal requests are atomic flags; controller teardown stays on ordinary application threads.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};


/// Own this process's SIGINT/SIGTERM registrations and its monotonic shutdown request.
pub(crate) struct ShutdownRequest {
    /// Never cleared; distinct from the reusable gameplay STOP flag.
    requested: Arc<AtomicBool>,
    /// Only this owner's handlers are unregistered on normal teardown.
    #[cfg(unix)]
    registrations: Vec<signal_hook::SigId>,
}


impl ShutdownRequest {
    /// Install flag-only handlers; return registration errors before starting the worker.
    /// SIGKILL cannot be caught. The UI polls this flag and requests cooperative STOP.
    pub(crate) fn install() -> std::io::Result<Self> {
        let mut request = Self {
            requested: Arc::new(AtomicBool::new(false)),
            #[cfg(unix)]
            registrations: Vec::new(),
        };
        #[cfg(unix)]
        for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
            request
                .registrations
                .push(signal_hook::flag::register(signal, request.flag())?);
        }
        Ok(request)
    }


    /// Share the irreversible request with the serial worker and UI.
    pub(crate) fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.requested)
    }


    /// Whether EXIT, native close or a supported termination signal requested teardown.
    pub(crate) fn requested(&self) -> bool {
        self.requested.load(Ordering::SeqCst)
    }


    /// Latch shutdown without doing I/O, taking locks or waiting for the worker.
    pub(crate) fn request(&self) {
        self.requested.store(true, Ordering::SeqCst);
    }
}


impl Drop for ShutdownRequest {
    /// Remove only owned handlers after normal controller teardown.
    fn drop(&mut self) {
        #[cfg(unix)]
        for registration in self.registrations.drain(..) {
            signal_hook::low_level::unregister(registration);
        }
    }
}


#[cfg(all(test, unix))]
mod tests {
    use super::*;


    /// Exercise each real signal in a separate test process, isolating global dispositions.
    #[test]
    fn termination_signals_latch_request_in_isolated_processes() {
        for signal in ["INT", "TERM"] {
            let result = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "shutdown::tests::signal_child", "--nocapture"])
                .env("QMP_SHUTDOWN_SIGNAL_TEST", signal)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }


    /// Child-only real signal handling; normal parallel test execution changes no handlers.
    #[test]
    fn signal_child() {
        let Ok(signal) = std::env::var("QMP_SHUTDOWN_SIGNAL_TEST") else {
            return;
        };
        let shutdown = ShutdownRequest::install().unwrap();
        assert!(!shutdown.requested());
        signal_hook::low_level::raise(match signal.as_str() {
            "INT" => signal_hook::consts::SIGINT,
            "TERM" => signal_hook::consts::SIGTERM,
            _ => panic!("unsupported isolated test signal"),
        })
        .unwrap();
        assert!(shutdown.requested());
    }
}
