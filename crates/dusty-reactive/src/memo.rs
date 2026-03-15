//! Memos — cached derived computations that auto-track dependencies.
//!
//! A memo lazily re-evaluates when its dependencies change. It caches the
//! result and only notifies downstream subscribers when the value actually
//! changes (via `PartialEq`).
//!
//! # Examples
//!
//! ```
//! # dusty_reactive::initialize_runtime();
//! let count = dusty_reactive::create_signal(2).unwrap();
//! let doubled = dusty_reactive::create_memo(move || count.get().unwrap() * 2).unwrap();
//! assert_eq!(doubled.get().unwrap(), 4);
//!
//! count.set(3).unwrap();
//! assert_eq!(doubled.get().unwrap(), 6);
//! # dusty_reactive::dispose_runtime();
//! ```

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;

use smallvec::SmallVec;

use crate::error::{ReactiveError, Result};
use crate::runtime::{with_runtime, with_runtime_mut, FreshenerFn, SignalId};
use crate::signal::{create_signal_raw, track_signal, with_signal_value};
use crate::subscriber::{
    invoke_subscriber, pop_tracking, push_tracking, register_subscriber, unregister_subscriber,
    SubscriberId,
};
use crate::tracking::unsubscribe_from_signals;

// ---------------------------------------------------------------------------
// Freshener registry helpers — stored in the Runtime
// ---------------------------------------------------------------------------

fn register_freshener(index: usize, f: FreshenerFn) {
    let _ = with_runtime_mut(|rt| {
        rt.fresheners.insert(index, f);
    });
}

fn unregister_freshener(index: usize) {
    let _ = with_runtime_mut(|rt| {
        rt.fresheners.remove(&index);
    });
}

// ---------------------------------------------------------------------------
// Dependency info — signal ID + version snapshot + optional freshener
// ---------------------------------------------------------------------------

struct DepInfo {
    signal_id: SignalId,
    version: u64,
    freshener: Option<FreshenerFn>,
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A cached derived computation that auto-tracks its dependencies.
///
/// `Memo<T>` is `Clone` (via `Rc`), not `Copy`. The cached value lives in a
/// signal slot so downstream memos/effects can subscribe to it through the
/// existing tracking infrastructure.
///
/// # Examples
///
/// ```
/// # dusty_reactive::initialize_runtime();
/// let a = dusty_reactive::create_signal(1).unwrap();
/// let b = dusty_reactive::create_signal(2).unwrap();
/// let sum = dusty_reactive::create_memo(move || a.get().unwrap() + b.get().unwrap()).unwrap();
/// assert_eq!(sum.get().unwrap(), 3);
///
/// a.set(10).unwrap();
/// assert_eq!(sum.get().unwrap(), 12);
/// # dusty_reactive::dispose_runtime();
/// ```
pub struct Memo<T: 'static> {
    signal_id: SignalId,
    state: Rc<MemoInner<T>>,
    _not_send: PhantomData<*const ()>,
}

impl<T: 'static> std::fmt::Debug for Memo<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Memo")
            .field("signal_id", &self.signal_id)
            .finish_non_exhaustive()
    }
}

impl<T: 'static> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            signal_id: self.signal_id,
            state: Rc::clone(&self.state),
            _not_send: PhantomData,
        }
    }
}

struct MemoInner<T> {
    computation: Box<dyn Fn() -> T>,
    dirty: Rc<Cell<bool>>,
    deps: RefCell<SmallVec<[DepInfo; 4]>>,
    subscriber_id: SubscriberId,
    signal_id: SignalId,
    disposed: Cell<bool>,
}

impl<T> Drop for MemoInner<T> {
    fn drop(&mut self) {
        // Clean up stale freshener entry if memo was dropped without explicit disposal
        if !self.disposed.get() {
            unregister_freshener(self.signal_id.index);
        }
    }
}

// ---------------------------------------------------------------------------
// Creation
// ---------------------------------------------------------------------------

/// Create a memo — a cached derived computation that auto-tracks dependencies.
///
/// The computation `f` is called lazily on the first `.get()` and re-evaluated
/// only when a dependency actually changes value. If the new value equals the
/// old value (via `PartialEq`), downstream subscribers are not notified.
///
/// # Errors
///
/// Returns [`ReactiveError::NoRuntime`] if no runtime is initialized.
///
/// # Examples
///
/// ```
/// # dusty_reactive::initialize_runtime();
/// let count = dusty_reactive::create_signal(5).unwrap();
/// let doubled = dusty_reactive::create_memo(move || count.get().unwrap() * 2).unwrap();
/// assert_eq!(doubled.get().unwrap(), 10);
/// # dusty_reactive::dispose_runtime();
/// ```
pub fn create_memo<T>(f: impl Fn() -> T + 'static) -> Result<Memo<T>>
where
    T: Clone + PartialEq + 'static,
{
    let signal_id = create_signal_raw::<Option<T>>(None)?;

    let dirty = Rc::new(Cell::new(true));
    let dirty_for_cb = Rc::clone(&dirty);
    let signal_id_for_cb = signal_id;

    let subscriber_id = register_subscriber(move || {
        dirty_for_cb.set(true);
        propagate_dirty(signal_id_for_cb);
    })?;

    let state = Rc::new(MemoInner {
        computation: Box::new(f),
        dirty,
        deps: RefCell::new(SmallVec::new()),
        subscriber_id,
        signal_id,
        disposed: Cell::new(false),
    });

    let freshener: FreshenerFn = {
        let weak = Rc::downgrade(&state);
        let sid = signal_id;
        Rc::new(move || {
            weak.upgrade()
                .map_or(Ok(()), |st| ensure_fresh_inner::<T>(&st, sid))
        })
    };

    register_freshener(signal_id.index, Rc::clone(&freshener));

    // Register a disposer with the current scope (if any)
    let memo_state = Rc::clone(&state);
    let memo_signal_id = signal_id;
    crate::scope::register_disposer(Box::new(move || {
        let memo_ref = Memo {
            signal_id: memo_signal_id,
            state: memo_state,
            _not_send: PhantomData,
        };
        let _ = dispose_memo(&memo_ref);
    }))?;

    Ok(Memo {
        signal_id,
        state,
        _not_send: PhantomData,
    })
}

/// Propagate dirty flag to a memo's downstream subscribers.
///
/// This is always called from within subscriber callbacks (during
/// notification), when `batch_depth` is 0. The generation check in
/// `invoke_subscriber` ensures stale references are silently skipped.
fn propagate_dirty(signal_id: SignalId) {
    // INVARIANT: Subscribers are collected into a SmallVec under an immutable
    // borrow (`with_runtime`), the borrow is released, THEN subscribers are
    // invoked. This ordering is critical — inlining invocation into the borrow
    // would cause a `RuntimeBorrowError` if any subscriber writes to a signal.
    let subs: std::result::Result<SmallVec<[SubscriberId; 8]>, _> = with_runtime(|rt| {
        rt.signals
            .get(signal_id.index)
            .filter(|slot| slot.alive && slot.generation == signal_id.generation)
            .map_or_else(SmallVec::new, |slot| {
                slot.subscribers.iter().copied().collect()
            })
    });

    if let Ok(subs) = subs {
        for sub_id in subs {
            let _ = invoke_subscriber(sub_id);
        }
    }
}

// ---------------------------------------------------------------------------
// Disposal
// ---------------------------------------------------------------------------

/// Dispose of a memo, cleaning up its subscriptions and freeing its signal slot.
///
/// After disposal, all operations on this memo will return
/// [`ReactiveError::MemoDisposed`].
///
/// # Errors
///
/// Returns [`ReactiveError::MemoDisposed`] if the memo was already disposed.
pub fn dispose_memo<T: 'static>(memo: &Memo<T>) -> Result<()> {
    if memo.state.disposed.get() {
        return Err(ReactiveError::MemoDisposed);
    }
    memo.state.disposed.set(true);

    let deps = std::mem::take(&mut *memo.state.deps.borrow_mut());
    let sub_id = memo.state.subscriber_id;
    unsubscribe_from_signals(sub_id, deps.iter().map(|d| d.signal_id));

    let _ = unregister_subscriber(sub_id);
    unregister_freshener(memo.signal_id.index);

    with_runtime_mut(|rt| -> std::result::Result<(), ReactiveError> {
        let slot = rt
            .signals
            .get_mut(memo.signal_id.index)
            .ok_or(ReactiveError::MemoDisposed)?;
        if !slot.alive || slot.generation != memo.signal_id.generation {
            return Err(ReactiveError::MemoDisposed);
        }
        // Clear subscriber list but do NOT unregister downstream subscribers.
        // They may depend on other signals and must remain functional.
        slot.subscribers.clear();
        slot.alive = false;
        rt.free_list.push(memo.signal_id.index);
        Ok(())
    })?
}

// ---------------------------------------------------------------------------
// Core ensure-fresh logic (standalone fn so the freshener closure can call it)
// ---------------------------------------------------------------------------

fn ensure_fresh_inner<T: Clone + PartialEq + 'static>(
    state: &MemoInner<T>,
    signal_id: SignalId,
) -> Result<()> {
    if !state.dirty.get() {
        // Outside a batch, the dirty flag is reliable: subscriber callbacks
        // run synchronously on signal changes.
        // Inside a batch, callbacks are deferred, so the dirty flag may be
        // stale — fall through to dep-version check.
        let in_batch = with_runtime(|rt| rt.batch_depth > 0).unwrap_or(false);
        if !in_batch {
            return Ok(());
        }
    }

    // Ensure all deps are up-to-date, then check if any actually changed.
    if freshen_deps_and_check_unchanged(state)? {
        state.dirty.set(false);
        return Ok(());
    }

    evaluate_memo(state, signal_id)
}

/// Ensure all dep memos are fresh, then return `true` if NO dep version changed.
fn freshen_deps_and_check_unchanged<T>(state: &MemoInner<T>) -> Result<bool> {
    let deps = state.deps.borrow();
    if deps.is_empty() {
        return Ok(false); // first evaluation — must run
    }

    for dep in deps.iter() {
        if let Some(ref freshener) = dep.freshener {
            freshener()?;
        }
    }

    let any_changed = with_runtime(|rt| {
        deps.iter().any(|dep| {
            rt.signals
                .get(dep.signal_id.index)
                .map_or(true, |slot| slot.version != dep.version)
        })
    })?;

    Ok(!any_changed)
}

/// Full re-evaluation: unsubscribe from old deps, run computation, compare,
/// store result, and notify downstream if the value changed.
fn evaluate_memo<T: Clone + PartialEq + 'static>(
    state: &MemoInner<T>,
    signal_id: SignalId,
) -> Result<()> {
    let old_deps = std::mem::take(&mut *state.deps.borrow_mut());
    let sub_id = state.subscriber_id;
    unsubscribe_from_signals(sub_id, old_deps.iter().map(|d| d.signal_id));

    push_tracking(sub_id)?;

    let compute_result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (state.computation)()));

    let pop_result = pop_tracking();

    let new_value = match compute_result {
        Ok(val) => val,
        Err(payload) => std::panic::resume_unwind(payload),
    };

    let new_signal_ids = pop_result?;

    let new_deps: SmallVec<[DepInfo; 4]> = with_runtime(|rt| {
        new_signal_ids
            .iter()
            .map(|&id| {
                let version = rt.signals.get(id.index).map_or(0, |slot| slot.version);
                let freshener = rt.fresheners.get(&id.index).cloned();
                DepInfo {
                    signal_id: id,
                    version,
                    freshener,
                }
            })
            .collect()
    })?;
    *state.deps.borrow_mut() = new_deps;

    let changed = with_signal_value::<Option<T>, bool>(signal_id, |opt| {
        opt.as_ref().map_or(true, |old| *old != new_value)
    })?;

    update_memo_slot::<T>(signal_id, new_value, changed)?;

    state.dirty.set(false);
    Ok(())
}

/// Update the signal slot value, optionally bumping version and notifying downstream.
///
/// When `notify` is true, this is batch-aware: if `batch_depth > 0`, subscribers
/// are queued to `pending_batch_subscribers` instead of being invoked immediately.
#[allow(clippy::type_complexity)]
fn update_memo_slot<T: 'static>(signal_id: SignalId, new_value: T, notify: bool) -> Result<()> {
    let maybe_subs = with_runtime_mut(
        |rt| -> std::result::Result<Option<(SmallVec<[SubscriberId; 8]>, bool)>, ReactiveError> {
            let slot = rt
                .signals
                .get_mut(signal_id.index)
                .ok_or(ReactiveError::MemoDisposed)?;
            if !slot.alive || slot.generation != signal_id.generation {
                return Err(ReactiveError::MemoDisposed);
            }
            // TypeMismatch should be unreachable through the safe API.
            let value = slot
                .value
                .downcast_mut::<Option<T>>()
                .ok_or(ReactiveError::TypeMismatch)?;
            *value = Some(new_value);
            if notify {
                slot.version += 1;
                let subs: SmallVec<[SubscriberId; 8]> = slot.subscribers.iter().copied().collect();
                let batching = rt.batch_depth > 0;
                if batching {
                    rt.pending_batch_subscribers.extend(subs.iter().copied());
                }
                Ok(Some((subs, batching)))
            } else {
                Ok(None)
            }
        },
    )??;

    if let Some((subs, in_batch)) = maybe_subs {
        if !in_batch {
            crate::tracking::notify_subscribers(subs)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Read cached value from slot (returns error instead of panicking)
// ---------------------------------------------------------------------------

fn read_cached<T: 'static, R>(signal_id: SignalId, f: impl FnOnce(&T) -> R) -> Result<R> {
    with_signal_value::<Option<T>, Result<R>>(signal_id, |opt| {
        opt.as_ref().map(f).ok_or(ReactiveError::MemoDisposed)
    })?
}

// ---------------------------------------------------------------------------
// Memo<T> impl
// ---------------------------------------------------------------------------

impl<T: Clone + PartialEq + 'static> Memo<T> {
    /// Get the memo's current value, re-evaluating if dirty.
    /// Registers the caller as a subscriber.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the memo is disposed.
    pub fn get(&self) -> Result<T> {
        self.with(T::clone)
    }

    /// Access the memo's cached value by reference. Re-evaluates if dirty.
    /// Registers the caller as a subscriber.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the memo is disposed.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R> {
        if self.state.disposed.get() {
            return Err(ReactiveError::MemoDisposed);
        }
        ensure_fresh_inner::<T>(&self.state, self.signal_id).map_err(to_memo_error)?;
        track_signal(self.signal_id).map_err(to_memo_error)?;
        read_cached::<T, R>(self.signal_id, f).map_err(to_memo_error)
    }

    /// Access the memo's cached value by reference without registering as a subscriber.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the memo is disposed.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R> {
        if self.state.disposed.get() {
            return Err(ReactiveError::MemoDisposed);
        }
        ensure_fresh_inner::<T>(&self.state, self.signal_id).map_err(to_memo_error)?;
        read_cached::<T, R>(self.signal_id, f).map_err(to_memo_error)
    }
}

/// Map `SignalDisposed` to `MemoDisposed` at the public API boundary so
/// users never see internal signal-layer errors from memo operations.
const fn to_memo_error(e: ReactiveError) -> ReactiveError {
    match e {
        ReactiveError::SignalDisposed => ReactiveError::MemoDisposed,
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::dispose_runtime;
    use crate::signal::create_signal;
    use crate::tracking::with_test_runtime;
    use static_assertions::assert_not_impl_any;
    use std::cell::Cell;

    assert_not_impl_any!(Memo<i32>: Send, Sync);

    // -- Step 2: Basic memo creation and read --

    #[test]
    fn memo_returns_computed_value() {
        with_test_runtime(|| {
            let memo = create_memo(|| 42).unwrap();
            assert_eq!(memo.get().unwrap(), 42);
        });
    }

    #[test]
    fn memo_is_clone() {
        with_test_runtime(|| {
            let memo = create_memo(|| 10).unwrap();
            let memo2 = memo.clone();
            assert_eq!(memo.get().unwrap(), 10);
            assert_eq!(memo2.get().unwrap(), 10);
        });
    }

    #[test]
    fn create_memo_no_runtime_returns_error() {
        dispose_runtime();
        let result = create_memo(|| 0);
        assert_eq!(result.unwrap_err(), ReactiveError::NoRuntime);
    }

    // -- Step 3: Auto-tracking --

    #[test]
    fn memo_updates_when_signal_changes() {
        with_test_runtime(|| {
            let count = create_signal(2).unwrap();
            let doubled = create_memo(move || count.get().unwrap() * 2).unwrap();

            assert_eq!(doubled.get().unwrap(), 4);

            count.set(5).unwrap();
            assert_eq!(doubled.get().unwrap(), 10);
        });
    }

    #[test]
    fn memo_tracks_multiple_signals() {
        with_test_runtime(|| {
            let a = create_signal(1).unwrap();
            let b = create_signal(2).unwrap();
            let sum = create_memo(move || a.get().unwrap() + b.get().unwrap()).unwrap();

            assert_eq!(sum.get().unwrap(), 3);

            a.set(10).unwrap();
            assert_eq!(sum.get().unwrap(), 12);

            b.set(20).unwrap();
            assert_eq!(sum.get().unwrap(), 30);
        });
    }

    #[test]
    fn memo_with_ref_access() {
        with_test_runtime(|| {
            let name = create_signal(String::from("hello")).unwrap();
            let upper = create_memo(move || name.get().unwrap().to_uppercase()).unwrap();

            let len = upper.with(|s| s.len()).unwrap();
            assert_eq!(len, 5);

            name.set(String::from("world!")).unwrap();
            let len = upper.with(|s| s.len()).unwrap();
            assert_eq!(len, 6);
        });
    }

    // -- Step 4: Lazy re-evaluation / caching --

    #[test]
    fn memo_does_not_recompute_when_not_dirty() {
        with_test_runtime(|| {
            let count = create_signal(1).unwrap();
            let eval_count = Rc::new(Cell::new(0));
            let ec = Rc::clone(&eval_count);

            let memo = create_memo(move || {
                ec.set(ec.get() + 1);
                count.get().unwrap() * 2
            })
            .unwrap();

            assert_eq!(memo.get().unwrap(), 2);
            assert_eq!(eval_count.get(), 1);

            assert_eq!(memo.get().unwrap(), 2);
            assert_eq!(eval_count.get(), 1);

            assert_eq!(memo.get().unwrap(), 2);
            assert_eq!(eval_count.get(), 1);
        });
    }

    #[test]
    fn memo_recomputes_only_when_dependency_changes() {
        with_test_runtime(|| {
            let a = create_signal(1).unwrap();
            let b = create_signal(100).unwrap();
            let eval_count = Rc::new(Cell::new(0));
            let ec = Rc::clone(&eval_count);

            let memo = create_memo(move || {
                ec.set(ec.get() + 1);
                a.get().unwrap() + b.get().unwrap()
            })
            .unwrap();

            assert_eq!(memo.get().unwrap(), 101);
            assert_eq!(eval_count.get(), 1);

            a.set(2).unwrap();
            assert_eq!(memo.get().unwrap(), 102);
            assert_eq!(eval_count.get(), 2);

            assert_eq!(memo.get().unwrap(), 102);
            assert_eq!(eval_count.get(), 2);
        });
    }

    // -- Step 5: Value equality prevents spurious downstream notifications --

    #[test]
    fn memo_equality_prevents_downstream_update() {
        with_test_runtime(|| {
            let input = create_signal(3).unwrap();

            let clamped = create_memo(move || {
                let v = input.get().unwrap();
                v.min(5)
            })
            .unwrap();

            let downstream_count = Rc::new(Cell::new(0));
            let dc = Rc::clone(&downstream_count);

            let downstream = create_memo(move || {
                dc.set(dc.get() + 1);
                clamped.get().unwrap() * 10
            })
            .unwrap();

            assert_eq!(downstream.get().unwrap(), 30);
            assert_eq!(downstream_count.get(), 1);

            // clamped 3 → 5 (changed), downstream re-evals
            input.set(10).unwrap();
            assert_eq!(downstream.get().unwrap(), 50);
            assert_eq!(downstream_count.get(), 2);

            // clamped stays 5, downstream skips eval
            input.set(20).unwrap();
            assert_eq!(downstream.get().unwrap(), 50);
            assert_eq!(downstream_count.get(), 2);
        });
    }

    // -- Step 6: Diamond dependency --

    #[test]
    fn diamond_dependency_evaluates_each_memo_once() {
        with_test_runtime(|| {
            let source = create_signal(1).unwrap();

            let a_count = Rc::new(Cell::new(0));
            let b_count = Rc::new(Cell::new(0));
            let c_count = Rc::new(Cell::new(0));

            let ac = Rc::clone(&a_count);
            let bc = Rc::clone(&b_count);
            let cc = Rc::clone(&c_count);

            let a = create_memo(move || {
                ac.set(ac.get() + 1);
                source.get().unwrap() * 2
            })
            .unwrap();

            let b = create_memo(move || {
                bc.set(bc.get() + 1);
                source.get().unwrap() * 3
            })
            .unwrap();

            let c = create_memo(move || {
                cc.set(cc.get() + 1);
                a.get().unwrap() + b.get().unwrap()
            })
            .unwrap();

            assert_eq!(c.get().unwrap(), 5);
            assert_eq!(a_count.get(), 1);
            assert_eq!(b_count.get(), 1);
            assert_eq!(c_count.get(), 1);

            source.set(2).unwrap();
            assert_eq!(c.get().unwrap(), 10);
            assert_eq!(a_count.get(), 2);
            assert_eq!(b_count.get(), 2);
            assert_eq!(c_count.get(), 2);
        });
    }

    #[test]
    fn diamond_produces_correct_value() {
        with_test_runtime(|| {
            let s = create_signal(10).unwrap();
            let left = create_memo(move || s.get().unwrap() + 1).unwrap();
            let right = create_memo(move || s.get().unwrap() * 2).unwrap();
            let combined = create_memo(move || left.get().unwrap() + right.get().unwrap()).unwrap();

            assert_eq!(combined.get().unwrap(), 31);

            s.set(5).unwrap();
            assert_eq!(combined.get().unwrap(), 16);
        });
    }

    // -- Step 7: Chained memos --

    #[test]
    fn chained_memos_propagate_correctly() {
        with_test_runtime(|| {
            let source = create_signal(1).unwrap();
            let m1 = create_memo(move || source.get().unwrap() * 2).unwrap();
            let m2 = create_memo(move || m1.get().unwrap() + 10).unwrap();
            let m3 = create_memo(move || m2.get().unwrap() * 3).unwrap();

            assert_eq!(m3.get().unwrap(), 36);

            source.set(5).unwrap();
            assert_eq!(m3.get().unwrap(), 60);
        });
    }

    #[test]
    fn chained_memos_evaluate_in_order() {
        with_test_runtime(|| {
            let source = create_signal(0).unwrap();
            let order = Rc::new(RefCell::new(Vec::new()));

            let o1 = Rc::clone(&order);
            let m1 = create_memo(move || {
                o1.borrow_mut().push(1);
                source.get().unwrap() + 1
            })
            .unwrap();

            let o2 = Rc::clone(&order);
            let m2 = create_memo(move || {
                o2.borrow_mut().push(2);
                m1.get().unwrap() + 1
            })
            .unwrap();

            let o3 = Rc::clone(&order);
            let m3 = create_memo(move || {
                o3.borrow_mut().push(3);
                m2.get().unwrap() + 1
            })
            .unwrap();

            // Initial: pull-based, m3 pulls m2 pulls m1
            assert_eq!(m3.get().unwrap(), 3);
            assert_eq!(*order.borrow(), vec![3, 2, 1]);

            order.borrow_mut().clear();

            // Subsequent: fresheners ensure bottom-up (m1, m2, m3)
            source.set(10).unwrap();
            assert_eq!(m3.get().unwrap(), 13);
            assert_eq!(*order.borrow(), vec![1, 2, 3]);
        });
    }

    #[test]
    fn deeply_chained_memos() {
        with_test_runtime(|| {
            let source = create_signal(1).unwrap();

            let mut prev: Option<Memo<i32>> = None;
            let mut memos = Vec::new();

            for _ in 0..10 {
                let memo = if let Some(p) = prev.clone() {
                    create_memo(move || p.get().unwrap() + 1).unwrap()
                } else {
                    create_memo(move || source.get().unwrap()).unwrap()
                };
                prev = Some(memo.clone());
                memos.push(memo);
            }

            assert_eq!(memos[9].get().unwrap(), 10);

            source.set(100).unwrap();
            assert_eq!(memos[9].get().unwrap(), 109);
        });
    }

    // -- Step 8: Dynamic dependencies --

    #[test]
    fn memo_tracks_new_dependencies_on_reevaluation() {
        with_test_runtime(|| {
            let flag = create_signal(true).unwrap();
            let a = create_signal(10).unwrap();
            let b = create_signal(20).unwrap();

            let eval_count = Rc::new(Cell::new(0));
            let ec = Rc::clone(&eval_count);

            let memo = create_memo(move || {
                ec.set(ec.get() + 1);
                if flag.get().unwrap() {
                    a.get().unwrap()
                } else {
                    b.get().unwrap()
                }
            })
            .unwrap();

            assert_eq!(memo.get().unwrap(), 10);
            assert_eq!(eval_count.get(), 1);

            // b is not a dependency
            b.set(30).unwrap();
            assert_eq!(memo.get().unwrap(), 10);
            assert_eq!(eval_count.get(), 1);

            // Switch branch
            flag.set(false).unwrap();
            assert_eq!(memo.get().unwrap(), 30);
            assert_eq!(eval_count.get(), 2);

            // a is no longer a dependency
            a.set(99).unwrap();
            assert_eq!(memo.get().unwrap(), 30);
            assert_eq!(eval_count.get(), 2);

            // b is now a dependency
            b.set(50).unwrap();
            assert_eq!(memo.get().unwrap(), 50);
            assert_eq!(eval_count.get(), 3);
        });
    }

    // -- Step 9: Untracked read --

    #[test]
    fn memo_with_untracked_does_not_register_reader() {
        with_test_runtime(|| {
            let source = create_signal(5).unwrap();
            let memo = create_memo(move || source.get().unwrap() * 2).unwrap();

            let val = memo.with_untracked(|v| *v).unwrap();
            assert_eq!(val, 10);

            let sub_count =
                with_runtime(|rt| rt.signals[memo.signal_id.index].subscribers.len()).unwrap();
            assert_eq!(sub_count, 0);
        });
    }

    // -- Step 10: Disposal --

    #[test]
    fn disposed_memo_returns_error() {
        with_test_runtime(|| {
            let memo = create_memo(|| 42).unwrap();
            assert_eq!(memo.get().unwrap(), 42);

            dispose_memo(&memo).unwrap();
            assert_eq!(memo.get().unwrap_err(), ReactiveError::MemoDisposed);
        });
    }

    #[test]
    fn disposing_memo_cleans_up_subscriptions() {
        with_test_runtime(|| {
            let source = create_signal(1).unwrap();
            let memo = create_memo(move || source.get().unwrap() * 2).unwrap();

            assert_eq!(memo.get().unwrap(), 2);

            let sub_id = memo.state.subscriber_id;
            dispose_memo(&memo).unwrap();

            let is_none = with_runtime(|rt| rt.subscribers[sub_id.index].is_none()).unwrap();
            assert!(is_none);
        });
    }

    #[test]
    fn memo_panic_restores_tracking_stack() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let should_panic = Rc::new(Cell::new(true));
            let sp = Rc::clone(&should_panic);

            let memo = create_memo(move || {
                let val = sig.get().unwrap();
                if sp.get() {
                    panic!("memo computation panic");
                }
                val * 2
            })
            .unwrap();

            // First .get() triggers evaluation which panics
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                memo.get().unwrap();
            }));
            assert!(result.is_err());

            // Tracking stack should be clean — verify by creating an effect that works
            let observed = Rc::new(Cell::new(0));
            let ob = Rc::clone(&observed);

            let _effect = crate::effect::create_effect(move || {
                ob.set(sig.get().unwrap());
            })
            .unwrap();

            sig.set(5).unwrap();
            assert_eq!(observed.get(), 5);
        });
    }
}
