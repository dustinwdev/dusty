//! Performance profiling — frame timing, runtime snapshots, and delta reports.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::error::Result;

/// Ring-buffer timer that tracks frame durations.
///
/// The caller drives timing via [`begin_frame`](Self::begin_frame) and
/// [`end_frame`](Self::end_frame). Old entries are evicted when capacity
/// is reached.
///
/// # Examples
///
/// ```
/// use dusty_devtools::profiler::FrameTimer;
///
/// let mut timer = FrameTimer::new(60);
/// timer.begin_frame();
/// // ... render ...
/// timer.end_frame();
/// assert!(timer.stats().is_some());
/// ```
#[derive(Debug)]
pub struct FrameTimer {
    durations: VecDeque<Duration>,
    capacity: usize,
    current_start: Option<Instant>,
}

/// Aggregate statistics over recorded frame durations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameStats {
    /// Number of frames recorded.
    pub frame_count: usize,
    /// Shortest frame duration.
    pub min: Duration,
    /// Longest frame duration.
    pub max: Duration,
    /// Mean frame duration.
    pub avg: Duration,
    /// 95th percentile frame duration.
    pub p95: Duration,
    /// Most recent frame duration.
    pub last: Duration,
}

impl FrameTimer {
    /// Creates a new timer with the given ring-buffer capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            durations: VecDeque::with_capacity(capacity),
            capacity,
            current_start: None,
        }
    }

    /// Marks the start of a frame. Call [`end_frame`](Self::end_frame) to
    /// record the duration.
    pub fn begin_frame(&mut self) {
        self.current_start = Some(Instant::now());
    }

    /// Records the duration since the last [`begin_frame`](Self::begin_frame).
    ///
    /// If `begin_frame` was not called, this is a no-op.
    pub fn end_frame(&mut self) {
        if let Some(start) = self.current_start.take() {
            self.push_duration(start.elapsed());
        }
    }

    /// Directly records a duration. Useful for manual timing or testing.
    pub fn push_duration(&mut self, duration: Duration) {
        if self.durations.len() == self.capacity {
            self.durations.pop_front();
        }
        self.durations.push_back(duration);
    }

    /// Computes aggregate statistics over the recorded durations.
    ///
    /// Returns `None` if no frames have been recorded.
    #[must_use]
    pub fn stats(&self) -> Option<FrameStats> {
        if self.durations.is_empty() {
            return None;
        }

        let mut sorted: Vec<Duration> = self.durations.iter().copied().collect();
        sorted.sort();

        let frame_count = sorted.len();
        let min = sorted[0];
        let max = sorted[frame_count - 1];
        let sum: Duration = sorted.iter().sum();
        #[allow(clippy::cast_possible_truncation)]
        let avg = sum / frame_count as u32;

        // p95: use ceiling index
        let p95_index = (frame_count * 95).div_ceil(100).min(frame_count) - 1;
        let p95 = sorted[p95_index];

        // last is the most recently pushed duration (back of deque)
        let last = self.durations.back().copied().unwrap_or_default();

        Some(FrameStats {
            frame_count,
            min,
            max,
            avg,
            p95,
            last,
        })
    }

    /// Returns the number of recorded frames.
    #[must_use]
    pub fn len(&self) -> usize {
        self.durations.len()
    }

    /// Returns `true` if no frames have been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.durations.is_empty()
    }
}

/// Snapshot of the reactive runtime's state at a point in time.
#[derive(Debug, Clone)]
pub struct RuntimeSnapshot {
    /// Total signal slots ever allocated.
    pub total_signals: usize,
    /// Signals currently alive.
    pub live_signals: usize,
    /// Total scope slots ever allocated.
    pub total_scopes: usize,
    /// Scopes currently alive.
    pub live_scopes: usize,
    /// Total subscriber slots ever allocated.
    pub total_subscribers: usize,
    /// Subscribers currently alive.
    pub live_subscribers: usize,
    /// Per-signal snapshot data.
    pub signals: Vec<SignalSnapshot>,
    /// When this snapshot was taken.
    pub timestamp: Instant,
}

/// Snapshot of a single signal at a point in time.
#[derive(Debug, Clone)]
pub struct SignalSnapshot {
    /// Slot index.
    pub index: usize,
    /// Version counter at snapshot time.
    pub version: u64,
    /// Number of subscribers at snapshot time.
    pub subscriber_count: usize,
    /// Whether the signal was alive at snapshot time.
    pub alive: bool,
}

/// Report comparing two runtime snapshots.
#[derive(Debug, Clone)]
pub struct ProfileReport {
    /// Time elapsed between the two snapshots.
    pub elapsed: Duration,
    /// Signals whose version changed between snapshots.
    pub signal_deltas: Vec<SignalDelta>,
}

/// Version change for a single signal between two snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalDelta {
    /// Signal slot index.
    pub index: usize,
    /// How many times the signal's value changed.
    pub version_delta: u64,
}

/// Takes a snapshot of the reactive runtime's current state.
///
/// # Errors
///
/// Returns an error if the reactive runtime is not initialized.
///
/// # Examples
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_signal, dispose_runtime};
/// use dusty_devtools::profiler::snapshot_runtime;
///
/// initialize_runtime();
/// let _sig = create_signal(42).unwrap();
/// let snap = snapshot_runtime().unwrap();
/// assert_eq!(snap.live_signals, 1);
/// dispose_runtime();
/// ```
pub fn snapshot_runtime() -> Result<RuntimeSnapshot> {
    let stats = dusty_reactive::stats::runtime_stats()?;
    let timestamp = Instant::now();

    Ok(RuntimeSnapshot {
        total_signals: stats.total_signals,
        live_signals: stats.live_signals,
        total_scopes: stats.total_scopes,
        live_scopes: stats.live_scopes,
        total_subscribers: stats.total_subscribers,
        live_subscribers: stats.live_subscribers,
        signals: stats
            .signals
            .into_iter()
            .map(|s| SignalSnapshot {
                index: s.index,
                version: s.version,
                subscriber_count: s.subscriber_count,
                alive: s.alive,
            })
            .collect(),
        timestamp,
    })
}

/// Computes the difference between two runtime snapshots.
///
/// Produces a [`ProfileReport`] listing signals whose version changed and
/// the total elapsed time between the snapshots.
///
/// # Examples
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_signal, dispose_runtime};
/// use dusty_devtools::profiler::{snapshot_runtime, diff_snapshots};
///
/// initialize_runtime();
/// let sig = create_signal(0).unwrap();
/// let before = snapshot_runtime().unwrap();
/// sig.set(1).unwrap();
/// sig.set(2).unwrap();
/// let after = snapshot_runtime().unwrap();
///
/// let report = diff_snapshots(&before, &after);
/// assert_eq!(report.signal_deltas.len(), 1);
/// assert_eq!(report.signal_deltas[0].version_delta, 2);
/// dispose_runtime();
/// ```
#[must_use]
pub fn diff_snapshots(before: &RuntimeSnapshot, after: &RuntimeSnapshot) -> ProfileReport {
    let elapsed = after
        .timestamp
        .checked_duration_since(before.timestamp)
        .unwrap_or_default();

    let mut signal_deltas = Vec::new();

    for after_sig in &after.signals {
        let before_version = before
            .signals
            .iter()
            .find(|s| s.index == after_sig.index)
            .map_or(0, |s| s.version);

        let delta = after_sig.version.saturating_sub(before_version);
        if delta > 0 {
            signal_deltas.push(SignalDelta {
                index: after_sig.index,
                version_delta: delta,
            });
        }
    }

    ProfileReport {
        elapsed,
        signal_deltas,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn empty_timer_returns_none() {
        let timer = FrameTimer::new(60);
        assert!(timer.stats().is_none());
        assert!(timer.is_empty());
        assert_eq!(timer.len(), 0);
    }

    #[test]
    fn records_single_frame() {
        let mut timer = FrameTimer::new(60);
        timer.begin_frame();
        timer.end_frame();

        let stats = timer.stats().unwrap();
        assert_eq!(stats.frame_count, 1);
        assert!(stats.min > Duration::ZERO || stats.min == Duration::ZERO);
        assert_eq!(timer.len(), 1);
    }

    #[test]
    fn push_duration_directly() {
        let mut timer = FrameTimer::new(60);
        timer.push_duration(Duration::from_millis(16));

        let stats = timer.stats().unwrap();
        assert_eq!(stats.frame_count, 1);
        assert_eq!(stats.min, Duration::from_millis(16));
        assert_eq!(stats.max, Duration::from_millis(16));
        assert_eq!(stats.avg, Duration::from_millis(16));
        assert_eq!(stats.last, Duration::from_millis(16));
    }

    #[test]
    fn capacity_eviction() {
        let mut timer = FrameTimer::new(3);
        timer.push_duration(Duration::from_millis(10));
        timer.push_duration(Duration::from_millis(20));
        timer.push_duration(Duration::from_millis(30));
        assert_eq!(timer.len(), 3);

        // Push a 4th — oldest should be evicted
        timer.push_duration(Duration::from_millis(40));
        assert_eq!(timer.len(), 3);

        let stats = timer.stats().unwrap();
        assert_eq!(stats.min, Duration::from_millis(20));
        assert_eq!(stats.max, Duration::from_millis(40));
    }

    #[test]
    fn min_max_avg_calculation() {
        let mut timer = FrameTimer::new(10);
        timer.push_duration(Duration::from_millis(10));
        timer.push_duration(Duration::from_millis(20));
        timer.push_duration(Duration::from_millis(30));

        let stats = timer.stats().unwrap();
        assert_eq!(stats.min, Duration::from_millis(10));
        assert_eq!(stats.max, Duration::from_millis(30));
        assert_eq!(stats.avg, Duration::from_millis(20));
        assert_eq!(stats.last, Duration::from_millis(30));
    }

    #[test]
    fn p95_calculation() {
        let mut timer = FrameTimer::new(100);
        // Push 100 frames: 1ms, 2ms, ..., 100ms
        for i in 1..=100 {
            timer.push_duration(Duration::from_millis(i));
        }

        let stats = timer.stats().unwrap();
        assert_eq!(stats.frame_count, 100);
        // p95 of 1..=100 → 95th percentile is at index 94 (0-based) = 95ms
        assert_eq!(stats.p95, Duration::from_millis(95));
    }

    #[test]
    fn begin_without_end_ignored() {
        let mut timer = FrameTimer::new(60);
        timer.begin_frame();
        // No end_frame call

        assert!(timer.stats().is_none());
        assert!(timer.is_empty());

        // A new begin_frame should overwrite the pending start
        timer.begin_frame();
        timer.end_frame();
        assert_eq!(timer.len(), 1);
    }

    #[test]
    fn end_without_begin_ignored() {
        let mut timer = FrameTimer::new(60);
        timer.end_frame(); // No begin_frame
        assert!(timer.is_empty());
    }

    #[test]
    fn snapshot_reads_runtime_stats() {
        dusty_reactive::initialize_runtime();
        let _sig = dusty_reactive::create_signal(42).unwrap();

        let snap = snapshot_runtime().unwrap();
        assert_eq!(snap.live_signals, 1);
        assert!(!snap.signals.is_empty());
        assert!(snap.signals[0].alive);

        dusty_reactive::dispose_runtime();
    }

    #[test]
    fn diff_computes_deltas() {
        dusty_reactive::initialize_runtime();
        let sig = dusty_reactive::create_signal(0).unwrap();

        let before = snapshot_runtime().unwrap();
        sig.set(1).unwrap();
        sig.set(2).unwrap();
        let after = snapshot_runtime().unwrap();

        let report = diff_snapshots(&before, &after);
        assert_eq!(report.signal_deltas.len(), 1);
        assert_eq!(report.signal_deltas[0].version_delta, 2);

        dusty_reactive::dispose_runtime();
    }

    #[test]
    fn diff_no_changes_empty_deltas() {
        dusty_reactive::initialize_runtime();
        let _sig = dusty_reactive::create_signal(0).unwrap();

        let before = snapshot_runtime().unwrap();
        let after = snapshot_runtime().unwrap();

        let report = diff_snapshots(&before, &after);
        assert!(report.signal_deltas.is_empty());

        dusty_reactive::dispose_runtime();
    }

    #[test]
    fn diff_new_signal_in_after() {
        dusty_reactive::initialize_runtime();

        let before = snapshot_runtime().unwrap();
        let sig = dusty_reactive::create_signal(0).unwrap();
        sig.set(1).unwrap();
        let after = snapshot_runtime().unwrap();

        let report = diff_snapshots(&before, &after);
        // New signal with version 1
        assert_eq!(report.signal_deltas.len(), 1);
        assert_eq!(report.signal_deltas[0].version_delta, 1);

        dusty_reactive::dispose_runtime();
    }
}
