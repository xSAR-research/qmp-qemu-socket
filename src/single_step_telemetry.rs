//! Request-scoped, session-file-only Single step telemetry on the existing worker thread.
//!
//! No capture, input, retries or filesystem access originate here. The ordinary worker
//! installs one scope from explicit UI intent; disabled requests perform no formatting.

use std::{cell::RefCell, fmt::Display};

use serde_json::{Value, json};


/// One serial worker request's bounded sequence and file-event callback.
struct Trace {
    /// Ordered sequence within this request, unrelated to capture transport identifiers.
    sequence: u64,
    /// Existing worker channel writes through the UI's private session logger.
    emit: Box<dyn Fn(String)>,
}


thread_local! {
    /// Only the owning worker thread can observe this request's tracing state.
    static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) };
}


/// Restore the prior disabled/enabled state on every normal or early request exit.
pub(crate) struct Scope(Option<Trace>);


impl Scope {
    /// Enable only explicit Single step requests; a disabled nested scope stays silent.
    pub(crate) fn install(enabled: bool, emit: impl Fn(String) + 'static) -> Self {
        let trace = enabled.then(|| Trace {
            sequence: 0,
            emit: Box::new(emit),
        });
        let scope = Self(TRACE.with(|slot| slot.replace(trace)));
        record(
            || json!({"event": "begin", "scope": "Single step button", "ack_means": "delivery only"}),
        );
        scope
    }
}


impl Drop for Scope {
    fn drop(&mut self) {
        record(|| json!({"event": "end"}));
        TRACE.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}


/// Evaluate structured telemetry lazily only while this exact request is enabled.
pub(crate) fn record(build: impl FnOnce() -> Value) {
    TRACE.with(|slot| {
        if let Some(trace) = slot.borrow_mut().as_mut()
            && let Some(sequence) = trace.sequence.checked_add(1)
        {
            trace.sequence = sequence;
            (trace.emit)(format!(
                "SINGLE_STEP {}",
                json!({"sequence": sequence, "detail": build()})
            ));
        }
    });
}


/// Observe one existing transport call without changing its result or replaying it.
pub(crate) fn operation<T, E: Display>(
    operation: &str,
    details: impl FnOnce() -> Value,
    action: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    record(|| json!({"event": "transport-attempt", "operation": operation, "details": details()}));
    let result = action();
    record(|| {
        json!({"event": "transport-result", "operation": operation,
        "acknowledged": result.is_ok(), "error": result.as_ref().err().map(ToString::to_string)})
    });
    result
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    /// Explicit scope resets after exit and disabled requests never evaluate telemetry.
    #[test]
    fn single_step_302_scope_is_explicit_and_restores_after_exit() {
        let lines = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&lines);
        {
            let _scope = Scope::install(true, move |line| sink.borrow_mut().push(line));
            let mut calls = 0;
            let result: Result<(), &str> = operation(
                "click",
                || json!({"x": 1296, "y": 199}),
                || {
                    calls += 1;
                    Err("uncertain delivery")
                },
            );
            assert_eq!(result, Err("uncertain delivery"));
            assert_eq!(calls, 1);
            {
                let _disabled = Scope::install(false, |_| panic!("disabled request emitted"));
                record(|| panic!("disabled telemetry formatted"));
            }
            record(|| json!({"event": "after-disabled"}));
        }
        record(|| panic!("telemetry leaked beyond request"));
        let lines = lines.borrow();
        assert_eq!(lines.len(), 5);
        assert!(lines.first().unwrap().contains("begin"));
        assert!(lines.last().unwrap().contains("end"));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("uncertain delivery") && line.contains("false"))
        );
    }
}
