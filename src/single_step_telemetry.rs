//! Request-scoped session telemetry for Single Step and A* Free Cell development.
//!
//! No capture, input, retries or filesystem access originate here. The ordinary worker
//! installs one scope from explicit UI intent; disabled requests perform no formatting.

use std::{cell::RefCell, fmt::Display};

use serde_json::{Value, json};


/// One serial worker request's bounded sequence and file-event callback.
struct Trace {
    /// Distinguish existing Single Step records from scoped Free Cell development records.
    prefix: &'static str,
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
            prefix: "SINGLE_STEP",
            sequence: 0,
            emit: Box::new(emit),
        });
        let scope = Self(TRACE.with(|slot| slot.replace(trace)));
        record(
            || json!({"event": "begin", "scope": "Single step button", "ack_means": "delivery only"}),
        );
        scope
    }


    /// Trace one A* Free Cell command, retaining its exact immutable correlation fields.
    ///
    /// The caller selects the scope before work begins. This grants no input authority.
    pub(crate) fn freecell(identity: Value, emit: impl Fn(String) + 'static) -> Self {
        let scope = Self(TRACE.with(|slot| {
            slot.replace(Some(Trace {
                prefix: "FREE_CELL",
                sequence: 0,
                emit: Box::new(emit),
            }))
        }));
        record(|| {
            json!({"event":"begin", "scope":"A* Solution / FREE CELL", "identity":identity,
            "ack_means":"delivery only", "effect_proved":false})
        });
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
                "{} {}",
                trace.prefix,
                json!({"sequence": sequence, "detail": build()})
            ));
        }
    });
}


/// Identify the active Free Cell scope without evaluating telemetry or changing authority.
pub(crate) fn is_freecell() -> bool {
    TRACE.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|trace| trace.prefix == "FREE_CELL")
    })
}


/// Evaluate new development details only for A* Free Cell, excluding existing CV tracing.
pub(crate) fn record_freecell(build: impl FnOnce() -> Value) {
    if is_freecell() {
        record(build);
    }
}


/// Observe new transport details exclusively within the Free Cell development scope.
pub(crate) fn freecell_operation<T, E: Display>(
    operation_name: &str,
    details: impl FnOnce() -> Value,
    action: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    if is_freecell() {
        operation(operation_name, details, action)
    } else {
        action()
    }
}


/// Observe one existing transport call without changing its result or replaying it.
pub(crate) fn operation<T, E: Display>(
    operation: &str,
    details: impl FnOnce() -> Value,
    action: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    record(|| json!({"event": "transport-attempt", "operation": operation, "details": details()}));
    let result = action();
    let freecell = is_freecell();
    record(|| {
        let mut detail = json!({"event": "transport-result", "operation": operation,
        "acknowledged": result.is_ok(),
        "error": result.as_ref().err().map(ToString::to_string)});
        if freecell {
            detail["effect_proved"] = false.into();
        }
        detail
    });
    result
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;


    /// Development tracing is lazy outside Free Cell and preserves one uncertain result.
    #[test]
    fn freecell_development_scope_preserves_results_and_isolates_existing_tracing() {
        let lines = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&lines);
        record_freecell(|| panic!("disabled development detail evaluated"));
        let result: Result<u8, &str> =
            freecell_operation("offline", || panic!("disabled details evaluated"), || Ok(7));
        assert_eq!(result, Ok(7));
        {
            let _legacy = Scope::install(true, move |line| sink.borrow_mut().push(line));
            record_freecell(|| panic!("Free Cell leaked into existing CV tracing"));
            let sink = Rc::clone(&lines);
            {
                let _freecell = Scope::freecell(json!({"request_id":42}), move |line| {
                    sink.borrow_mut().push(line)
                });
                let mut calls = 0;
                let result: Result<(), &str> = freecell_operation(
                    "drag",
                    || json!({"pointer":[123,456]}),
                    || {
                        calls += 1;
                        Err("original uncertain response")
                    },
                );
                assert_eq!(calls, 1);
                assert_eq!(result, Err("original uncertain response"));
            }
            assert!(!is_freecell());
            record_freecell(|| panic!("development scope survived its request"));
        }
        let lines = lines.borrow();
        let details: Vec<Value> = lines
            .iter()
            .filter_map(|line| line.strip_prefix("FREE_CELL "))
            .map(|text| serde_json::from_str(text).unwrap())
            .collect();
        assert_eq!(details.len(), 4);
        for (index, entry) in details.iter().enumerate() {
            assert_eq!(entry["sequence"], index + 1);
        }
        assert_eq!(details[0]["detail"]["identity"]["request_id"], 42);
        assert_eq!(details[2]["detail"]["acknowledged"], false);
        assert_eq!(details[2]["detail"]["effect_proved"], false);
        assert_eq!(details[2]["detail"]["error"], "original uncertain response");
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.starts_with("SINGLE_STEP "))
                .count(),
            2
        );
    }

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
