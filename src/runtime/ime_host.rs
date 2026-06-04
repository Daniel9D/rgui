//! Phase 3 / Plan 03-01: IME host driver abstraction.
//!
//! This module provides a producer-side trait (`ImeHostDriver`) that the
//! runtime polls once per frame. Hosts (winit, AppKit, browser, mock)
//! implement this trait to deliver `ImePreedit` / `ImeCommit` events
//! into the runtime's event queue via an `ImeEventSink`.
//!
//! The receive side (`handle_ime_preedit` and `is_focused_ime_enabled`)
//! is unchanged from Phase 2; this module only adds the *source* of the
//! events. Real drivers (winit, AppKit) are out of scope for v1; apps
//! that need them write a 30-line adapter.

/// A single IME operation that a `MockDriver` can replay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImeOp {
    Begin,
    Preedit(String, Option<(usize, usize)>),
    Commit(String),
    End,
}

impl ImeOp {
    /// Dispatch this op to the given sink. `Begin` and `End` are
    /// no-ops on the sink; `Preedit` and `Commit` push their
    /// payload to the matching `ImeEventSink` method.
    pub fn fire(&self, sink: &mut dyn ImeEventSink) {
        match self {
            ImeOp::Begin | ImeOp::End => {}
            ImeOp::Preedit(text, cursor) => sink.preedit(text.clone(), *cursor),
            ImeOp::Commit(text) => sink.commit(text.clone()),
        }
    }
}

/// Sink for IME events produced by an [`ImeHostDriver`]. The runtime
/// implements this on `UiRuntime` to push events into the existing
/// input event queue, which then routes through `handle_ime_preedit`
/// (Phase 2) — including the `ime_enabled` gate.
pub trait ImeEventSink {
    /// Push a preedit update. `cursor` is the byte range of the
    /// preedit cursor in the original `text`, or `None` if the
    /// host doesn't track it.
    fn preedit(&mut self, text: String, cursor: Option<(usize, usize)>);
    /// Push a commit. The preedit is cleared on commit.
    fn commit(&mut self, text: String);
}

/// Producer-side interface for a host's IME source. The runtime
/// invokes `poll` once per frame; the implementation pushes its
/// pending events through the sink.
///
/// Implementations are `&mut self` so they can hold state (e.g. a
/// `MockDriver` has a script cursor). The trait is intentionally
/// sync: per `PROJECT.md` the hot path is sync; async drivers wrap
/// their own runtime.
///
/// `Send + Sync` is required so `Box<dyn ImeHostDriver>` is
/// `Send + Sync` and `UiRuntime` (which holds one) can move
/// between threads — necessary for multi-threaded host loops
/// (winit's `EventLoop::run` on Linux / Windows, e.g.).
pub trait ImeHostDriver: Send + Sync {
    /// Drain any pending IME events into `sink`. Called once per
    /// frame, before the runtime processes the input event queue.
    fn poll(&mut self, sink: &mut dyn ImeEventSink);
}

/// Default driver that produces no IME events. The runtime uses
/// this when the host doesn't need IME.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoopDriver;

impl ImeHostDriver for NoopDriver {
    fn poll(&mut self, _sink: &mut dyn ImeEventSink) {}
}

/// Test driver that replays a `Vec<ImeOp>` script. Each call to
/// `poll` consumes one op from the script and dispatches it to the
/// sink. After the script is exhausted, `poll` is a no-op.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MockDriver {
    /// The script of IME ops to replay, in order.
    pub script: Vec<ImeOp>,
    /// Cursor into the script. Incremented each `poll` call.
    pub cursor: usize,
    /// Debug flag: `true` once at least one op has been fired.
    /// Optional in the public surface; the executor subagent
    /// doesn't have to set it.
    pub fired: bool,
}

impl ImeHostDriver for MockDriver {
    fn poll(&mut self, sink: &mut dyn ImeEventSink) {
        if self.cursor < self.script.len() {
            let op = &self.script[self.cursor];
            op.fire(sink);
            self.cursor += 1;
            self.fired = true;
        }
    }
}

/// Internal helper: an `ImeEventSink` that buffers events into a
/// `Vec<UiEvent>` for the runtime to drain after `driver.poll()`
/// returns. Lives in this module so the runtime's `update()` can
/// `use` it without adding a new field to `UiRuntime`.
///
/// Note: the `events` field carries `crate::core::UiEvent`, but
/// that re-export is in `core::event` — the runtime already
/// imports from `core`, so this is reachable.
#[derive(Default)]
pub(crate) struct ImeUpdateSink {
    pub events: Vec<crate::core::UiEvent>,
}

impl ImeEventSink for ImeUpdateSink {
    fn preedit(&mut self, text: String, cursor: Option<(usize, usize)>) {
        self.events.push(crate::core::UiEvent::ImePreedit(
            crate::core::ImePreedit {
                text,
                cursor_byte_range: cursor,
            },
        ));
    }

    fn commit(&mut self, text: String) {
        self.events
            .push(crate::core::UiEvent::ImeCommit(text));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test sink: a `Vec<(kind, payload)>` of recorded events.
    #[derive(Default)]
    struct RecordingSink {
        events: Vec<(String, String)>,
    }

    impl ImeEventSink for RecordingSink {
        fn preedit(&mut self, text: String, _cursor: Option<(usize, usize)>) {
            self.events.push(("preedit".to_string(), text));
        }
        fn commit(&mut self, text: String) {
            self.events.push(("commit".to_string(), text));
        }
    }

    #[test]
    fn mock_driver_replays_one_op_per_poll() {
        let mut driver = MockDriver {
            script: vec![
                ImeOp::Preedit("a".to_string(), None),
                ImeOp::Commit("a".to_string()),
                ImeOp::End,
            ],
            cursor: 0,
            fired: false,
        };
        let mut sink = RecordingSink::default();

        driver.poll(&mut sink);
        assert_eq!(sink.events.len(), 1);
        assert_eq!(sink.events[0].0, "preedit");
        assert_eq!(sink.events[0].1, "a");
        assert_eq!(driver.cursor, 1);
        assert!(driver.fired);

        driver.poll(&mut sink);
        assert_eq!(sink.events.len(), 2);
        assert_eq!(sink.events[1].0, "commit");
        assert_eq!(sink.events[1].1, "a");
        assert_eq!(driver.cursor, 2);

        driver.poll(&mut sink);
        assert_eq!(sink.events.len(), 2, "End is a no-op on the sink");
        assert_eq!(driver.cursor, 3);

        driver.poll(&mut sink);
        assert_eq!(
            sink.events.len(),
            2,
            "no further events after script is exhausted"
        );
        assert_eq!(driver.cursor, 3);
    }

    #[test]
    fn noop_driver_emits_no_events() {
        let mut driver = NoopDriver;
        let mut sink = RecordingSink::default();
        for _ in 0..5 {
            driver.poll(&mut sink);
        }
        assert!(sink.events.is_empty());
    }
}
