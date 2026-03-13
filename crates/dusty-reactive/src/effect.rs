//! Effects — side effects that re-run when their dependencies change.
//!
//! An effect is like a memo but eager: it runs immediately on creation and
//! re-runs whenever any dependency changes. Effects produce no value but can
//! register cleanup functions via [`on_cleanup`].
//!
//! # Examples
//!
//! ```
//! # dusty_reactive::initialize_runtime();
//! let count = dusty_reactive::create_signal(0).unwrap();
//! let effect = dusty_reactive::create_effect(move || {
//!     let _val = count.get().unwrap();
//!     // side effect: log, update DOM, etc.
//! }).unwrap();
//!
//! count.set(1).unwrap(); // effect re-runs
//! dusty_reactive::dispose_effect(&effect).unwrap();
//! # dusty_reactive::dispose_runtime();
//! ```

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;

use crate::error::{ReactiveError, Result};
use crate::runtime::{with_runtime, with_runtime_mut, SignalId};
use crate::subscriber::{
    pop_tracking, push_tracking, register_subscriber, unregister_subscriber, SubscriberId,
};

// ---------------------------------------------------------------------------
// Freshener lookup (shared with memo.rs)
// ---------------------------------------------------------------------------

type FreshenerFn = Rc<dyn Fn() -> Result<()>>;

fn get_freshener(index: usize) -> Option<FreshenerFn> {
    crate::memo::get_freshener_pub(index)
}

// ---------------------------------------------------------------------------
// Dependency info
// ---------------------------------------------------------------------------

struct DepInfo {
    signal_id: SignalId,
    #[allow(dead_code)]
    version: u64,
    #[allow(dead_code)]
    freshener: Option<FreshenerFn>,
}

// ---------------------------------------------------------------------------
// Thread-locals for cleanup sink and pending effects
// ---------------------------------------------------------------------------

type CleanupVec = Vec<Box<dyn FnOnce()>>;

thread_local! {
    /// During effect execution, this holds the vec that `on_cleanup` pushes into.
    static CLEANUP_SINK: RefCell<Option<CleanupVec>> = const { RefCell::new(None) };

    /// Effects that were marked dirty during signal notification and need re-execution.
    static PENDING_EFFECTS: RefCell<Vec<Rc<EffectInner>>> = const { RefCell::new(Vec::new()) };
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A reactive effect that re-runs when its dependencies change.
///
/// `Effect` is `Clone` (via `Rc`), not `Copy`.
pub struct Effect {
    state: Rc<EffectInner>,
    _not_send: PhantomData<*const ()>,
}

impl std::fmt::Debug for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Effect")
            .field("subscriber_id", &self.state.subscriber_id)
            .finish_non_exhaustive()
    }
}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
            _not_send: PhantomData,
        }
    }
}

struct EffectInner {
    f: Box<dyn Fn()>,
    subscriber_id: SubscriberId,
    dirty: Rc<Cell<bool>>,
    deps: RefCell<Vec<DepInfo>>,
    cleanups: RefCell<Vec<Box<dyn FnOnce()>>>,
    disposed: Cell<bool>,
    running: Cell<bool>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create an effect that runs immediately and re-runs whenever its
/// dependencies change.
///
/// # Errors
///
/// Returns [`ReactiveError::NoRuntime`] if no runtime is initialized.
pub fn create_effect(f: impl Fn() + 'static) -> Result<Effect> {
    let dirty = Rc::new(Cell::new(false));
    let dirty_for_cb = Rc::clone(&dirty);

    // We need to create the subscriber first, then create EffectInner,
    // then set up the callback to reference the EffectInner.
    // Use a two-phase approach: register a dummy subscriber, create the state,
    // then replace the callback.

    // Phase 1: allocate subscriber with a placeholder
    let state_slot: Rc<RefCell<Option<Rc<EffectInner>>>> = Rc::new(RefCell::new(None));
    let state_slot_for_cb = Rc::clone(&state_slot);

    let subscriber_id = register_subscriber(move || {
        dirty_for_cb.set(true);
        // Queue for deferred execution
        if let Some(state) = state_slot_for_cb.borrow().as_ref() {
            PENDING_EFFECTS.with(|pe| {
                pe.borrow_mut().push(Rc::clone(state));
            });
        }
    })?;

    let state = Rc::new(EffectInner {
        f: Box::new(f),
        subscriber_id,
        dirty,
        deps: RefCell::new(Vec::new()),
        cleanups: RefCell::new(Vec::new()),
        disposed: Cell::new(false),
        running: Cell::new(false),
    });

    // Wire up the callback's reference to state
    *state_slot.borrow_mut() = Some(Rc::clone(&state));

    // Register disposer with current scope if any
    let state_for_scope = Rc::clone(&state);
    crate::scope::register_disposer(Box::new(move || {
        let _ = dispose_effect_inner(&state_for_scope);
    }))?;

    // Run immediately
    execute_effect(&state)?;

    Ok(Effect {
        state,
        _not_send: PhantomData,
    })
}

/// Dispose of an effect, running its cleanup functions and unsubscribing.
///
/// # Errors
///
/// Returns [`ReactiveError::EffectDisposed`] if the effect was already disposed.
pub fn dispose_effect(effect: &Effect) -> Result<()> {
    dispose_effect_inner(&effect.state)
}

/// Register a cleanup function that will run before the effect re-executes
/// or when the effect is disposed.
///
/// If called outside of an effect, this is a no-op.
pub fn on_cleanup(cleanup: impl FnOnce() + 'static) {
    CLEANUP_SINK.with(|sink| {
        if let Ok(mut borrow) = sink.try_borrow_mut() {
            if let Some(ref mut vec) = *borrow {
                vec.push(Box::new(cleanup));
            }
        }
    });
}

/// Clear auxiliary thread-locals. Called by `dispose_runtime` to prevent
/// stale closures from surviving across runtime re-initialization.
pub(crate) fn clear_thread_locals() {
    CLEANUP_SINK.with(|sink| {
        *sink.borrow_mut() = None;
    });
    PENDING_EFFECTS.with(|pe| {
        pe.borrow_mut().clear();
    });
}

/// Drain and execute all pending dirty effects.
///
/// Called at the end of `set_and_notify` to ensure effects run after
/// all signal notifications are complete and borrows are released.
pub(crate) fn flush_pending_effects() {
    const MAX_ITERATIONS: usize = 100;

    for _ in 0..MAX_ITERATIONS {
        let pending: Vec<Rc<EffectInner>> =
            PENDING_EFFECTS.with(|pe| std::mem::take(&mut *pe.borrow_mut()));

        if pending.is_empty() {
            break;
        }

        for state in pending {
            if !state.disposed.get() && state.dirty.get() {
                let _ = execute_effect(&state);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Internal
// ---------------------------------------------------------------------------

fn execute_effect(state: &Rc<EffectInner>) -> Result<()> {
    if state.disposed.get() {
        return Ok(());
    }

    // Re-entrancy guard
    if state.running.get() {
        return Ok(());
    }
    state.running.set(true);

    // Run existing cleanups in reverse (LIFO)
    let old_cleanups = std::mem::take(&mut *state.cleanups.borrow_mut());
    for cleanup in old_cleanups.into_iter().rev() {
        cleanup();
    }

    // Unsubscribe from old deps
    let old_deps = std::mem::take(&mut *state.deps.borrow_mut());
    let sub_id = state.subscriber_id;
    for dep in &old_deps {
        let _ = with_runtime_mut(|rt| {
            if let Some(slot) = rt.signals.get_mut(dep.signal_id.index) {
                if slot.alive && slot.generation == dep.signal_id.generation {
                    slot.subscribers.remove(&sub_id);
                }
            }
        });
    }

    // Set up cleanup sink
    CLEANUP_SINK.with(|sink| {
        *sink.borrow_mut() = Some(Vec::new());
    });

    // Push subscriber onto tracking stack, run f, pop
    push_tracking(sub_id)?;

    let f_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (state.f)();
    }));

    // Always pop tracking and take cleanup sink, whether f() succeeded or panicked
    let pop_result = pop_tracking();
    let new_cleanups = CLEANUP_SINK.with(|sink| sink.borrow_mut().take().unwrap_or_default());

    // If f() panicked, dispose the effect and re-panic
    if let Err(payload) = f_result {
        state.disposed.set(true);
        state.running.set(false);
        let _ = unregister_subscriber(state.subscriber_id);
        // Unsubscribe from signals tracked during the partial execution
        if let Ok(ref signal_ids) = pop_result {
            let sub_id = state.subscriber_id;
            for sig_id in signal_ids {
                let _ = with_runtime_mut(|rt| {
                    if let Some(slot) = rt.signals.get_mut(sig_id.index) {
                        if slot.alive && slot.generation == sig_id.generation {
                            slot.subscribers.remove(&sub_id);
                        }
                    }
                });
            }
        }
        std::panic::resume_unwind(payload);
    }

    *state.cleanups.borrow_mut() = new_cleanups;

    // Capture new deps
    let new_signal_ids = pop_result?;
    let new_deps: Vec<DepInfo> = with_runtime(|rt| {
        new_signal_ids
            .iter()
            .map(|&id| {
                let version = rt.signals.get(id.index).map_or(0, |slot| slot.version);
                let freshener = get_freshener(id.index);
                DepInfo {
                    signal_id: id,
                    version,
                    freshener,
                }
            })
            .collect()
    })?;
    *state.deps.borrow_mut() = new_deps;

    state.dirty.set(false);
    state.running.set(false);

    Ok(())
}

fn dispose_effect_inner(state: &EffectInner) -> Result<()> {
    if state.disposed.get() {
        return Err(ReactiveError::EffectDisposed);
    }
    state.disposed.set(true);

    // Run cleanups in reverse
    let cleanups = std::mem::take(&mut *state.cleanups.borrow_mut());
    for cleanup in cleanups.into_iter().rev() {
        cleanup();
    }

    // Unsubscribe from all deps
    let deps = std::mem::take(&mut *state.deps.borrow_mut());
    let sub_id = state.subscriber_id;
    for dep in &deps {
        let _ = with_runtime_mut(|rt| {
            if let Some(slot) = rt.signals.get_mut(dep.signal_id.index) {
                if slot.alive && slot.generation == dep.signal_id.generation {
                    slot.subscribers.remove(&sub_id);
                }
            }
        });
    }

    let _ = unregister_subscriber(sub_id);

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{dispose_runtime, initialize_runtime};
    use crate::signal::create_signal;
    use static_assertions::assert_not_impl_any;
    use std::cell::Cell;

    assert_not_impl_any!(Effect: Send, Sync);

    fn with_test_runtime(f: impl FnOnce()) {
        initialize_runtime();
        f();
        dispose_runtime();
    }

    #[test]
    fn effect_runs_on_creation() {
        with_test_runtime(|| {
            let ran = Rc::new(Cell::new(false));
            let r = Rc::clone(&ran);

            let _effect = create_effect(move || {
                r.set(true);
            })
            .unwrap();

            assert!(ran.get());
        });
    }

    #[test]
    fn effect_reruns_when_signal_changes() {
        with_test_runtime(|| {
            let count = create_signal(0).unwrap();
            let run_count = Rc::new(Cell::new(0));
            let rc = Rc::clone(&run_count);

            let _effect = create_effect(move || {
                let _val = count.get().unwrap();
                rc.set(rc.get() + 1);
            })
            .unwrap();

            assert_eq!(run_count.get(), 1); // initial run

            count.set(1).unwrap();
            assert_eq!(run_count.get(), 2); // re-run

            count.set(2).unwrap();
            assert_eq!(run_count.get(), 3); // re-run again
        });
    }

    #[test]
    fn effect_tracks_multiple_signals() {
        with_test_runtime(|| {
            let a = create_signal(1).unwrap();
            let b = create_signal(2).unwrap();
            let run_count = Rc::new(Cell::new(0));
            let rc = Rc::clone(&run_count);

            let _effect = create_effect(move || {
                let _va = a.get().unwrap();
                let _vb = b.get().unwrap();
                rc.set(rc.get() + 1);
            })
            .unwrap();

            assert_eq!(run_count.get(), 1);

            a.set(10).unwrap();
            assert_eq!(run_count.get(), 2);

            b.set(20).unwrap();
            assert_eq!(run_count.get(), 3);
        });
    }

    #[test]
    fn effect_cleanup_runs_before_rerun() {
        with_test_runtime(|| {
            let count = create_signal(0).unwrap();
            let log = Rc::new(RefCell::new(Vec::<String>::new()));
            let l = Rc::clone(&log);

            let _effect = create_effect(move || {
                let val = count.get().unwrap();
                let l2 = Rc::clone(&l);
                on_cleanup(move || {
                    l2.borrow_mut().push(format!("cleanup-{val}"));
                });
                l.borrow_mut().push(format!("run-{val}"));
            })
            .unwrap();

            assert_eq!(*log.borrow(), vec!["run-0"]);

            count.set(1).unwrap();
            assert_eq!(*log.borrow(), vec!["run-0", "cleanup-0", "run-1"]);

            count.set(2).unwrap();
            assert_eq!(
                *log.borrow(),
                vec!["run-0", "cleanup-0", "run-1", "cleanup-1", "run-2"]
            );
        });
    }

    #[test]
    fn effect_cleanup_runs_on_dispose() {
        with_test_runtime(|| {
            let cleaned = Rc::new(Cell::new(false));
            let c = Rc::clone(&cleaned);

            let effect = create_effect(move || {
                let c2 = Rc::clone(&c);
                on_cleanup(move || c2.set(true));
            })
            .unwrap();

            assert!(!cleaned.get());
            dispose_effect(&effect).unwrap();
            assert!(cleaned.get());
        });
    }

    #[test]
    fn effect_multiple_cleanups_reverse_order() {
        with_test_runtime(|| {
            let order = Rc::new(RefCell::new(Vec::new()));
            let o = Rc::clone(&order);

            let effect = create_effect(move || {
                let o1 = Rc::clone(&o);
                on_cleanup(move || o1.borrow_mut().push(1));
                let o2 = Rc::clone(&o);
                on_cleanup(move || o2.borrow_mut().push(2));
                let o3 = Rc::clone(&o);
                on_cleanup(move || o3.borrow_mut().push(3));
            })
            .unwrap();

            dispose_effect(&effect).unwrap();
            assert_eq!(*order.borrow(), vec![3, 2, 1]);
        });
    }

    #[test]
    fn effect_dynamic_dependencies() {
        with_test_runtime(|| {
            let flag = create_signal(true).unwrap();
            let a = create_signal(10).unwrap();
            let b = create_signal(20).unwrap();
            let observed = Rc::new(Cell::new(0));
            let ob = Rc::clone(&observed);

            let _effect = create_effect(move || {
                if flag.get().unwrap() {
                    ob.set(a.get().unwrap());
                } else {
                    ob.set(b.get().unwrap());
                }
            })
            .unwrap();

            assert_eq!(observed.get(), 10);

            // b is not a dependency, changing it should not re-run
            let run_before = observed.get();
            b.set(30).unwrap();
            assert_eq!(observed.get(), run_before);

            // Switch to b branch
            flag.set(false).unwrap();
            assert_eq!(observed.get(), 30);

            // Now a is not a dependency
            a.set(99).unwrap();
            assert_eq!(observed.get(), 30);

            // b is a dependency
            b.set(50).unwrap();
            assert_eq!(observed.get(), 50);
        });
    }

    #[test]
    fn effect_untracked_no_subscription() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let run_count = Rc::new(Cell::new(0));
            let rc = Rc::clone(&run_count);

            let _effect = create_effect(move || {
                let _val = sig.with_untracked(|v| *v).unwrap();
                rc.set(rc.get() + 1);
            })
            .unwrap();

            assert_eq!(run_count.get(), 1);

            // Untracked read should not cause re-run
            sig.set(1).unwrap();
            assert_eq!(run_count.get(), 1);
        });
    }

    #[test]
    fn disposed_effect_does_not_rerun() {
        with_test_runtime(|| {
            let count = create_signal(0).unwrap();
            let run_count = Rc::new(Cell::new(0));
            let rc = Rc::clone(&run_count);

            let effect = create_effect(move || {
                let _val = count.get().unwrap();
                rc.set(rc.get() + 1);
            })
            .unwrap();

            assert_eq!(run_count.get(), 1);

            dispose_effect(&effect).unwrap();

            count.set(1).unwrap();
            assert_eq!(run_count.get(), 1); // no re-run
        });
    }

    #[test]
    fn effect_reentrance_guard() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let run_count = Rc::new(Cell::new(0));
            let rc = Rc::clone(&run_count);

            // Effect reads sig and writes to sig — should not infinite loop
            let _effect = create_effect(move || {
                let val = sig.get().unwrap();
                rc.set(rc.get() + 1);
                if val < 3 {
                    let _ = sig.set(val + 1);
                }
            })
            .unwrap();

            // Should have run a bounded number of times, not infinite
            assert!(run_count.get() < 200);
            assert!(run_count.get() >= 1);
        });
    }

    #[test]
    fn on_cleanup_outside_effect_is_noop() {
        with_test_runtime(|| {
            // Should not panic or do anything
            on_cleanup(|| {
                panic!("should never run");
            });
        });
    }

    #[test]
    fn double_dispose_effect_errors() {
        with_test_runtime(|| {
            let effect = create_effect(|| {}).unwrap();
            dispose_effect(&effect).unwrap();
            assert_eq!(
                dispose_effect(&effect).unwrap_err(),
                ReactiveError::EffectDisposed
            );
        });
    }

    #[test]
    fn effect_panic_restores_tracking_stack() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();

            // Create an effect that panics on first run
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                create_effect(move || {
                    let _val = sig.get().unwrap();
                    panic!("effect panic");
                })
                .unwrap();
            }));
            assert!(result.is_err());

            // Tracking stack should be clean — create another effect that works
            let observed = Rc::new(Cell::new(0));
            let ob = Rc::clone(&observed);

            let _effect2 = create_effect(move || {
                ob.set(sig.get().unwrap());
            })
            .unwrap();

            assert_eq!(observed.get(), 0);
            sig.set(5).unwrap();
            assert_eq!(observed.get(), 5);
        });
    }

    #[test]
    fn effect_panic_resets_running_flag() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let should_panic = Rc::new(Cell::new(true));
            let sp = Rc::clone(&should_panic);
            let run_count = Rc::new(Cell::new(0));
            let rc = Rc::clone(&run_count);

            // Effect that panics conditionally
            let effect = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                create_effect(move || {
                    let _val = sig.get().unwrap();
                    rc.set(rc.get() + 1);
                    if sp.get() {
                        panic!("effect panic");
                    }
                })
                .unwrap()
            }));

            // Effect panicked during creation, so it wasn't fully created
            assert!(effect.is_err());

            // But the runtime should still be usable
            should_panic.set(false);
            let observed = Rc::new(Cell::new(-1));
            let ob = Rc::clone(&observed);
            let _effect2 = create_effect(move || {
                ob.set(sig.get().unwrap());
            })
            .unwrap();

            sig.set(42).unwrap();
            assert_eq!(observed.get(), 42);
        });
    }

    use std::cell::RefCell;
}
