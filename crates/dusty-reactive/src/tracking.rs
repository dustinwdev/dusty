//! Shared tracking helpers for reactive dependency management.
//!
//! Consolidates duplicated patterns from memo.rs, effect.rs, and signal.rs:
//! unsubscribing from deps, notifying subscribers, and test runtime setup.

use crate::error::Result;
use crate::runtime::{with_runtime_mut, SignalId};
use crate::subscriber::{invoke_subscriber, SubscriberId};

/// Unsubscribe `sub_id` from every signal in `deps`.
///
/// Used by both memo and effect during re-evaluation and disposal.
pub fn unsubscribe_from_signals(sub_id: SubscriberId, deps: impl IntoIterator<Item = SignalId>) {
    for signal_id in deps {
        let _ = with_runtime_mut(|rt| {
            if let Some(slot) = rt.signals.get_mut(signal_id.index) {
                if slot.alive && slot.generation == signal_id.generation {
                    slot.subscribers.remove(&sub_id);
                }
            }
        });
    }
}

/// Invoke each subscriber in the collection. Errors are propagated.
pub fn notify_subscribers(subs: impl IntoIterator<Item = SubscriberId>) -> Result<()> {
    for sub_id in subs {
        invoke_subscriber(sub_id)?;
    }
    Ok(())
}

/// Test-only helper: initialize runtime, run closure, dispose runtime.
#[cfg(test)]
pub fn with_test_runtime(f: impl FnOnce()) {
    crate::runtime::initialize_runtime();
    f();
    crate::runtime::dispose_runtime();
}
