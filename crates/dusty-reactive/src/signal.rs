//! Signal types — the core reactive primitive.
//!
//! A signal holds a value and notifies subscribers when it changes.
//! Signals are `Copy` handles backed by generational indices into the
//! thread-local runtime.

use std::collections::HashSet;
use std::marker::PhantomData;

use crate::error::{ReactiveError, Result};
use crate::runtime::{with_runtime, with_runtime_mut, SignalId, SignalSlot};
use crate::subscriber::SubscriberId;

/// A reactive signal with both read and write access.
///
/// `Signal<T>` is a `Copy` handle — it does not own the value.
/// The value lives in the thread-local runtime.
///
/// # Examples
///
/// ```
/// # dusty_reactive::initialize_runtime();
/// let count = dusty_reactive::create_signal(0)?;
/// assert_eq!(count.get()?, 0);
///
/// count.set(5)?;
/// assert_eq!(count.get()?, 5);
///
/// count.update(|n| *n += 1)?;
/// assert_eq!(count.get()?, 6);
/// # dusty_reactive::dispose_runtime();
/// # Ok::<(), dusty_reactive::ReactiveError>(())
/// ```
pub struct Signal<T: 'static> {
    id: SignalId,
    _marker: PhantomData<fn() -> T>,
    _not_send: PhantomData<*const ()>,
}

impl<T: 'static> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal").field("id", &self.id).finish()
    }
}

impl<T: 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for Signal<T> {}

/// Read-only handle to a signal.
///
/// Obtained via [`create_signal_split`] or [`Signal::read`].
pub struct ReadSignal<T: 'static> {
    id: SignalId,
    _marker: PhantomData<fn() -> T>,
    _not_send: PhantomData<*const ()>,
}

impl<T: 'static> std::fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadSignal").field("id", &self.id).finish()
    }
}

impl<T: 'static> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for ReadSignal<T> {}

/// Write-only handle to a signal.
///
/// Obtained via [`create_signal_split`] or [`Signal::write`].
pub struct WriteSignal<T: 'static> {
    id: SignalId,
    _marker: PhantomData<fn() -> T>,
    _not_send: PhantomData<*const ()>,
}

impl<T: 'static> std::fmt::Debug for WriteSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteSignal").field("id", &self.id).finish()
    }
}

impl<T: 'static> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for WriteSignal<T> {}

// ---------------------------------------------------------------------------
// Creation
// ---------------------------------------------------------------------------

/// Create a signal with the given initial value. Returns a combined
/// read/write handle.
///
/// # Errors
///
/// Returns [`ReactiveError::NoRuntime`] if no runtime is initialized.
///
/// # Examples
///
/// ```
/// # dusty_reactive::initialize_runtime();
/// let sig = dusty_reactive::create_signal(42)?;
/// assert_eq!(sig.get()?, 42);
/// # dusty_reactive::dispose_runtime();
/// # Ok::<(), dusty_reactive::ReactiveError>(())
/// ```
pub fn create_signal<T: 'static>(value: T) -> Result<Signal<T>> {
    let id = create_signal_raw(value)?;
    Ok(Signal {
        id,
        _marker: PhantomData,
        _not_send: PhantomData,
    })
}

/// Create a signal and immediately split it into read and write handles.
///
/// # Errors
///
/// Returns [`ReactiveError::NoRuntime`] if no runtime is initialized.
pub fn create_signal_split<T: 'static>(value: T) -> Result<(ReadSignal<T>, WriteSignal<T>)> {
    let id = create_signal_raw(value)?;
    Ok((
        ReadSignal {
            id,
            _marker: PhantomData,
            _not_send: PhantomData,
        },
        WriteSignal {
            id,
            _marker: PhantomData,
            _not_send: PhantomData,
        },
    ))
}

pub(crate) fn create_signal_raw<T: 'static>(value: T) -> Result<SignalId> {
    let id = with_runtime_mut(|rt| {
        if let Some(index) = rt.free_list.pop() {
            let generation = rt.signals[index].generation + 1;
            rt.signals[index] = SignalSlot {
                value: Box::new(value),
                generation,
                subscribers: HashSet::new(),
                alive: true,
                version: 0,
            };
            SignalId { index, generation }
        } else {
            let index = rt.signals.len();
            rt.signals.push(SignalSlot {
                value: Box::new(value),
                generation: 0,
                subscribers: HashSet::new(),
                alive: true,
                version: 0,
            });
            SignalId {
                index,
                generation: 0,
            }
        }
    })?;

    // Register a disposer with the current scope (if any)
    let signal_id = id;
    crate::scope::register_disposer(Box::new(move || {
        let _ = dispose_signal_raw(signal_id);
    }))?;

    Ok(id)
}

// ---------------------------------------------------------------------------
// Disposal
// ---------------------------------------------------------------------------

/// Dispose of a signal, freeing its slot for reuse.
///
/// After disposal, all operations on this signal will return
/// [`ReactiveError::SignalDisposed`].
///
/// # Errors
///
/// Returns [`ReactiveError::SignalDisposed`] if the signal was already disposed.
pub fn dispose_signal<T: 'static>(signal: Signal<T>) -> Result<()> {
    dispose_signal_raw(signal.id)
}

fn dispose_signal_raw(id: SignalId) -> Result<()> {
    // Collect subscriber IDs to clean up, then mark slot dead.
    let subscriber_ids = with_runtime_mut(
        |rt| -> std::result::Result<HashSet<SubscriberId>, ReactiveError> {
            let slot = rt
                .signals
                .get_mut(id.index)
                .ok_or(ReactiveError::SignalDisposed)?;
            if !slot.alive || slot.generation != id.generation {
                return Err(ReactiveError::SignalDisposed);
            }
            let subs = std::mem::take(&mut slot.subscribers);
            slot.alive = false;
            rt.free_list.push(id.index);
            Ok(subs)
        },
    )??;

    // Unregister each subscriber
    for sub_id in subscriber_ids {
        // Best effort — subscriber may already be gone
        let _ = crate::subscriber::unregister_subscriber(sub_id);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Access a signal's value by reference. Validates generation.
pub(crate) fn with_signal_value<T: 'static, R>(id: SignalId, f: impl FnOnce(&T) -> R) -> Result<R> {
    with_runtime(|rt| -> std::result::Result<R, ReactiveError> {
        let slot = rt
            .signals
            .get(id.index)
            .ok_or(ReactiveError::SignalDisposed)?;
        if !slot.alive || slot.generation != id.generation {
            return Err(ReactiveError::SignalDisposed);
        }
        // TypeMismatch should be unreachable through the safe API — it would
        // indicate an internal bug where the slot is accessed with the wrong type.
        let value = slot
            .value
            .downcast_ref::<T>()
            .ok_or(ReactiveError::TypeMismatch)?;
        Ok(f(value))
    })?
}

/// Combined track + read in a single mutable borrow cycle.
///
/// Reduces `ReadSignal::get()` from 3 separate borrow cycles (`current_tracking`,
/// `track_signal`, `with_signal_value`) down to 1. This is the hot path for signal reads.
pub(crate) fn track_and_read<T: 'static, R>(id: SignalId, f: impl FnOnce(&T) -> R) -> Result<R> {
    with_runtime_mut(|rt| -> std::result::Result<R, ReactiveError> {
        // Track (if active)
        if let Some(&sub_id) = rt.tracking_stack.last() {
            if let Some(slot) = rt.signals.get_mut(id.index) {
                if slot.alive && slot.generation == id.generation {
                    slot.subscribers.insert(sub_id);
                }
            }
            if let Some(deps) = rt.dependency_stack.last_mut() {
                if !deps.contains(&id) {
                    deps.push(id);
                }
            }
        }
        // Read
        let slot = rt
            .signals
            .get(id.index)
            .ok_or(ReactiveError::SignalDisposed)?;
        if !slot.alive || slot.generation != id.generation {
            return Err(ReactiveError::SignalDisposed);
        }
        let value = slot
            .value
            .downcast_ref::<T>()
            .ok_or(ReactiveError::TypeMismatch)?;
        Ok(f(value))
    })?
}

/// Register the current tracking context as a subscriber to this signal.
/// Also records the signal in the dependency stack for memo dependency tracking.
pub(crate) fn track_signal(id: SignalId) -> Result<()> {
    let maybe_sub = crate::subscriber::current_tracking()?;
    if let Some(sub_id) = maybe_sub {
        with_runtime_mut(|rt| {
            if let Some(slot) = rt.signals.get_mut(id.index) {
                if slot.alive && slot.generation == id.generation {
                    slot.subscribers.insert(sub_id);
                }
            }
            // Record this signal as a dependency of the current tracking scope
            if let Some(deps) = rt.dependency_stack.last_mut() {
                if !deps.contains(&id) {
                    deps.push(id);
                }
            }
        })?;
    }
    Ok(())
}

/// Collect-then-notify pattern: set or update the value while holding a
/// mutable borrow, collect subscriber IDs, release borrow, then notify.
pub(crate) fn set_and_notify<T: 'static>(id: SignalId, mutate: impl FnOnce(&mut T)) -> Result<()> {
    // Phase 1: mutate value + collect subscribers, check batch state
    let (subs, in_batch) = with_runtime_mut(
        |rt| -> std::result::Result<(HashSet<SubscriberId>, bool), ReactiveError> {
            let slot = rt
                .signals
                .get_mut(id.index)
                .ok_or(ReactiveError::SignalDisposed)?;
            if !slot.alive || slot.generation != id.generation {
                return Err(ReactiveError::SignalDisposed);
            }
            // TypeMismatch should be unreachable through the safe API.
            let value = slot
                .value
                .downcast_mut::<T>()
                .ok_or(ReactiveError::TypeMismatch)?;
            mutate(value);
            slot.version += 1;
            let subs = slot.subscribers.clone();
            let batching = rt.batch_depth > 0;
            if batching {
                rt.pending_batch_subscribers.extend(subs.iter().copied());
            }
            Ok((subs, batching))
        },
    )??;

    if !in_batch {
        // Phase 2: notify subscribers outside the mutable borrow.
        // Each callback is invoked via an immutable borrow with a
        // generation check to skip stale subscriber references.
        for sub_id in subs {
            crate::subscriber::invoke_subscriber(sub_id)?;
        }

        // Phase 3: flush any pending effects that were queued during notification.
        crate::effect::flush_pending_effects();
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// ReadSignal<T> impl
// ---------------------------------------------------------------------------

impl<T: 'static> ReadSignal<T> {
    /// Get the signal's current value by cloning it. Registers the current
    /// tracking context as a subscriber.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the signal is disposed.
    pub fn get(&self) -> Result<T>
    where
        T: Clone,
    {
        track_and_read(self.id, T::clone)
    }

    /// Access the signal's value by reference without cloning. Registers
    /// the current tracking context as a subscriber.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the signal is disposed.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R> {
        track_and_read(self.id, f)
    }

    /// Access the signal's value by reference without tracking.
    /// No subscriber registration occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the signal is disposed.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R> {
        with_signal_value(self.id, f)
    }
}

// ---------------------------------------------------------------------------
// WriteSignal<T> impl
// ---------------------------------------------------------------------------

impl<T: 'static> WriteSignal<T> {
    /// Replace the signal's value and notify all subscribers.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the signal is disposed.
    pub fn set(&self, value: T) -> Result<()> {
        set_and_notify::<T>(self.id, move |v| *v = value)
    }

    /// Mutate the signal's value in place and notify all subscribers.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the signal is disposed.
    pub fn update(&self, f: impl FnOnce(&mut T)) -> Result<()> {
        set_and_notify::<T>(self.id, f)
    }
}

// ---------------------------------------------------------------------------
// Signal<T> impl — delegates to Read + Write
// ---------------------------------------------------------------------------

impl<T: 'static> Signal<T> {
    /// Split into separate read and write handles.
    #[must_use]
    pub fn split(self) -> (ReadSignal<T>, WriteSignal<T>) {
        (self.read(), self.write())
    }

    /// Get a read-only handle to this signal.
    #[must_use]
    pub fn read(&self) -> ReadSignal<T> {
        ReadSignal {
            id: self.id,
            _marker: PhantomData,
            _not_send: PhantomData,
        }
    }

    /// Get a write-only handle to this signal.
    #[must_use]
    pub fn write(&self) -> WriteSignal<T> {
        WriteSignal {
            id: self.id,
            _marker: PhantomData,
            _not_send: PhantomData,
        }
    }

    /// Get the signal's current value by cloning it. Registers tracking.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the signal is disposed.
    pub fn get(&self) -> Result<T>
    where
        T: Clone,
    {
        self.read().get()
    }

    /// Access the signal's value by reference. Registers tracking.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the signal is disposed.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R> {
        self.read().with(f)
    }

    /// Access the signal's value by reference without tracking.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the signal is disposed.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R> {
        self.read().with_untracked(f)
    }

    /// Replace the signal's value and notify subscribers.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the signal is disposed.
    pub fn set(&self, value: T) -> Result<()> {
        self.write().set(value)
    }

    /// Mutate the signal's value in place and notify subscribers.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the signal is disposed.
    pub fn update(&self, f: impl FnOnce(&mut T)) -> Result<()> {
        self.write().update(f)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{dispose_runtime, initialize_runtime};
    use static_assertions::assert_not_impl_any;
    use std::cell::Cell;
    use std::rc::Rc;

    assert_not_impl_any!(Signal<i32>: Send, Sync);
    assert_not_impl_any!(ReadSignal<i32>: Send, Sync);
    assert_not_impl_any!(WriteSignal<i32>: Send, Sync);

    fn with_test_runtime(f: impl FnOnce()) {
        initialize_runtime();
        f();
        dispose_runtime();
    }

    // -- Creation + Read --

    #[test]
    fn create_signal_and_get() {
        with_test_runtime(|| {
            let sig = create_signal(42).unwrap();
            assert_eq!(sig.get().unwrap(), 42);
        });
    }

    #[test]
    fn create_signal_no_runtime() {
        dispose_runtime();
        let result = create_signal(0);
        assert_eq!(result.unwrap_err(), ReactiveError::NoRuntime);
    }

    #[test]
    fn signal_with_ref_access() {
        with_test_runtime(|| {
            let sig = create_signal(String::from("hello")).unwrap();
            let len = sig.with(|s| s.len()).unwrap();
            assert_eq!(len, 5);
        });
    }

    #[test]
    fn signal_with_untracked() {
        with_test_runtime(|| {
            let sig = create_signal(10).unwrap();
            let val = sig.with_untracked(|v| *v).unwrap();
            assert_eq!(val, 10);
        });
    }

    #[test]
    fn signal_is_copy() {
        with_test_runtime(|| {
            let sig = create_signal(1).unwrap();
            let sig2 = sig;
            // Both should work — Copy semantics
            assert_eq!(sig.get().unwrap(), 1);
            assert_eq!(sig2.get().unwrap(), 1);
        });
    }

    #[test]
    fn read_signal_is_copy() {
        with_test_runtime(|| {
            let sig = create_signal(1).unwrap();
            let r = sig.read();
            let r2 = r;
            assert_eq!(r.get().unwrap(), 1);
            assert_eq!(r2.get().unwrap(), 1);
        });
    }

    #[test]
    fn write_signal_is_copy() {
        with_test_runtime(|| {
            let sig = create_signal(1).unwrap();
            let w = sig.write();
            let w2 = w;
            w.set(2).unwrap();
            w2.set(3).unwrap();
            assert_eq!(sig.get().unwrap(), 3);
        });
    }

    // -- Write + Update --

    #[test]
    fn set_changes_value() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            sig.set(99).unwrap();
            assert_eq!(sig.get().unwrap(), 99);
        });
    }

    #[test]
    fn multiple_sets() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            sig.set(1).unwrap();
            sig.set(2).unwrap();
            sig.set(3).unwrap();
            assert_eq!(sig.get().unwrap(), 3);
        });
    }

    #[test]
    fn update_mutates_in_place() {
        with_test_runtime(|| {
            let sig = create_signal(vec![1, 2, 3]).unwrap();
            sig.update(|v| v.push(4)).unwrap();
            let len = sig.with(|v| v.len()).unwrap();
            assert_eq!(len, 4);
        });
    }

    #[test]
    fn update_with_closure() {
        with_test_runtime(|| {
            let sig = create_signal(10).unwrap();
            sig.update(|n| *n *= 2).unwrap();
            assert_eq!(sig.get().unwrap(), 20);
        });
    }

    // -- Split access --

    #[test]
    fn split_returns_working_pair() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let (read, write) = sig.split();
            write.set(42).unwrap();
            assert_eq!(read.get().unwrap(), 42);
        });
    }

    #[test]
    fn create_signal_split_shares_state() {
        with_test_runtime(|| {
            let (read, write) = create_signal_split(100).unwrap();
            assert_eq!(read.get().unwrap(), 100);
            write.set(200).unwrap();
            assert_eq!(read.get().unwrap(), 200);
        });
    }

    #[test]
    fn read_write_from_signal() {
        with_test_runtime(|| {
            let sig = create_signal(5).unwrap();
            let r = sig.read();
            let w = sig.write();
            w.set(10).unwrap();
            assert_eq!(r.get().unwrap(), 10);
            // Original signal also sees the update
            assert_eq!(sig.get().unwrap(), 10);
        });
    }

    // -- Subscriber notification --

    #[test]
    fn set_notifies_subscriber() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let count = Rc::new(Cell::new(0));
            let count_clone = Rc::clone(&count);

            let sub_id = crate::subscriber::register_subscriber(move || {
                count_clone.set(count_clone.get() + 1);
            })
            .unwrap();

            // Manually subscribe to the signal
            crate::runtime::with_runtime_mut(|rt| {
                rt.signals[sig.read().id.index].subscribers.insert(sub_id);
            })
            .unwrap();

            sig.set(1).unwrap();
            assert_eq!(count.get(), 1);

            sig.set(2).unwrap();
            assert_eq!(count.get(), 2);
        });
    }

    #[test]
    fn update_notifies_subscriber() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let notified = Rc::new(Cell::new(false));
            let notified_clone = Rc::clone(&notified);

            let sub_id = crate::subscriber::register_subscriber(move || {
                notified_clone.set(true);
            })
            .unwrap();

            crate::runtime::with_runtime_mut(|rt| {
                rt.signals[sig.read().id.index].subscribers.insert(sub_id);
            })
            .unwrap();

            sig.update(|n| *n += 1).unwrap();
            assert!(notified.get());
        });
    }

    #[test]
    fn multiple_subscribers_notified() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let count_a = Rc::new(Cell::new(0));
            let count_b = Rc::new(Cell::new(0));

            let ca = Rc::clone(&count_a);
            let cb = Rc::clone(&count_b);

            let sub_a = crate::subscriber::register_subscriber(move || {
                ca.set(ca.get() + 1);
            })
            .unwrap();
            let sub_b = crate::subscriber::register_subscriber(move || {
                cb.set(cb.get() + 1);
            })
            .unwrap();

            crate::runtime::with_runtime_mut(|rt| {
                let subs = &mut rt.signals[sig.read().id.index].subscribers;
                subs.insert(sub_a);
                subs.insert(sub_b);
            })
            .unwrap();

            sig.set(1).unwrap();
            assert_eq!(count_a.get(), 1);
            assert_eq!(count_b.get(), 1);
        });
    }

    #[test]
    fn tracking_registers_subscriber() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let notified = Rc::new(Cell::new(false));
            let notified_clone = Rc::clone(&notified);

            let sub_id = crate::subscriber::register_subscriber(move || {
                notified_clone.set(true);
            })
            .unwrap();

            // Push subscriber onto tracking stack, then read
            crate::subscriber::push_tracking(sub_id).unwrap();
            let _val = sig.get().unwrap();
            crate::subscriber::pop_tracking().unwrap();

            // Now set should trigger notification
            sig.set(1).unwrap();
            assert!(notified.get());
        });
    }

    #[test]
    fn untracked_does_not_register() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let notified = Rc::new(Cell::new(false));
            let notified_clone = Rc::clone(&notified);

            let sub_id = crate::subscriber::register_subscriber(move || {
                notified_clone.set(true);
            })
            .unwrap();

            // Push subscriber onto tracking stack, but use untracked read
            crate::subscriber::push_tracking(sub_id).unwrap();
            let _val = sig.with_untracked(|v| *v).unwrap();
            crate::subscriber::pop_tracking().unwrap();

            // Set should NOT trigger notification
            sig.set(1).unwrap();
            assert!(!notified.get());
        });
    }

    #[test]
    fn unsubscribe_stops_notification() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let count = Rc::new(Cell::new(0));
            let count_clone = Rc::clone(&count);

            let sub_id = crate::subscriber::register_subscriber(move || {
                count_clone.set(count_clone.get() + 1);
            })
            .unwrap();

            crate::runtime::with_runtime_mut(|rt| {
                rt.signals[sig.read().id.index].subscribers.insert(sub_id);
            })
            .unwrap();

            sig.set(1).unwrap();
            assert_eq!(count.get(), 1);

            // Unregister the subscriber
            crate::subscriber::unregister_subscriber(sub_id).unwrap();

            sig.set(2).unwrap();
            // Callback slot is None, so notification is a no-op
            assert_eq!(count.get(), 1);
        });
    }

    // -- Disposal --

    #[test]
    fn disposed_signal_returns_error() {
        with_test_runtime(|| {
            let sig = create_signal(42).unwrap();
            dispose_signal(sig).unwrap();
            assert_eq!(sig.get().unwrap_err(), ReactiveError::SignalDisposed);
            assert_eq!(sig.set(0).unwrap_err(), ReactiveError::SignalDisposed);
        });
    }

    #[test]
    fn disposed_slot_is_recycled() {
        with_test_runtime(|| {
            let sig1 = create_signal(1).unwrap();
            let old_index = sig1.id.index;
            dispose_signal(sig1).unwrap();

            let sig2 = create_signal(2).unwrap();
            // Should reuse the same slot
            assert_eq!(sig2.id.index, old_index);
            // But with a higher generation
            assert!(sig2.id.generation > sig1.id.generation);
            assert_eq!(sig2.get().unwrap(), 2);

            // Old handle still fails
            assert_eq!(sig1.get().unwrap_err(), ReactiveError::SignalDisposed);
        });
    }

    #[test]
    fn disposal_cleans_up_subscribers() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let count = Rc::new(Cell::new(0));
            let count_clone = Rc::clone(&count);

            let sub_id = crate::subscriber::register_subscriber(move || {
                count_clone.set(count_clone.get() + 1);
            })
            .unwrap();

            crate::runtime::with_runtime_mut(|rt| {
                rt.signals[sig.read().id.index].subscribers.insert(sub_id);
            })
            .unwrap();

            dispose_signal(sig).unwrap();

            // Subscriber should have been unregistered
            crate::runtime::with_runtime(|rt| {
                assert!(rt.subscribers[sub_id.index].is_none());
            })
            .unwrap();
        });
    }

    // -- HashSet subscriber properties --

    #[test]
    fn idempotent_subscriber_tracking() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();
            let sub_id = crate::subscriber::register_subscriber(|| {}).unwrap();

            crate::subscriber::push_tracking(sub_id).unwrap();
            let _v1 = sig.get().unwrap();
            let _v2 = sig.get().unwrap(); // read again from same context
            crate::subscriber::pop_tracking().unwrap();

            let count = crate::runtime::with_runtime(|rt| {
                rt.signals[sig.read().id.index].subscribers.len()
            })
            .unwrap();
            assert_eq!(count, 1);
        });
    }

    #[test]
    fn signal_with_many_subscribers_tracks_correctly() {
        with_test_runtime(|| {
            let sig = create_signal(0).unwrap();

            for _ in 0..100 {
                let sub_id = crate::subscriber::register_subscriber(|| {}).unwrap();
                crate::subscriber::push_tracking(sub_id).unwrap();
                let _v = sig.get().unwrap();
                crate::subscriber::pop_tracking().unwrap();
            }

            let count = crate::runtime::with_runtime(|rt| {
                rt.signals[sig.read().id.index].subscribers.len()
            })
            .unwrap();
            assert_eq!(count, 100);
        });
    }

    // -- Complex types --

    #[test]
    fn signal_with_complex_type() {
        with_test_runtime(|| {
            #[derive(Clone, Debug, PartialEq)]
            struct User {
                name: String,
                age: u32,
            }

            let sig = create_signal(User {
                name: "Alice".into(),
                age: 30,
            })
            .unwrap();

            sig.update(|u| u.age += 1).unwrap();
            let user = sig.get().unwrap();
            assert_eq!(user.age, 31);
            assert_eq!(user.name, "Alice");
        });
    }
}
