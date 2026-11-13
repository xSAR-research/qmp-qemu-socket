//! Passive outgoing QMP evidence on the existing ordered log channel.
//!
//! One worker scope covers every game and connection. Per-command and per-connect
//! identities are frozen before transport work; recording grants no input authority.

use super::*;
use serde_json::{Value, json};
use xsar::qmp::{QmpWireConnection, QmpWireEvent, QmpWireObserver};


/// Shared counters distinguish commands and connections even when QMP IDs restart.
struct Counters {
    /// Monotonic origin for wire callbacks, independent of UI polling.
    started_at: Instant,
    /// Ordered records across all connections on this worker.
    records: std::sync::atomic::AtomicU64,
    /// Application connection ordinal; recovery is additionally tagged by xSAR.
    connections: std::sync::atomic::AtomicU64,
    /// Worker command ordinal, including commands without their own request ID.
    commands: std::sync::atomic::AtomicU64,
}


/// Immutable command identity copied into every connection observer.
#[derive(Clone)]
struct Owner {
    /// Context used for event delivery, distinct from any later selection.
    context: Option<ControllerContext>,
    /// Persisted context and request fields; the UI does not serialise outer tags.
    detail: Value,
}


/// Only the serial worker installs an outgoing-wire destination.
struct State {
    /// Existing ordered small-event channel; no new file is opened here.
    sender: Sender<ContextualWorkerEvent>,
    /// Counter ownership survives a connection and its release recovery.
    counters: Arc<Counters>,
    /// Exact command whose connection is currently being created.
    owner: Option<Owner>,
}


thread_local! {
    /// Isolated from UI state, other workers and parallel tests.
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}


/// Restore the previous wire destination when the worker exits.
pub(super) struct WorkerScope(Option<State>);


impl WorkerScope {
    /// Attach the existing event sender before any command can open QMP.
    pub(super) fn install(events: &WorkerEventSink) -> Self {
        let state = State {
            sender: events.events.clone(),
            counters: Arc::new(Counters {
                started_at: Instant::now(),
                records: std::sync::atomic::AtomicU64::new(0),
                connections: std::sync::atomic::AtomicU64::new(0),
                commands: std::sync::atomic::AtomicU64::new(0),
            }),
            owner: None,
        };
        Self(STATE.with(|slot| slot.replace(Some(state))))
    }
}


impl Drop for WorkerScope {
    /// Never leave an observer destination enabled on an unrelated later request.
    fn drop(&mut self) {
        STATE.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}


/// Restore the prior request identity after every normal or early command exit.
pub(super) struct CommandScope(Option<Owner>);


impl CommandScope {
    /// Freeze existing request IDs and a worker ordinal without reading UI state.
    pub(super) fn install(command: &WorkerCommand) -> Self {
        Self(STATE.with(|slot| {
            let mut slot = slot.borrow_mut();
            let state = slot.as_mut()?;
            let (context, mut detail) = identity(command);
            detail["worker_command_id"] = next(&state.counters.commands).into();
            detail["context"] = context.as_ref().map(context_details).into();
            state.owner.replace(Owner { context, detail })
        }))
    }
}


impl Drop for CommandScope {
    /// A completed command cannot lend its identity to a later connection.
    fn drop(&mut self) {
        STATE.with(|slot| {
            if let Some(state) = slot.borrow_mut().as_mut() {
                state.owner = self.0.take();
            }
        });
    }
}


/// Advance a diagnostic counter without wrapping into an earlier identity.
fn next(counter: &std::sync::atomic::AtomicU64) -> Option<u64> {
    counter
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .ok()
        .and_then(|previous| previous.checked_add(1))
}


/// Retain structured controller fields rather than only an outer channel tag.
fn context_details(context: &ControllerContext) -> Value {
    json!({"socket":context.socket_path,"game":context.game_mode.label(),
        "strategy":context.strategy.label(),"generation":context.generation})
}


/// Inspect only bounded command metadata, excluding frames, routes and predictions.
fn identity(command: &WorkerCommand) -> (Option<ControllerContext>, Value) {
    let (context, detail) = match command {
        WorkerCommand::CalculateFreeCell { identity, path, .. } => (
            Some(&identity.context),
            json!({"command":"calculate-freecell","request_id":identity.request_id,
                "source_generation":identity.source_generation,"source":path}),
        ),
        WorkerCommand::ExecuteFreeCellRoute {
            plan,
            run_id,
            single_step,
            ..
        } => (
            Some(&plan.context),
            json!({"command":"execute-freecell","request_id":plan.request_id,
                "source_generation":plan.source_generation,"run_id":run_id,
                "single_step":single_step,
                "diagnostic_drag_mode":"Absolute",
                "gesture_phase_wait_ms":crate::parameters::FREECELL_ROUTE_ABS_GESTURE_INTERVAL.as_millis()}),
        ),
        WorkerCommand::ListFreeCellSources {
            context,
            request_id,
            directory,
        } => (
            Some(context),
            json!({"command":"list-freecell-sources",
                "request_id":request_id,"directory":directory}),
        ),
        WorkerCommand::ReconnoitreFreeCell {
            context,
            request_id,
            request,
        } => (
            Some(context),
            json!({"command":"reconnoitre-freecell",
                "request_id":request_id,"export":request.destination}),
        ),
        WorkerCommand::CaptureFrame { context } => {
            (Some(context), json!({"command":"capture-frame"}))
        }
        WorkerCommand::CaptureResetPreview {
            context,
            request_id,
        } => (
            Some(context),
            json!({"command":"capture-reset-preview","request_id":request_id}),
        ),
        WorkerCommand::PrepareSnapshot {
            context,
            request_id,
        } => (
            Some(context),
            json!({"command":"prepare-snapshot","request_id":request_id}),
        ),
        WorkerCommand::ExecuteSteps {
            context, settings, ..
        } => (
            Some(context),
            json!({"command":"execute-steps",
                "single_step":settings.single_step_diagnostics()}),
        ),
        WorkerCommand::Reconnoitre { context, .. } => {
            (Some(context), json!({"command":"reconnoitre-pyramid"}))
        }
        WorkerCommand::StartPyramidSession { context, request } => (
            Some(context),
            json!({"command":"pyramid-session","run_id":request.run_id}),
        ),
        WorkerCommand::StartFreeCellSession { context, request } => (
            Some(context),
            json!({"command":"freecell-session","run_id":request.run_id,"play_style":request.play_style.label()}),
        ),
        WorkerCommand::CalculateRoute {
            context,
            run_id,
            path,
            ..
        } => (
            Some(context),
            json!({"command":"calculate-route","run_id":run_id,"source":path}),
        ),
        WorkerCommand::ExecuteRoute {
            context,
            run_id,
            plan_id,
            ..
        } => (
            Some(context),
            json!({"command":"execute-route","run_id":run_id,"plan_id":plan_id}),
        ),
        WorkerCommand::RecordPyramidFailure { context, .. } => {
            (Some(context), json!({"command":"persist-failure"}))
        }
        WorkerCommand::ResetProgress(context) => {
            (Some(context), json!({"command":"reset-progress"}))
        }
        WorkerCommand::Disconnect(context) => (Some(context), json!({"command":"disconnect"})),
        WorkerCommand::Shutdown => (None, json!({"command":"shutdown"})),
    };
    (context.cloned(), detail)
}


/// Frozen per-connect callback, inherited unchanged by the transport's recovery.
struct Observer {
    /// Existing ordered event destination for private provenance and readable packets.
    sender: Sender<ContextualWorkerEvent>,
    /// Shared record and clock state, never gameplay authority.
    counters: Arc<Counters>,
    /// Primary application connection; recovery has its own additional stream tag.
    connection_id: Option<u64>,
    /// Exact socket passed to this connection, independent of later selection.
    socket: PathBuf,
    /// Exact command which opened the primary connection.
    owner: Option<Owner>,
}


impl QmpWireObserver for Observer {
    /// Queue evidence synchronously at the transport boundary without disk or input I/O.
    fn observe(&self, event: QmpWireEvent<'_>) {
        let emitted_at = worker_elapsed();
        let elapsed_us = self.counters.started_at.elapsed().as_micros();
        let (connection, id, detail, packet) = match event {
            QmpWireEvent::Attempted {
                connection,
                id,
                json,
            } => (connection, id, json!({"event":"write-attempt"}), Some(json)),
            QmpWireEvent::Flushed { connection, id } => {
                (connection, id, json!({"event":"write-flushed"}), None)
            }
            QmpWireEvent::Failed {
                connection,
                id,
                stage,
                error,
            } => (
                connection,
                id,
                json!({"event":"write-failed","stage":format!("{stage:?}"),
                    "error":error.to_string()}),
                None,
            ),
        };
        let connection_kind = match connection {
            QmpWireConnection::Primary => "primary",
            QmpWireConnection::ReleaseRecovery => "release-recovery",
        };
        let sequence = next(&self.counters.records);
        let metadata = format!(
            "QMP_WIRE {}",
            json!({
                "sequence":sequence,"wire_elapsed_us":elapsed_us,
                "source_elapsed_us":emitted_at.map(|time|time.as_micros()),
                "connection_id":self.connection_id,"connection_kind":connection_kind,
                "socket":self.socket,"owner":self.owner.as_ref().map(|owner|&owner.detail),
                "qmp_id":id,"detail":detail,"effect_proved":false
            })
        );
        let event = match packet {
            Some(packet) => WorkerEvent::QmpWireLog {
                metadata,
                packet_line: format!(
                    "QMP_TX sequence={} connection={}/{} qmp_id={} attempted\n{}",
                    json!(sequence),
                    json!(self.connection_id),
                    connection_kind,
                    id,
                    packet
                ),
            },
            None => WorkerEvent::SessionTelemetry(metadata),
        };
        let _ = self.sender.send(ContextualWorkerEvent {
            context: self.owner.as_ref().and_then(|owner| owner.context.clone()),
            emitted_at,
            event,
        });
    }
}


/// Supply an observer before the constructor sends its capabilities handshake.
///
/// Outside the serial worker (for example a pure transport test), tracing stays off.
pub(crate) fn observer(socket: &Path) -> Option<Arc<dyn QmpWireObserver>> {
    STATE.with(|slot| {
        let slot = slot.borrow();
        let state = slot.as_ref()?;
        Some(Arc::new(Observer {
            sender: state.sender.clone(),
            counters: Arc::clone(&state.counters),
            connection_id: next(&state.counters.connections),
            socket: socket.to_path_buf(),
            owner: state.owner.clone(),
        }) as Arc<dyn QmpWireObserver>)
    })
}


#[cfg(test)]
mod tests {
    //! Exercise the private adapter without QEMU, filesystem writes or UI polling.

    use super::*;
    use xsar::qmp::{QmpError, QmpWireStage};


    /// Use the real ordered event channel while leaving frame delivery empty.
    fn sink() -> (WorkerEventSink, Receiver<ContextualWorkerEvent>) {
        let (events, receiver) = mpsc::channel();
        (
            WorkerEventSink {
                events,
                latest_frame: LatestFrameSlot::default(),
                capture_context: Mutex::new(None),
            },
            receiver,
        )
    }


    /// Preserve the exact wire text alongside IDs even after command selection changes.
    #[test]
    fn qmp_wire_316_all_games_freeze_owner_and_distinguish_recovery() {
        let (sink, receiver) = sink();
        let _worker = WorkerScope::install(&sink);
        let first = ControllerContext::new(
            PathBuf::from("/tmp/primary.sock"),
            GameMode::Klondike,
            SolvingStrategy::ComputerVision,
            7,
        );
        let primary = {
            let _command = CommandScope::install(&WorkerCommand::CaptureFrame {
                context: first.clone(),
            });
            observer(&first.socket_path).unwrap()
        };
        let second = ControllerContext::new(
            PathBuf::from("/tmp/second.sock"),
            GameMode::FreeCell,
            SolvingStrategy::ShortestPath,
            8,
        );
        let later = {
            let _command = CommandScope::install(&WorkerCommand::PrepareSnapshot {
                context: second.clone(),
                request_id: 42,
            });
            observer(&second.socket_path).unwrap()
        };
        let json = r#"{"execute":"qmp_capabilities","id":1}"#;
        primary.observe(QmpWireEvent::Attempted {
            connection: QmpWireConnection::Primary,
            id: 1,
            json,
        });
        primary.observe(QmpWireEvent::Flushed {
            connection: QmpWireConnection::Primary,
            id: 1,
        });
        primary.observe(QmpWireEvent::Attempted {
            connection: QmpWireConnection::ReleaseRecovery,
            id: 1,
            json,
        });
        primary.observe(QmpWireEvent::Flushed {
            connection: QmpWireConnection::ReleaseRecovery,
            id: 1,
        });
        later.observe(QmpWireEvent::Attempted {
            connection: QmpWireConnection::Primary,
            id: 1,
            json,
        });
        let error = QmpError::Protocol("original write-stage diagnostic".to_owned());
        later.observe(QmpWireEvent::Failed {
            connection: QmpWireConnection::Primary,
            id: 1,
            stage: QmpWireStage::Flush,
            error: &error,
        });
        let events: Vec<_> = receiver.try_iter().collect();
        assert_eq!(events.len(), 6);
        let records: Vec<Value> = events
            .iter()
            .enumerate()
            .map(|(index, event)| {
                assert_eq!(
                    event.context.as_ref(),
                    Some(if index < 4 { &first } else { &second })
                );
                let message = match &event.event {
                    WorkerEvent::QmpWireLog {
                        metadata,
                        packet_line,
                    } => {
                        assert!([0, 2, 4].contains(&index));
                        assert_eq!(packet_line.lines().last(), Some(json));
                        assert_eq!(packet_line.matches(json).count(), 1);
                        if index == 2 {
                            assert!(packet_line.contains("connection=1/release-recovery"));
                        }
                        metadata
                    }
                    WorkerEvent::SessionTelemetry(message) => {
                        assert!([1, 3, 5].contains(&index));
                        message
                    }
                    _ => panic!("wire evidence must bypass generic Log/Status tracing"),
                };
                let record: Value =
                    serde_json::from_str(message.strip_prefix("QMP_WIRE ").unwrap()).unwrap();
                assert_eq!(record["sequence"], index + 1);
                assert_eq!(record["qmp_id"], 1);
                assert_eq!(record["effect_proved"], false);
                assert!(record["wire_elapsed_us"].is_number());
                assert!(record["detail"].get("json").is_none());
                assert!(!message.contains("qmp_capabilities"));
                record
            })
            .collect();
        assert_eq!(records[0]["connection_id"], 1);
        assert_eq!(records[2]["connection_id"], 1);
        assert_eq!(records[2]["connection_kind"], "release-recovery");
        assert_eq!(records[4]["connection_id"], 2);
        assert_eq!(records[0]["owner"]["context"]["generation"], 7);
        assert_eq!(records[0]["owner"]["worker_command_id"], 1);
        assert_eq!(records[4]["owner"]["worker_command_id"], 2);
        assert_eq!(records[4]["owner"]["request_id"], 42);
        assert_eq!(records[0]["detail"]["event"], "write-attempt");
        assert_eq!(records[2]["detail"]["event"], "write-attempt");
        assert_eq!(records[5]["detail"]["stage"], "Flush");
        assert_eq!(records[5]["detail"]["error"], error.to_string());
        assert!(sink.latest_frame.take().is_none());
    }


    /// Worker exit restores disabled tracing; no stale owner or sender leaks into tests.
    #[test]
    fn qmp_wire_316_scope_restores_disabled_observer() {
        assert!(observer(Path::new("/tmp/inactive.sock")).is_none());
        let (sink, receiver) = sink();
        {
            let _worker = WorkerScope::install(&sink);
            let _command = CommandScope::install(&WorkerCommand::Shutdown);
            observer(Path::new("/tmp/observed.sock"))
                .unwrap()
                .observe(QmpWireEvent::Attempted {
                    connection: QmpWireConnection::Primary,
                    id: 1,
                    json: r#"{"execute":"qmp_capabilities","id":1}"#,
                });
        }
        assert!(observer(Path::new("/tmp/inactive.sock")).is_none());
        let event = receiver.try_recv().unwrap();
        assert!(event.context.is_none());
        assert!(matches!(event.event, WorkerEvent::QmpWireLog { .. }));
        assert!(receiver.try_recv().is_err());
    }
}
