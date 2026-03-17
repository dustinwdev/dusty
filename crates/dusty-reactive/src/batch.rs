//! Batching — coalesce multiple signal writes into a single notification pass.
//!
//! Inside a `batch()`, signal writes are recorded but subscribers are not
//! notified until the outermost batch completes. This avoids redundant
//! recomputation when multiple signals change together.
//!
//! # Examples
//!
//! ```
//! # dusty_reactive::initialize_runtime();
//! let a = dusty_reactive::create_signal(0);
//! let b = dusty_reactive::create_signal(0);
//!
//! dusty_reactive::batch(|| {
//!     a.set(1);
//!     b.set(2);
//! });
//!
//! assert_eq!(a.get(), 1);
//! assert_eq!(b.get(), 2);
//! # dusty_reactive::dispose_runtime();
//! ```

use std::collections::HashSet;

use crate::error::{unwrap_reactive, Result};
use crate::runtime::with_runtime_mut;
use crate::subscriber::SubscriberId;

/// Drop guard that decrements batch depth and flushes on both normal
/// return and unwind, preventing permanent "stuck in batch" state.
struct BatchGuard;

impl Drop for BatchGuard {
    fn drop(&mut self) {
        let pending = with_runtime_mut(|rt| {
            rt.batch_depth -= 1;
            if rt.batch_depth == 0 {
                Some(std::mem::take(&mut rt.pending_batch_subscribers))
            } else {
                None
            }
        });
        if let Ok(Some(subs)) = pending {
            let result = flush_batch(subs);
            if let Err(e) = result {
                eprintln!("dusty: flush_batch failed: {e:?}");
            }
        }
    }
}

/// Run a closure with batched notifications.
///
/// All signal writes inside `f` are coalesced — subscribers are notified
/// only once when the outermost `batch` returns. Nested `batch` calls are
/// supported; only the outermost flush triggers notifications.
///
/// # Panics
///
/// Panics if no runtime is initialized.
pub fn batch<T>(f: impl FnOnce() -> T) -> T {
    unwrap_reactive(try_batch(f), "batch")
}

/// Fallible version of [`batch`]. Returns an error instead of panicking
/// if no runtime is initialized.
///
/// # Errors
///
/// Returns an error if no runtime is initialized.
pub fn try_batch<T>(f: impl FnOnce() -> T) -> Result<T> {
    // Increment batch depth
    with_runtime_mut(|rt| {
        rt.batch_depth += 1;
    })?;

    let guard = BatchGuard;
    let result = f();
    drop(guard);

    Ok(result)
}

/// Invoke each queued subscriber once, then flush effects.
///
/// The `HashSet` guarantees uniqueness — no deduplication needed.
fn flush_batch(subs: HashSet<SubscriberId>) -> Result<()> {
    for sub_id in subs {
        crate::subscriber::invoke_subscriber(sub_id)?;
    }

    crate::effect::flush_pending_effects();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::dispose_runtime;
    use crate::signal::create_signal;
    use crate::tracking::with_test_runtime;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn batch_returns_closure_value() {
        with_test_runtime(|| {
            let val = batch(|| 42);
            assert_eq!(val, 42);
        });
    }

    #[test]
    fn batch_coalesces_notifications() {
        with_test_runtime(|| {
            let sig = create_signal(0);
            let run_count = Rc::new(Cell::new(0));
            let rc = Rc::clone(&run_count);

            let _effect = crate::effect::create_effect(move || {
                let _val = sig.get();
                rc.set(rc.get() + 1);
            });

            assert_eq!(run_count.get(), 1); // initial

            batch(|| {
                sig.set(1);
                sig.set(2);
                sig.set(3);
            });

            // Effect should re-run exactly once after batch
            assert_eq!(run_count.get(), 2);
            assert_eq!(sig.get(), 3);
        });
    }

    #[test]
    fn batch_multiple_signals_single_notification() {
        with_test_runtime(|| {
            let a = create_signal(0);
            let b = create_signal(0);
            let run_count = Rc::new(Cell::new(0));
            let rc = Rc::clone(&run_count);

            let _effect = crate::effect::create_effect(move || {
                let _va = a.get();
                let _vb = b.get();
                rc.set(rc.get() + 1);
            });

            assert_eq!(run_count.get(), 1);

            batch(|| {
                a.set(10);
                b.set(20);
            });

            // Effect re-runs once, not twice
            assert_eq!(run_count.get(), 2);
            assert_eq!(a.get(), 10);
            assert_eq!(b.get(), 20);
        });
    }

    #[test]
    fn batch_deduplicates_subscribers() {
        with_test_runtime(|| {
            let sig = create_signal(0);
            let call_count = Rc::new(Cell::new(0));
            let cc = Rc::clone(&call_count);

            let _effect = crate::effect::create_effect(move || {
                let _val = sig.get();
                cc.set(cc.get() + 1);
            });

            assert_eq!(call_count.get(), 1);

            batch(|| {
                sig.set(1);
                sig.set(2);
            });

            // Subscriber called once despite two writes
            assert_eq!(call_count.get(), 2);
        });
    }

    #[test]
    fn batch_nested_only_outermost_flushes() {
        with_test_runtime(|| {
            let sig = create_signal(0);
            let run_count = Rc::new(Cell::new(0));
            let rc = Rc::clone(&run_count);

            let _effect = crate::effect::create_effect(move || {
                let _val = sig.get();
                rc.set(rc.get() + 1);
            });

            assert_eq!(run_count.get(), 1);

            batch(|| {
                sig.set(1);

                batch(|| {
                    sig.set(2);
                });

                // Inner batch should NOT have triggered flush
                assert_eq!(run_count.get(), 1);

                sig.set(3);
            });

            // Only outermost batch triggers flush
            assert_eq!(run_count.get(), 2);
            assert_eq!(sig.get(), 3);
        });
    }

    #[test]
    fn batch_empty_is_noop() {
        with_test_runtime(|| {
            batch(|| {});
        });
    }

    #[test]
    fn batch_no_runtime_returns_error() {
        dispose_runtime();
        let result = try_batch(|| 42);
        assert!(result.is_err());
    }

    #[test]
    fn batch_panic_restores_runtime() {
        with_test_runtime(|| {
            let sig = create_signal(0);
            let observed = Rc::new(Cell::new(0));
            let ob = Rc::clone(&observed);

            let _effect = crate::effect::create_effect(move || {
                ob.set(sig.get());
            });

            assert_eq!(observed.get(), 0);

            // Panic inside batch
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                batch(|| {
                    sig.set(1);
                    panic!("test panic inside batch");
                });
            }));
            assert!(result.is_err());

            // Runtime should still work — signal notifications must function
            sig.set(2);
            assert_eq!(observed.get(), 2);
        });
    }

    #[test]
    fn batch_with_effects_deferred() {
        with_test_runtime(|| {
            let sig = create_signal(0);
            let observed = Rc::new(Cell::new(0));
            let ob = Rc::clone(&observed);

            let _effect = crate::effect::create_effect(move || {
                ob.set(sig.get());
            });

            assert_eq!(observed.get(), 0);

            batch(|| {
                sig.set(5);
                // Effect has not run yet — still in batch
                assert_eq!(observed.get(), 0);
            });

            // After batch, effect sees the final value
            assert_eq!(observed.get(), 5);
        });
    }

    #[test]
    fn batch_with_memos_coherent() {
        with_test_runtime(|| {
            let a = create_signal(1);
            let b = create_signal(2);
            let sum = crate::memo::create_memo(move || a.get() + b.get());

            assert_eq!(sum.get(), 3);

            batch(|| {
                a.set(10);
                b.set(20);
            });

            // Memo reads correct values after batch
            assert_eq!(sum.get(), 30);
        });
    }
}
