//! Per-control search diagnostics spanning the first scan and any fresh retry.
//!
//! Each lookup reports pixel work separately from elapsed wall time. When the
//! first frame is already available, wall timing starts at lookup construction
//! before its first search, excluding that earlier capture. The interval includes
//! waits and recapture performed while the lookup is alive. Recaptures count
//! attempted fresh captures, including failed attempts. Its owner retains
//! capture, cancellation, retry and guest-input policy.

use std::time::{Duration, Instant};

use super::{WorkerEventSink, milliseconds, send_log};
use crate::{
    capture::CapturedFrame,
    geometry::PixelPoint,
    parameters::{
        LEVEL_UP_CONFIRMATION_DELAY, TERMINAL_CONTROL_SEARCH_ATTEMPTS,
        TERMINAL_OK_FINAL_CONFIRMATIONS,
    },
    terminal_controls::{self, Control, profiles::{GameControlProfile, SearchSpec}},
};


/// Accumulate one expected control's scans and emit a summary on every exit path.
pub(super) struct ControlLookup<'a> {
    /// Game label used in each scan and final summary.
    game: String,
    /// Expected control whose bounded region this lookup examines.
    control: Control,
    /// Per-game native bounds and required consecutive black run length.
    spec: SearchSpec,
    /// Worker event sink receiving source-timestamped scan diagnostics.
    events: &'a WorkerEventSink,
    /// Wall-clock origin including waits and captures while this lookup is alive.
    started: Instant,
    /// Number of bounded scans attempted, including misses and scan errors.
    searches: u64,
    /// Number of fresh capture attempts recorded by the controller.
    recaptures: u64,
    /// Sum of pixels examined by successfully completed scans.
    pixels: u64,
    /// Sum of rows examined by successfully completed scans.
    rows: u64,
    /// Combined scanner time, separate from waits and capture wall time.
    scan_total: Duration,
    /// Latest scan or controller exit state reported by the final summary.
    outcome: &'static str,
    /// Previous fresh OK geometry; recognition never grants another input attempt.
    ok_readiness: terminal_controls::ok_readiness::OkReadiness,
    /// A visible OK button prevents a pending modal from exposing its underlay.
    ok_panel: bool,
    /// The latest frame needs only fresh geometry agreement before OK input.
    ok_confirmation_pending: bool,
    /// The latest body agrees with a preceding settled glyph that is now absent.
    ok_interrupted_confirmation: bool,
    /// A newly measured settled-size body still lacks geometry confirmation and ink.
    ok_body_confirmation_pending: bool,
    /// One final fresh observation has been reserved; never reset within this lookup.
    final_ok_confirmation_reserved: bool,
}


impl<'a> ControlLookup<'a> {
    /// Start wall timing before the first search, or before its initial capture.
    pub(super) fn new(
        game: impl Into<String>,
        profile: &GameControlProfile,
        control: Control,
        events: &'a WorkerEventSink,
    ) -> Self {
        Self {
            game: game.into(), control, spec: profile.spec(control), events, started: Instant::now(),
            searches: 0, recaptures: 0, pixels: 0, rows: 0,
            scan_total: Duration::ZERO, outcome: "not-searched",
            ok_readiness: terminal_controls::ok_readiness::OkReadiness::new(),
            ok_panel: false,
            ok_confirmation_pending: false,
            ok_interrupted_confirmation: false,
            ok_body_confirmation_pending: false,
            final_ok_confirmation_reserved: false,
        }
    }


    /// Scan and log one already captured frame through the normal worker sink.
    pub(super) fn search(&mut self, frame: &CapturedFrame) -> Result<PixelPoint, String> {
        self.ok_confirmation_pending = false;
        self.ok_interrupted_confirmation = false;
        self.ok_body_confirmation_pending = false;
        self.ok_panel = false;
        self.searches += 1;
        let bounds = self.spec.bounds;
        let right = bounds.width.checked_sub(1).and_then(|offset| bounds.x.checked_add(offset))
            .map_or_else(|| "invalid".to_owned(), |edge| edge.to_string());
        let bottom = bounds.height.checked_sub(1).and_then(|offset| bounds.y.checked_add(offset))
            .map_or_else(|| "invalid".to_owned(), |edge| edge.to_string());
        let prefix = format!(
            "{} {} search {}: X={}..={}, Y={}..={} inclusive; bottom-to-top rows, left-to-right pixels; {} consecutive RGB(0,0,0)",
            self.game, self.control.label(), self.searches,
            bounds.x, right, bounds.y, bottom, self.spec.run_length,
        );
        match terminal_controls::search(frame, self.spec) {
            Ok(result) => {
                self.pixels += result.pixels_examined;
                self.rows += u64::from(result.rows_examined);
                self.scan_total += result.scan_time;
                self.outcome = if result.point == PixelPoint::new(0, 0) { "missing" } else { "found" };
                send_log(self.events, format!(
                    "{prefix}; result=({}, {}); pixels={}; rows={}; scan_ms={:.3}.",
                    result.point.x, result.point.y, result.pixels_examined,
                    result.rows_examined, milliseconds(result.scan_time),
                ));
                Ok(result.point)
            }
            Err(error) => {
                self.forget_ok_geometry();
                self.outcome = "error";
                let message = format!("{prefix}; result=error: {error}; pixels=0; rows=0; scan_ms=0.000.");
                send_log(self.events, message);
                Err(format!("{} {} search failed: {error}", self.game, self.control.label()))
            }
        }
    }


    /// Confirm that OK has finished entering before its sole authorised click.
    ///
    /// The caller supplies each fresh frame once within its normal search
    /// budget or the single final OK reserve. The current frame needs settled
    /// geometry agreeing with its predecessor and a current exact-black glyph; the fading fixed caption
    /// is diagnostic only and never vetoes that evidence.
    /// Missing or incomplete evidence returns the normal no-match sentinel.
    /// Other controls keep their existing coordinate-only readiness policy.
    pub(super) fn confirm_ok(
        &mut self,
        frame: &CapturedFrame,
        point: PixelPoint,
    ) -> Result<PixelPoint, String> {
        self.ok_confirmation_pending = false;
        self.ok_interrupted_confirmation = false;
        self.ok_body_confirmation_pending = false;
        self.ok_panel = false;
        if self.control != Control::Ok {
            return Ok(point);
        }
        let started = Instant::now();
        let observation = self.ok_readiness.observe(frame, point).map_err(|error| {
            self.forget_ok_geometry();
            self.outcome = "readiness-error";
            format!("{} OK readiness failed: {error}", self.game)
        })?;
        self.ok_panel = observation.button.is_some();
        self.ok_confirmation_pending = observation.confirmation_pending;
        self.ok_interrupted_confirmation = observation.interrupted_confirmation;
        self.ok_body_confirmation_pending = observation.body_confirmation_pending;
        self.outcome = if observation.ready { "found" } else if point == PixelPoint::new(0, 0) {
            "missing"
        } else {
            "awaiting-settled-OK"
        };
        send_log(self.events, format!(
            "{} OK readiness: raw=({}, {}); selected=({}, {}); button={:?}; caption_present={}; caption_y={:?}; geometry_stable={}; glyph_bounds={:?}; ready={}; confirmation_pending={}; interrupted_confirmation={}; body_confirmation_pending={}; reason={}; readiness_pixels={}/{}; readiness_ms={:.3}; caption_gate=false; normal_search_budget={}; final_OK_reserve={}; reserve_consumed={}; guest input sent=0.",
            self.game, point.x, point.y, observation.point.x, observation.point.y,
            observation.button, observation.caption_present, observation.caption_y,
            observation.geometry_stable, observation.glyph_bounds,
            observation.ready, observation.confirmation_pending, observation.interrupted_confirmation,
            observation.body_confirmation_pending,
            observation.reason, observation.pixels_examined,
            terminal_controls::ok_readiness::MAX_READINESS_PIXEL_COMPARISONS,
            milliseconds(started.elapsed()),
            TERMINAL_CONTROL_SEARCH_ATTEMPTS, TERMINAL_OK_FINAL_CONFIRMATIONS,
            self.final_ok_confirmation_reserved,
        ));
        Ok(observation.point)
    }


    /// Choose and log the next existing retry wait without adding an observation.
    /// A current settled-size glyph awaiting geometry gets 200 ms. Interrupted
    /// glyphs and bodies without ink keep ordinary spacing until the final observation is reserved.
    pub(super) fn retry_delay(&self, normal: Duration) -> Duration {
        let confirmation = self.control == Control::Ok && self.ok_confirmation_pending;
        let interrupted = self.control == Control::Ok
            && self.ok_interrupted_confirmation
            && self.final_ok_confirmation_reserved;
        let body_confirmation = self.control == Control::Ok
            && self.ok_body_confirmation_pending
            && self.final_ok_confirmation_reserved;
        let delay = if confirmation || interrupted || body_confirmation {
            LEVEL_UP_CONFIRMATION_DELAY
        } else {
            normal
        };
        let policy = if confirmation {
            "fresh OK geometry confirmation"
        } else if interrupted {
            "reserved interrupted OK confirmation"
        } else if body_confirmation {
            "reserved late OK body confirmation"
        } else {
            "normal retry"
        };
        send_log(self.events, format!(
            "{} {} retry wait: chosen_ms={:.0}; normal_ms={:.0}; confirmation_pending={}; policy={}; normal_search_budget={}; interrupted_confirmation={}; body_confirmation_pending={}; reserve_consumed={}; guest input sent=0.",
            self.game, self.control.label(), milliseconds(delay), milliseconds(normal), confirmation, policy,
            TERMINAL_CONTROL_SEARCH_ATTEMPTS, self.ok_interrupted_confirmation,
            self.ok_body_confirmation_pending,
            self.final_ok_confirmation_reserved,
        ));
        delay
    }


    /// Reserve one final OK observation for glyph, interrupted or late-body confirmation on search three.
    ///
    /// The caller still owns its cancellable 200 ms wait and fresh capture.
    /// This method neither changes current readiness nor sends guest input.
    /// Consuming the reserve survives recaptures, geometry resets and errors.
    pub(super) fn reserve_final_ok_confirmation(&mut self, observations: usize) -> bool {
        let granted = self.control == Control::Ok
            && (self.ok_confirmation_pending || self.ok_interrupted_confirmation
                || self.ok_body_confirmation_pending)
            && observations == TERMINAL_CONTROL_SEARCH_ATTEMPTS
            && !self.final_ok_confirmation_reserved;
        if granted {
            self.final_ok_confirmation_reserved = true;
        }
        send_log(self.events, format!(
            "{} {} final OK confirmation reserve: granted={}; observations={}; normal_budget={}; allowance={}; consumed={}; maximum_observations={}; confirmation_pending={}; interrupted_confirmation={}; body_confirmation_pending={}; guest input sent=0. A granted reserve permits one fresh observation, not an OK click.",
            self.game, self.control.label(), granted, observations,
            TERMINAL_CONTROL_SEARCH_ATTEMPTS, TERMINAL_OK_FINAL_CONFIRMATIONS,
            self.final_ok_confirmation_reserved,
            TERMINAL_CONTROL_SEARCH_ATTEMPTS + TERMINAL_OK_FINAL_CONFIRMATIONS,
            self.ok_confirmation_pending,
            self.ok_interrupted_confirmation,
            self.ok_body_confirmation_pending,
        ));
        granted
    }


    /// Keep a pending foreground OK ahead of an optional New Game underlay.
    pub(super) fn has_ok_panel(&self) -> bool {
        self.control == Control::Ok && self.ok_panel
    }


    /// An intervening gameplay scene ends the preceding modal geometry evidence.
    pub(super) fn forget_ok_geometry(&mut self) {
        self.ok_readiness = terminal_controls::ok_readiness::OkReadiness::new();
        self.ok_panel = false;
        self.ok_confirmation_pending = false;
        self.ok_interrupted_confirmation = false;
        self.ok_body_confirmation_pending = false;
    }


    /// Record a fresh retry capture attempt immediately before invoking capture.
    /// Call after any cancellable wait; a failed capture still counts as attempted.
    ///
    /// Two alternative controls examined on the same fresh frame each record
    /// that frame once; their counts are per lookup, not additive capture totals.
    pub(super) fn note_recapture(&mut self) {
        self.ok_confirmation_pending = false;
        self.ok_interrupted_confirmation = false;
        self.ok_body_confirmation_pending = false;
        self.recaptures += 1;
        self.outcome = "recapturing";
    }


    /// Describe caller-owned exits and invalidate preceding readiness evidence.
    /// The current foreground-panel flag remains available to block an underlay;
    /// the consumed final reserve is never renewed within this lookup.
    pub(super) fn set_outcome(&mut self, outcome: &'static str) {
        self.ok_readiness = terminal_controls::ok_readiness::OkReadiness::new();
        self.ok_confirmation_pending = false;
        self.ok_interrupted_confirmation = false;
        self.ok_body_confirmation_pending = false;
        self.outcome = outcome;
    }
}


impl Drop for ControlLookup<'_> {
    /// Include misses, errors and cancelled retries without duplicating summaries.
    fn drop(&mut self) {
        if self.searches == 0 && self.recaptures == 0 && self.outcome == "not-searched" {
            return;
        }
        let wall_time = self.started.elapsed();
        send_log(self.events, format!(
            "{} {} search summary: searches={}; recaptures={}; second_grab={}; pixels={}; rows={}; scan_total_ms={:.3}; wall_ms={:.3}; final_OK_reserve_consumed={}; outcome={}.",
            self.game, self.control.label(), self.searches, self.recaptures,
            self.recaptures != 0, self.pixels, self.rows,
            milliseconds(self.scan_total), milliseconds(wall_time), self.final_ok_confirmation_reserved,
            self.outcome,
        ));
    }
}


#[cfg(test)]
mod tests {
    //! Search diagnostics retain per-attempt and final-summary evidence.

    use super::*;
    use super::super::{LatestFrameSlot, WorkerEvent};
    use crate::{
        capture::{PixelFormat, decode_png},
        game::GameMode,
        terminal_controls::profiles::{SPIDER_CONTROLS, for_game},
    };


    /// Derive a synthetic near-black glyph without changing the native settled body.
    fn washed_ok(frame: &CapturedFrame) -> CapturedFrame {
        let mut washed = frame.clone();
        let bounds = for_game(GameMode::FreeCell).spec(Control::Ok).bounds;
        for y in bounds.y..bounds.bottom() {
            for x in bounds.x..bounds.right() {
                let offset = y as usize * washed.stride + x as usize * 4;
                if washed.pixels[offset..offset + 3] == [0, 0, 0] {
                    washed.pixels[offset..offset + 3].fill(2);
                }
            }
        }
        washed
    }


    /// Exercise the production bounded scanner and readiness helper on one fresh response.
    fn inspect_ok(lookup: &mut ControlLookup<'_>, frame: &CapturedFrame) -> PixelPoint {
        let raw = lookup.search(frame).unwrap();
        lookup.confirm_ok(frame, raw).unwrap()
    }


    /// Every game can reserve a lost-glyph observation but must click only its fresh ink.
    #[test]
    fn all_profiles_reserve_interrupted_confirmation_only_on_third_observation() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let entering = decode_png(include_bytes!(
            "../../tests/fixtures/freecell-confirmation-20261016/entering.png"
        ))
        .unwrap();
        let clean = decode_png(include_bytes!(
            "../../tests/fixtures/freecell-confirmation-20261016/clean.png"
        ))
        .unwrap();
        let washed = washed_ok(&clean);
        let mut fresh = clean.clone();
        for x in 930..935 {
            let offset = 770 * fresh.stride + x * 4;
            fresh.pixels[offset..offset + 3].fill(0);
        }
        for game in GameMode::AVAILABLE {
            let mut lookup =
                ControlLookup::new(format!("{game:?}"), for_game(game), Control::Ok, &events);
            for frame in [&entering, &clean, &washed] {
                assert_eq!(inspect_ok(&mut lookup, frame), PixelPoint::new(0, 0));
            }
            assert!(!lookup.ok_confirmation_pending && lookup.ok_interrupted_confirmation);
            assert!(lookup.has_ok_panel());
            assert_eq!(
                lookup.retry_delay(Duration::from_secs(2)),
                Duration::from_secs(2)
            );
            for observations in [1, 2, 4] {
                assert!(!lookup.reserve_final_ok_confirmation(observations));
            }
            assert!(lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
            assert_eq!(
                lookup.retry_delay(Duration::from_secs(2)),
                LEVEL_UP_CONFIRMATION_DELAY
            );
            assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
            lookup.note_recapture();
            assert!(!lookup.ok_interrupted_confirmation);
            assert_eq!(
                lookup.retry_delay(Duration::from_secs(2)),
                Duration::from_secs(2)
            );
            assert_eq!(inspect_ok(&mut lookup, &fresh), PixelPoint::new(930, 770));
            assert!(!lookup.reserve_final_ok_confirmation(4));
            assert_eq!(lookup.searches, 4);
        }
        let logs: Vec<_> = receiver
            .try_iter()
            .filter_map(|event| match event.event {
                WorkerEvent::Log(message) => Some(message),
                _ => None,
            })
            .collect();
        assert_eq!(logs.iter().filter(|line| line.contains(
            "chosen_ms=200; normal_ms=2000; confirmation_pending=false; policy=reserved interrupted OK confirmation"
        )).count(), GameMode::AVAILABLE.len());
        assert_eq!(
            logs.iter()
                .filter(|line| line
                    .contains("final OK confirmation reserve: granted=true; observations=3;")
                    && line.contains("interrupted_confirmation=true"))
                .count(),
            GameMode::AVAILABLE.len()
        );
    }


    /// Interrupted ink before search three keeps the ordinary missing-glyph retry spacing.
    #[test]
    fn interrupted_second_observation_does_not_shorten_normal_retry_spacing() {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let clean = decode_png(include_bytes!(
            "../../tests/fixtures/freecell-confirmation-20261016/clean.png"
        ))
        .unwrap();
        let washed = washed_ok(&clean);
        let mut lookup = ControlLookup::new(
            "Free Cell",
            for_game(GameMode::FreeCell),
            Control::Ok,
            &events,
        );
        assert_eq!(inspect_ok(&mut lookup, &clean), PixelPoint::new(0, 0));
        assert_eq!(
            lookup.retry_delay(Duration::from_secs(2)),
            LEVEL_UP_CONFIRMATION_DELAY
        );
        lookup.note_recapture();
        assert_eq!(inspect_ok(&mut lookup, &washed), PixelPoint::new(0, 0));
        assert!(lookup.ok_interrupted_confirmation);
        assert!(!lookup.reserve_final_ok_confirmation(2));
        assert_eq!(
            lookup.retry_delay(Duration::from_secs(2)),
            Duration::from_secs(2)
        );
        lookup.note_recapture();
        assert_eq!(inspect_ok(&mut lookup, &washed), PixelPoint::new(0, 0));
        assert!(!lookup.ok_interrupted_confirmation);
        assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
    }


    /// Persistent fourth-frame washout cannot renew the reserve or reuse the old glyph.
    #[test]
    fn interrupted_reserve_remains_consumed_when_the_fresh_glyph_is_missing() {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let entering = decode_png(include_bytes!(
            "../../tests/fixtures/freecell-confirmation-20261016/entering.png"
        ))
        .unwrap();
        let clean = decode_png(include_bytes!(
            "../../tests/fixtures/freecell-confirmation-20261016/clean.png"
        ))
        .unwrap();
        let washed = washed_ok(&clean);
        let mut lookup = ControlLookup::new(
            "Free Cell",
            for_game(GameMode::FreeCell),
            Control::Ok,
            &events,
        );
        for frame in [&entering, &clean, &washed] {
            assert_eq!(inspect_ok(&mut lookup, frame), PixelPoint::new(0, 0));
        }
        assert!(lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
        lookup.note_recapture();
        assert_eq!(inspect_ok(&mut lookup, &washed), PixelPoint::new(0, 0));
        assert!(!lookup.ok_interrupted_confirmation);
        assert!(!lookup.reserve_final_ok_confirmation(4));
        lookup.forget_ok_geometry();
        assert_eq!(inspect_ok(&mut lookup, &clean), PixelPoint::new(0, 0));
        assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
        assert!(lookup.final_ok_confirmation_reserved);
    }


    /// Caller outcomes invalidate the predecessor while preserving a current modal barrier.
    #[test]
    fn outcome_invalidates_interrupted_history_but_preserves_foreground_panel() {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let clean = decode_png(include_bytes!(
            "../../tests/fixtures/freecell-confirmation-20261016/clean.png"
        ))
        .unwrap();
        let washed = washed_ok(&clean);
        for outcome in [
            "rejected-context",
            "reward-transition",
            "cancelled",
            "capture-error",
        ] {
            let mut lookup = ControlLookup::new(
                "Free Cell",
                for_game(GameMode::FreeCell),
                Control::Ok,
                &events,
            );
            assert_eq!(inspect_ok(&mut lookup, &clean), PixelPoint::new(0, 0));
            assert!(lookup.has_ok_panel());
            lookup.set_outcome(outcome);
            assert!(lookup.has_ok_panel());
            assert!(!lookup.ok_confirmation_pending && !lookup.ok_interrupted_confirmation
                && !lookup.ok_body_confirmation_pending);
            assert_eq!(inspect_ok(&mut lookup, &washed), PixelPoint::new(0, 0));
            assert!(!lookup.ok_interrupted_confirmation);
            assert!(lookup.ok_body_confirmation_pending);
            // This is observation two: reset history cannot make it observation three.
            assert!(!lookup.reserve_final_ok_confirmation(2));
        }
    }


    /// Geometry resets and malformed frames cannot leave lost-glyph reserve eligibility.
    #[test]
    fn reset_and_frame_errors_clear_interrupted_confirmation() {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let clean = decode_png(include_bytes!(
            "../../tests/fixtures/freecell-confirmation-20261016/clean.png"
        ))
        .unwrap();
        let washed = washed_ok(&clean);
        let mut malformed = clean.clone();
        malformed.pixels.clear();
        for failure in ["reset", "scan", "readiness"] {
            let mut lookup = ControlLookup::new(
                "Free Cell",
                for_game(GameMode::FreeCell),
                Control::Ok,
                &events,
            );
            assert_eq!(inspect_ok(&mut lookup, &clean), PixelPoint::new(0, 0));
            assert_eq!(inspect_ok(&mut lookup, &washed), PixelPoint::new(0, 0));
            assert!(lookup.ok_interrupted_confirmation);
            match failure {
                "reset" => lookup.forget_ok_geometry(),
                "scan" => assert!(lookup.search(&malformed).is_err()),
                _ => assert!(
                    lookup
                        .confirm_ok(&malformed, PixelPoint::new(0, 0))
                        .is_err()
                ),
            }
            assert!(!lookup.has_ok_panel() && !lookup.ok_interrupted_confirmation);
            assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
            assert_eq!(inspect_ok(&mut lookup, &washed), PixelPoint::new(0, 0));
            assert!(!lookup.ok_interrupted_confirmation);
        }
    }


    /// All profiles can reserve exactly one fourth observation after native third-frame pending OK.
    #[test]
    fn all_game_profiles_reserve_only_one_pending_final_ok_confirmation() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let entering = decode_png(include_bytes!("../../tests/fixtures/freecell-confirmation-20261016/entering.png")).unwrap();
        let tinted = decode_png(include_bytes!("../../tests/fixtures/freecell-confirmation-20261016/tinted.png")).unwrap();
        let clean = decode_png(include_bytes!("../../tests/fixtures/freecell-confirmation-20261016/clean.png")).unwrap();
        for game in GameMode::AVAILABLE {
            let mut lookup = ControlLookup::new(format!("{game:?}"), for_game(game), Control::Ok, &events);
            for frame in [&entering, &tinted, &clean] {
                let point = lookup.search(frame).unwrap();
                assert_eq!(lookup.confirm_ok(frame, point).unwrap(), PixelPoint::new(0, 0));
            }
            assert!(!lookup.reserve_final_ok_confirmation(1));
            assert!(!lookup.reserve_final_ok_confirmation(2));
            assert!(!lookup.reserve_final_ok_confirmation(4));
            assert!(lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
            assert_eq!(lookup.retry_delay(Duration::from_secs(1)), LEVEL_UP_CONFIRMATION_DELAY);
            assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
            lookup.note_recapture();
            // Repeated clean pixels model a fresh static capture, not observed live 200 ms timing.
            let point = lookup.search(&clean).unwrap();
            assert_eq!(lookup.confirm_ok(&clean, point).unwrap(), PixelPoint::new(942, 770));
            assert!(!lookup.reserve_final_ok_confirmation(4));
            assert_eq!(lookup.searches, 4);
        }
        let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
            WorkerEvent::Log(message) => Some(message),
            _ => None,
        }).collect();
        assert_eq!(logs.iter().filter(|line| line.contains(
            "final OK confirmation reserve: granted=true; observations=3; normal_budget=3; allowance=1; consumed=true; maximum_observations=4"
        )).count(), GameMode::AVAILABLE.len());
    }



    /// Recorded late entry can reserve one fresh response, but only its current ink clicks.
    #[test]
    fn all_profiles_reserve_late_body_confirmation_only_on_third_observation() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let reward = decode_png(include_bytes!(
            "../../tests/fixtures/spider-late-ok-20261018/reward.png"
        )).unwrap();
        let entering = decode_png(include_bytes!(
            "../../tests/fixtures/spider-late-ok-20261018/entering.png"
        )).unwrap();
        let near_black = decode_png(include_bytes!(
            "../../tests/fixtures/spider-late-ok-20261018/near-black.png"
        )).unwrap();
        // This pre-existing native image is a controlled fourth response, not a live recording.
        let fresh = decode_png(include_bytes!(
            "../../tests/fixtures/spider-game-win-20261015/grey-caption.png"
        )).unwrap();
        for game in GameMode::AVAILABLE {
            let mut lookup = ControlLookup::new(
                format!("{game:?}"), for_game(game), Control::Ok, &events,
            );
            for frame in [&reward, &entering, &near_black] {
                assert_eq!(inspect_ok(&mut lookup, frame), PixelPoint::new(0, 0));
            }
            assert!(lookup.has_ok_panel() && lookup.ok_body_confirmation_pending);
            assert!(!lookup.ok_confirmation_pending && !lookup.ok_interrupted_confirmation);
            assert_eq!(lookup.retry_delay(Duration::from_secs(2)), Duration::from_secs(2));
            for observations in [1, 2, 4] {
                assert!(!lookup.reserve_final_ok_confirmation(observations));
            }
            assert!(lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
            assert_eq!(lookup.retry_delay(Duration::from_secs(2)), LEVEL_UP_CONFIRMATION_DELAY);
            assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
            lookup.note_recapture();
            assert!(!lookup.ok_body_confirmation_pending);
            assert_eq!(inspect_ok(&mut lookup, &fresh), PixelPoint::new(943, 770));
            assert!(!lookup.reserve_final_ok_confirmation(4));
            assert_eq!(lookup.searches, 4);
        }
        let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
            WorkerEvent::Log(message) => Some(message),
            _ => None,
        }).collect();
        assert_eq!(logs.iter().filter(|line| line.contains(
            "policy=reserved late OK body confirmation"
        )).count(), GameMode::AVAILABLE.len());
    }


    /// A late body is scheduling evidence only; persistent fourth-frame washout still fails.
    #[test]
    fn late_body_confirmation_cannot_click_washout_or_renew_the_reserve() {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let reward = decode_png(include_bytes!(
            "../../tests/fixtures/spider-late-ok-20261018/reward.png"
        )).unwrap();
        let entering = decode_png(include_bytes!(
            "../../tests/fixtures/spider-late-ok-20261018/entering.png"
        )).unwrap();
        let near_black = decode_png(include_bytes!(
            "../../tests/fixtures/spider-late-ok-20261018/near-black.png"
        )).unwrap();
        let mut lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::Ok, &events);
        for frame in [&reward, &entering, &near_black] {
            assert_eq!(inspect_ok(&mut lookup, frame), PixelPoint::new(0, 0));
        }
        assert!(lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
        lookup.note_recapture();
        assert_eq!(inspect_ok(&mut lookup, &near_black), PixelPoint::new(0, 0));
        assert!(!lookup.ok_body_confirmation_pending);
        assert!(!lookup.reserve_final_ok_confirmation(4));
        lookup.forget_ok_geometry();
        assert!(!lookup.ok_body_confirmation_pending);
        assert_eq!(inspect_ok(&mut lookup, &near_black), PixelPoint::new(0, 0));
        assert!(lookup.ok_body_confirmation_pending);
        assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
        lookup.set_outcome("cancelled");
        assert!(!lookup.ok_body_confirmation_pending);
        assert!(lookup.final_ok_confirmation_reserved);
    }



    /// A previously full-size modal moving without ink is not late first-body evidence.
    #[test]
    fn late_body_confirmation_rejects_a_moved_previously_settled_body() {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let reward = decode_png(include_bytes!(
            "../../tests/fixtures/spider-late-ok-20261018/reward.png"
        )).unwrap();
        let near_black = decode_png(include_bytes!(
            "../../tests/fixtures/spider-late-ok-20261018/near-black.png"
        )).unwrap();
        let displaced = washed_ok(&decode_png(include_bytes!(
            "../../tests/fixtures/freecell-FC10.png"
        )).unwrap());
        let mut lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::Ok, &events);
        for frame in [&reward, &near_black, &displaced] {
            assert_eq!(inspect_ok(&mut lookup, frame), PixelPoint::new(0, 0));
        }
        assert!(lookup.has_ok_panel());
        assert!(!lookup.ok_confirmation_pending && !lookup.ok_interrupted_confirmation);
        assert!(!lookup.ok_body_confirmation_pending);
        assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
        assert_eq!(lookup.retry_delay(Duration::from_secs(1)), Duration::from_secs(1));
    }


    /// Missing evidence and controls without OK confirmation never gain a reserve.
    #[test]
    fn final_confirmation_reserve_rejects_missing_glyphs_and_non_ok_controls() {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let cases: &[&[u8]] = &[
            include_bytes!("../../tests/fixtures/spider-SP15.png"),
            include_bytes!("../../tests/fixtures/freecell-confirmation-20261016/entering.png"),
            include_bytes!("../../tests/fixtures/freecell-confirmation-20261016/tinted.png"),
            include_bytes!("../../tests/fixtures/game-win-readiness/fading.png"),
            include_bytes!("../../tests/fixtures/spider-game-win-20261015/near-black.png"),
        ];
        for bytes in cases {
            let frame = decode_png(bytes).unwrap();
            let mut lookup = ControlLookup::new("Free Cell", for_game(GameMode::FreeCell), Control::Ok, &events);
            for _ in 0..TERMINAL_CONTROL_SEARCH_ATTEMPTS {
                let point = lookup.search(&frame).unwrap();
                assert_eq!(lookup.confirm_ok(&frame, point).unwrap(), PixelPoint::new(0, 0));
            }
            assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
            assert!(!lookup.final_ok_confirmation_reserved);
        }
        let clean = decode_png(include_bytes!("../../tests/fixtures/freecell-confirmation-20261016/clean.png")).unwrap();
        for control in [Control::NewGame, Control::Play] {
            let mut lookup = ControlLookup::new("Free Cell", for_game(GameMode::FreeCell), control, &events);
            lookup.confirm_ok(&clean, PixelPoint::new(942, 770)).unwrap();
            assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
            assert!(!lookup.final_ok_confirmation_reserved);
        }
        let mut ready = ControlLookup::new("Free Cell", for_game(GameMode::FreeCell), Control::Ok, &events);
        ready.confirm_ok(&clean, PixelPoint::new(942, 770)).unwrap();
        assert_eq!(ready.confirm_ok(&clean, PixelPoint::new(942, 770)).unwrap(), PixelPoint::new(942, 770));
        assert!(!ready.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
    }


    /// Recapture, reset and failures cannot restore a consumed final observation.
    #[test]
    fn final_confirmation_consumption_survives_all_geometry_and_error_resets() {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let clean = decode_png(include_bytes!("../../tests/fixtures/freecell-confirmation-20261016/clean.png")).unwrap();
        let point = PixelPoint::new(942, 770);
        let mut malformed = clean.clone();
        malformed.pixels.clear();
        let mut lookup = ControlLookup::new("Free Cell", for_game(GameMode::FreeCell), Control::Ok, &events);
        lookup.confirm_ok(&clean, point).unwrap();
        assert!(lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
        lookup.note_recapture();
        lookup.forget_ok_geometry();
        assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
        assert_eq!(lookup.confirm_ok(&clean, point).unwrap(), PixelPoint::new(0, 0));
        assert_eq!(lookup.retry_delay(Duration::from_secs(1)), LEVEL_UP_CONFIRMATION_DELAY);
        assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));

        assert!(lookup.search(&malformed).is_err());
        assert_eq!(lookup.confirm_ok(&clean, point).unwrap(), PixelPoint::new(0, 0));
        assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
        assert!(lookup.confirm_ok(&malformed, point).is_err());
        assert_eq!(lookup.confirm_ok(&clean, point).unwrap(), PixelPoint::new(0, 0));
        assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
        lookup.set_outcome("capture-error");
        lookup.forget_ok_geometry();
        assert_eq!(lookup.confirm_ok(&clean, point).unwrap(), PixelPoint::new(0, 0));
        assert!(!lookup.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
        assert!(!lookup.reserve_final_ok_confirmation(4));
        assert!(lookup.final_ok_confirmation_reserved);

        let mut next_stage = ControlLookup::new("Free Cell", for_game(GameMode::FreeCell), Control::Ok, &events);
        next_stage.confirm_ok(&clean, point).unwrap();
        assert!(next_stage.reserve_final_ok_confirmation(TERMINAL_CONTROL_SEARCH_ATTEMPTS));
    }


    /// Every game's first settled glyph gets 200 ms, while the confirming frame is ready.
    #[test]
    fn all_game_ok_lookups_shorten_only_pending_geometry_confirmation() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let frame = decode_png(include_bytes!("../../tests/fixtures/spider-level-up-505.png")).unwrap();
        for game in GameMode::AVAILABLE {
            for normal in [Duration::from_secs(1), Duration::from_secs(2)] {
                let mut lookup = ControlLookup::new(format!("{game:?}"), for_game(game), Control::Ok, &events);
                assert_eq!(lookup.retry_delay(normal), normal);
                let first = lookup.search(&frame).unwrap();
                assert_eq!(lookup.confirm_ok(&frame, first).unwrap(), PixelPoint::new(0, 0));
                assert_eq!(lookup.retry_delay(normal), LEVEL_UP_CONFIRMATION_DELAY);
                lookup.note_recapture();
                assert_eq!(lookup.retry_delay(normal), normal);
                let second = lookup.search(&frame).unwrap();
                assert_eq!(lookup.confirm_ok(&frame, second).unwrap(), second);
                assert_eq!(lookup.retry_delay(normal), normal);
            }
        }
        let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
            WorkerEvent::Log(message) => Some(message),
            _ => None,
        }).collect();
        assert_eq!(logs.iter().filter(|line| line.contains(
            "chosen_ms=200; normal_ms="
        )).count(), GameMode::AVAILABLE.len() * 2);
        assert!(logs.iter().any(|line| line.contains(
            "confirmation_pending=true; policy=fresh OK geometry confirmation; normal_search_budget=3"
        )));
    }


    /// Missing, entering, near-black and pointer-only evidence retain the normal wait.
    #[test]
    fn incomplete_ok_evidence_and_other_controls_keep_their_existing_delay() {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let normal = Duration::from_secs(2);
        let cases: &[&[u8]] = &[
            include_bytes!("../../tests/fixtures/spider-SP15.png"),
            include_bytes!("../../tests/fixtures/game-win-readiness/early.png"),
            include_bytes!("../../tests/fixtures/game-win-readiness/fading.png"),
            include_bytes!("../../tests/fixtures/spider-game-win-20261015/near-black.png"),
        ];
        for bytes in cases {
            let frame = decode_png(bytes).unwrap();
            let mut lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::Ok, &events);
            for _ in 0..2 {
                let point = lookup.search(&frame).unwrap();
                assert_eq!(lookup.confirm_ok(&frame, point).unwrap(), PixelPoint::new(0, 0));
                assert_eq!(lookup.retry_delay(normal), normal);
            }
        }
        let frame = decode_png(include_bytes!("../../tests/fixtures/spider-level-up-505.png")).unwrap();
        for control in [Control::NewGame, Control::Play] {
            let mut lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, control, &events);
            assert_eq!(lookup.confirm_ok(&frame, PixelPoint::new(944, 771)).unwrap(), PixelPoint::new(944, 771));
            assert_eq!(lookup.retry_delay(normal), normal);
        }
    }


    /// Reset, failed storage and capture/error exits cannot retain a stale short wait.
    #[test]
    fn reset_and_error_paths_clear_pending_confirmation_and_geometry() {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let frame = decode_png(include_bytes!("../../tests/fixtures/spider-level-up-505.png")).unwrap();
        let point = PixelPoint::new(944, 771);
        let normal = Duration::from_secs(2);
        let mut malformed = frame.clone();
        malformed.pixels.clear();
        let mut lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::Ok, &events);

        assert_eq!(lookup.confirm_ok(&frame, point).unwrap(), PixelPoint::new(0, 0));
        assert_eq!(lookup.retry_delay(normal), LEVEL_UP_CONFIRMATION_DELAY);
        lookup.forget_ok_geometry();
        assert_eq!(lookup.retry_delay(normal), normal);
        assert!(!lookup.has_ok_panel());

        assert_eq!(lookup.confirm_ok(&frame, point).unwrap(), PixelPoint::new(0, 0));
        assert_eq!(lookup.retry_delay(normal), LEVEL_UP_CONFIRMATION_DELAY);
        assert!(lookup.search(&malformed).is_err());
        assert_eq!(lookup.retry_delay(normal), normal);
        assert!(!lookup.has_ok_panel());

        assert_eq!(lookup.confirm_ok(&frame, point).unwrap(), PixelPoint::new(0, 0));
        assert_eq!(lookup.retry_delay(normal), LEVEL_UP_CONFIRMATION_DELAY);
        assert!(lookup.confirm_ok(&malformed, point).is_err());
        assert_eq!(lookup.retry_delay(normal), normal);
        assert!(!lookup.has_ok_panel());

        assert_eq!(lookup.confirm_ok(&frame, point).unwrap(), PixelPoint::new(0, 0));
        assert_eq!(lookup.retry_delay(normal), LEVEL_UP_CONFIRMATION_DELAY);
        lookup.set_outcome("capture-error");
        assert_eq!(lookup.retry_delay(normal), normal);
    }


    /// Diagnostics preserve work counts across a miss followed by a fresh match.
    #[test]
    fn logs_each_attempt_and_one_aggregate_with_recapture() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let missing = decode_png(include_bytes!("../../tests/fixtures/spider-SP15.png")).unwrap();
        let found = decode_png(include_bytes!("../../tests/fixtures/spider-level-up-505.png")).unwrap();
        {
            let mut lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::Ok, &events);
            assert_eq!(lookup.search(&missing).unwrap(), PixelPoint::new(0, 0));
            lookup.note_recapture();
            assert_eq!(lookup.search(&found).unwrap(), PixelPoint::new(944, 771));
        }
        let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
            WorkerEvent::Log(message) => Some(message),
            _ => None,
        }).collect();
        assert_eq!(logs.len(), 3);
        assert!(logs[0].contains("Spider OK search 1: X=930..=954, Y=700..=850"));
        assert!(logs[0].contains("result=(0, 0); pixels=3775; rows=151; scan_ms="));
        assert!(logs[1].contains("result=(944, 771); pixels=1994; rows=80; scan_ms="));
        assert!(logs[2].contains("searches=2; recaptures=1; second_grab=true; pixels=5769; rows=231;"));
        assert!(logs[2].contains("scan_total_ms="));
        assert!(logs[2].contains("wall_ms="));
        assert!(logs[2].ends_with("outcome=found."));
    }


    /// Invalid storage remains an error and produces no fabricated pixel work.
    #[test]
    fn failure_is_logged_and_summarised_without_a_second_capture() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        let malformed = CapturedFrame {
            width: 1920, height: 1080, stride: 7680,
            format: PixelFormat::Rgba8, pixels: Vec::new(),
        };
        {
            let mut lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::Play, &events);
            assert!(lookup.search(&malformed).is_err());
        }
        let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
            WorkerEvent::Log(message) => Some(message),
            _ => None,
        }).collect();
        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains("result=error:"));
        assert!(logs[1].contains("searches=1; recaptures=0; second_grab=false; pixels=0; rows=0;"));
        assert!(logs[1].ends_with("outcome=error."));
    }


    /// An initial capture failure still reports its wall interval before any scan.
    #[test]
    fn initial_capture_error_emits_zero_search_summary() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let events = WorkerEventSink {
            events: sender,
            latest_frame: LatestFrameSlot::default(),
            capture_context: std::sync::Mutex::new(None),
        };
        {
            let mut lookup = ControlLookup::new("Spider", &SPIDER_CONTROLS, Control::Ok, &events);
            lookup.set_outcome("capture-error");
        }
        let logs: Vec<_> = receiver.try_iter().filter_map(|event| match event.event {
            WorkerEvent::Log(message) => Some(message),
            _ => None,
        }).collect();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("searches=0; recaptures=0; second_grab=false; pixels=0; rows=0;"));
        assert!(logs[0].contains("wall_ms="));
        assert!(logs[0].ends_with("outcome=capture-error."));
    }
}
