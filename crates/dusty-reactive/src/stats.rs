//! Runtime statistics for devtools integration.
//!
//! Provides a read-only snapshot of the reactive runtime's internal state
//! for diagnostic and profiling purposes. Only compiled when the `devtools`
//! feature is enabled.

use crate::error::Result;
use crate::runtime::with_runtime;

/// Snapshot of the reactive runtime's internal state.
///
/// # Examples
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_signal, dispose_runtime};
/// use dusty_reactive::stats::runtime_stats;
///
/// initialize_runtime();
/// let _sig = create_signal(42);
/// let stats = runtime_stats().unwrap();
/// assert_eq!(stats.live_signals, 1);
/// dispose_runtime();
/// ```
#[derive(Debug, Clone)]
pub struct RuntimeStats {
    /// Total signal slots ever allocated (including disposed).
    pub total_signals: usize,
    /// Signals currently alive.
    pub live_signals: usize,
    /// Total scope slots ever allocated (including disposed).
    pub total_scopes: usize,
    /// Scopes currently alive.
    pub live_scopes: usize,
    /// Total subscriber slots ever allocated (including disposed).
    pub total_subscribers: usize,
    /// Subscribers currently alive.
    pub live_subscribers: usize,
    /// Current batch nesting depth (0 = not batching).
    pub batch_depth: usize,
    /// Per-signal statistics for all signal slots.
    pub signals: Vec<SignalStats>,
}

/// Statistics for a single signal slot.
#[derive(Debug, Clone)]
pub struct SignalStats {
    /// Slot index in the signal slab.
    pub index: usize,
    /// Version counter — incremented on each value change.
    pub version: u64,
    /// Number of subscribers currently registered.
    pub subscriber_count: usize,
    /// Whether this signal slot is alive.
    pub alive: bool,
}

/// Captures a snapshot of the reactive runtime's current state.
///
/// # Errors
///
/// Returns [`ReactiveError::NoRuntime`](crate::ReactiveError::NoRuntime) if
/// no runtime is initialized on the current thread.
///
/// # Examples
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_signal, dispose_runtime};
/// use dusty_reactive::stats::runtime_stats;
///
/// initialize_runtime();
/// let _ = create_signal(10);
/// let _ = create_signal(20);
///
/// let stats = runtime_stats().unwrap();
/// assert_eq!(stats.live_signals, 2);
/// assert_eq!(stats.total_signals, 2);
///
/// dispose_runtime();
/// ```
pub fn runtime_stats() -> Result<RuntimeStats> {
    with_runtime(|rt| {
        let signals = rt
            .signals
            .iter()
            .enumerate()
            .map(|(index, slot)| SignalStats {
                index,
                version: slot.version,
                subscriber_count: slot.subscribers.len(),
                alive: slot.alive,
            })
            .collect();

        RuntimeStats {
            total_signals: rt.signals.len(),
            live_signals: rt.signals.iter().filter(|s| s.alive).count(),
            total_scopes: rt.scopes.len(),
            live_scopes: rt.scopes.iter().filter(|s| s.alive).count(),
            total_subscribers: rt.subscribers.len(),
            live_subscribers: rt.subscribers.iter().filter(|s| s.is_some()).count(),
            batch_depth: rt.batch_depth,
            signals,
        }
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::runtime::{dispose_runtime, initialize_runtime};

    fn with_test_runtime(f: impl FnOnce()) {
        initialize_runtime();
        f();
        dispose_runtime();
    }

    #[test]
    fn no_runtime_returns_error() {
        dispose_runtime();
        let result = runtime_stats();
        assert!(result.is_err());
    }

    #[test]
    fn empty_runtime_zero_counts() {
        with_test_runtime(|| {
            let stats = runtime_stats().unwrap();
            assert_eq!(stats.total_signals, 0);
            assert_eq!(stats.live_signals, 0);
            assert_eq!(stats.total_scopes, 0);
            assert_eq!(stats.live_scopes, 0);
            assert_eq!(stats.total_subscribers, 0);
            assert_eq!(stats.live_subscribers, 0);
            assert_eq!(stats.batch_depth, 0);
            assert!(stats.signals.is_empty());
        });
    }

    #[test]
    fn counts_live_signals() {
        with_test_runtime(|| {
            let _s1 = crate::signal::create_signal(1);
            let _s2 = crate::signal::create_signal(2);

            let stats = runtime_stats().unwrap();
            assert_eq!(stats.total_signals, 2);
            assert_eq!(stats.live_signals, 2);
            assert_eq!(stats.signals.len(), 2);
        });
    }

    #[test]
    fn disposed_signal_tracked() {
        with_test_runtime(|| {
            let s1 = crate::signal::create_signal(1);
            let _s2 = crate::signal::create_signal(2);
            crate::signal::dispose_signal(s1);

            let stats = runtime_stats().unwrap();
            assert_eq!(stats.total_signals, 2);
            assert_eq!(stats.live_signals, 1);
            assert!(!stats.signals[0].alive);
            assert!(stats.signals[1].alive);
        });
    }

    #[test]
    fn signal_version_increments() {
        with_test_runtime(|| {
            let s = crate::signal::create_signal(0);
            s.set(1);
            s.set(2);

            let stats = runtime_stats().unwrap();
            assert_eq!(stats.signals[0].version, 2);
        });
    }

    #[test]
    fn subscriber_count_tracked() {
        with_test_runtime(|| {
            let s = crate::signal::create_signal(0);
            let _effect = crate::effect::create_effect(move || {
                let _ = s.get();
            });

            let stats = runtime_stats().unwrap();
            assert!(stats.signals[0].subscriber_count > 0);
        });
    }

    #[test]
    fn live_subscribers_tracked() {
        with_test_runtime(|| {
            let _s = crate::signal::create_signal(0);
            let _effect = crate::effect::create_effect(|| {});

            let stats = runtime_stats().unwrap();
            assert!(stats.live_subscribers > 0);
        });
    }

    #[test]
    fn scope_counts() {
        with_test_runtime(|| {
            let _scope = crate::scope::create_scope(|_cx| {});

            let stats = runtime_stats().unwrap();
            assert!(stats.total_scopes > 0);
            assert!(stats.live_scopes > 0);
        });
    }

    #[test]
    fn batch_depth_zero_outside_batch() {
        with_test_runtime(|| {
            let stats = runtime_stats().unwrap();
            assert_eq!(stats.batch_depth, 0);
        });
    }
}
