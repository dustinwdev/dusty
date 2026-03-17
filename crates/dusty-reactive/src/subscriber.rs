//! Subscriber tracking for reactive dependency management.

use std::rc::Rc;

use crate::error::Result;
use crate::runtime::{with_runtime, with_runtime_mut, SignalId};

/// Identifies a subscriber in the runtime's subscriber storage.
///
/// Includes a generational index to prevent stale IDs from invoking
/// callbacks that belong to a different subscriber reusing the same slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriberId {
    pub index: usize,
    pub generation: u64,
}

/// Register a subscriber callback in the runtime. Returns its ID.
///
/// # Errors
///
/// Returns [`ReactiveError::NoRuntime`] if no runtime is initialized.
pub fn register_subscriber(callback: impl Fn() + 'static) -> Result<SubscriberId> {
    with_runtime_mut(|rt| {
        if let Some(index) = rt.subscriber_free_list.pop() {
            let generation = rt.subscriber_generations[index] + 1;
            rt.subscriber_generations[index] = generation;
            rt.subscribers[index] = Some(Rc::new(callback));
            SubscriberId { index, generation }
        } else {
            let index = rt.subscribers.len();
            rt.subscribers.push(Some(Rc::new(callback)));
            rt.subscriber_generations.push(0);
            SubscriberId {
                index,
                generation: 0,
            }
        }
    })
}

/// Remove a subscriber from the runtime.
///
/// Checks the generational index to prevent double-free of subscriber slots.
/// If the generation doesn't match (slot already reused), this is a no-op.
pub fn unregister_subscriber(id: SubscriberId) -> Result<()> {
    with_runtime_mut(|rt| {
        if id.index < rt.subscribers.len()
            && rt.subscriber_generations[id.index] == id.generation
            && rt.subscribers[id.index].is_some()
        {
            rt.subscribers[id.index] = None;
            rt.subscriber_free_list.push(id.index);
        }
    })
}

/// Invoke a subscriber callback if it is still valid (correct generation).
///
/// Clones the `Rc` callback and releases the runtime borrow before invoking,
/// so the callback is free to mutate the runtime (e.g., write to signals).
pub fn invoke_subscriber(id: SubscriberId) -> Result<()> {
    let cb = with_runtime(|rt| {
        if id.index < rt.subscriber_generations.len()
            && rt.subscriber_generations[id.index] == id.generation
        {
            rt.subscribers[id.index].as_ref().map(Rc::clone)
        } else {
            None
        }
    })?;

    if let Some(callback) = cb {
        callback();
    }

    Ok(())
}

/// Push a subscriber onto the tracking stack. While on the stack,
/// any signal reads will register this subscriber as a dependent
/// and record the read signals in the dependency stack.
///
/// # Errors
///
/// Returns [`ReactiveError::NoRuntime`] if no runtime is initialized.
pub fn push_tracking(id: SubscriberId) -> Result<()> {
    with_runtime_mut(|rt| {
        rt.tracking_stack.push(id);
        rt.dependency_stack.push(std::collections::HashSet::new());
    })
}

/// Pop the current subscriber from the tracking stack.
/// Returns the list of signals that were read during this tracking scope.
///
/// # Errors
///
/// Returns [`ReactiveError::NoRuntime`] if no runtime is initialized.
pub fn pop_tracking() -> Result<Vec<SignalId>> {
    with_runtime_mut(|rt| {
        let popped_tracker = rt.tracking_stack.pop();
        let popped_deps = rt.dependency_stack.pop();
        debug_assert!(
            popped_tracker.is_some() && popped_deps.is_some(),
            "pop_tracking called with empty tracking stack"
        );
        popped_deps
            .map(|set| set.into_iter().collect())
            .unwrap_or_default()
    })
}

/// Get the subscriber currently being tracked (top of stack), if any.
///
/// Uses an immutable borrow — safe to call during notification callbacks.
pub fn current_tracking() -> Result<Option<SubscriberId>> {
    with_runtime(|rt| rt.tracking_stack.last().copied())
}

/// Run a closure without tracking any signal reads.
///
/// Any signal reads inside `f` will not register the current subscriber as a
/// dependent. The tracking stack is saved, cleared, then restored after `f`
/// returns. This works correctly even with nested tracking contexts.
///
/// If no runtime is initialized, the closure is still executed (tracking is
/// irrelevant without a runtime).
///
/// # Examples
///
/// ```
/// # dusty_reactive::initialize_runtime();
/// let sig = dusty_reactive::create_signal(42).unwrap();
/// let val = dusty_reactive::untrack(|| sig.get().unwrap());
/// assert_eq!(val, 42);
/// # dusty_reactive::dispose_runtime();
/// ```
pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    let saved = with_runtime_mut(|rt| {
        let stack = std::mem::take(&mut rt.tracking_stack);
        let deps = std::mem::take(&mut rt.dependency_stack);
        (stack, deps)
    });

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

    if let Ok((stack, deps)) = saved {
        let _ = with_runtime_mut(|rt| {
            rt.tracking_stack = stack;
            rt.dependency_stack = deps;
        });
    }

    match result {
        Ok(val) => val,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::dispose_runtime;
    use crate::tracking::with_test_runtime;

    #[test]
    fn register_subscriber_returns_id() {
        with_test_runtime(|| {
            let id = register_subscriber(|| {}).unwrap();
            assert_eq!(id.index, 0);
            assert_eq!(id.generation, 0);

            let id2 = register_subscriber(|| {}).unwrap();
            assert_eq!(id2.index, 1);
            assert_eq!(id2.generation, 0);
        });
    }

    #[test]
    fn unregister_frees_slot_for_reuse() {
        with_test_runtime(|| {
            let id0 = register_subscriber(|| {}).unwrap();
            let _id1 = register_subscriber(|| {}).unwrap();

            unregister_subscriber(id0).unwrap();

            // Next registration should reuse slot 0 with bumped generation
            let id2 = register_subscriber(|| {}).unwrap();
            assert_eq!(id2.index, 0);
            assert_eq!(id2.generation, 1);
        });
    }

    #[test]
    fn double_unregister_does_not_corrupt_free_list() {
        with_test_runtime(|| {
            let id = register_subscriber(|| {}).unwrap();
            unregister_subscriber(id).unwrap();
            // Second unregister must be a no-op — the is_some() guard prevents
            // pushing the same index to the free list twice.
            unregister_subscriber(id).unwrap();

            // Register two new subscribers — they MUST get distinct slots.
            // Without the is_some() guard, the free list would contain index 0
            // twice, causing both registrations to get the same slot.
            let counter1 = std::rc::Rc::new(std::cell::Cell::new(0));
            let counter2 = std::rc::Rc::new(std::cell::Cell::new(0));
            let c1 = std::rc::Rc::clone(&counter1);
            let c2 = std::rc::Rc::clone(&counter2);

            let id1 = register_subscriber(move || {
                c1.set(c1.get() + 1);
            })
            .unwrap();
            let id2 = register_subscriber(move || {
                c2.set(c2.get() + 1);
            })
            .unwrap();

            // Slots must be distinct
            assert_ne!(
                id1.index, id2.index,
                "double unregister corrupted free list: both registrations got the same slot"
            );

            // Both callbacks must invoke independently
            invoke_subscriber(id1).unwrap();
            assert_eq!(counter1.get(), 1);
            assert_eq!(counter2.get(), 0);

            invoke_subscriber(id2).unwrap();
            assert_eq!(counter1.get(), 1);
            assert_eq!(counter2.get(), 1);
        });
    }

    #[test]
    fn invoke_subscriber_checks_generation() {
        with_test_runtime(|| {
            let counter = std::rc::Rc::new(std::cell::Cell::new(0));
            let c = std::rc::Rc::clone(&counter);
            let old_id = register_subscriber(move || {
                c.set(c.get() + 1);
            })
            .unwrap();

            // Invoke should work with correct generation
            invoke_subscriber(old_id).unwrap();
            assert_eq!(counter.get(), 1);

            // Unregister and register a new subscriber at the same slot
            unregister_subscriber(old_id).unwrap();
            let new_counter = std::rc::Rc::new(std::cell::Cell::new(0));
            let nc = std::rc::Rc::clone(&new_counter);
            let new_id = register_subscriber(move || {
                nc.set(nc.get() + 1);
            })
            .unwrap();
            assert_eq!(new_id.index, old_id.index);
            assert_ne!(new_id.generation, old_id.generation);

            // Invoking with old_id should NOT call the new callback
            invoke_subscriber(old_id).unwrap();
            assert_eq!(new_counter.get(), 0);

            // Invoking with new_id should work
            invoke_subscriber(new_id).unwrap();
            assert_eq!(new_counter.get(), 1);
        });
    }

    #[test]
    fn invoke_subscriber_allows_runtime_mutation_in_callback() {
        with_test_runtime(|| {
            let id = register_subscriber(|| {
                // This requires a mutable borrow — would panic if invoke held an immutable borrow
                let _ = crate::runtime::with_runtime_mut(|rt| {
                    rt.batch_depth += 1;
                    rt.batch_depth -= 1;
                });
            })
            .unwrap();

            // Should not panic
            invoke_subscriber(id).unwrap();
        });
    }

    #[test]
    fn tracking_stack_push_pop() {
        with_test_runtime(|| {
            assert_eq!(current_tracking().unwrap(), None);

            let id = register_subscriber(|| {}).unwrap();
            push_tracking(id).unwrap();
            assert_eq!(current_tracking().unwrap(), Some(id));

            let deps = pop_tracking().unwrap();
            assert!(deps.is_empty());
            assert_eq!(current_tracking().unwrap(), None);
        });
    }

    #[test]
    fn tracking_stack_nested() {
        with_test_runtime(|| {
            let id1 = register_subscriber(|| {}).unwrap();
            let id2 = register_subscriber(|| {}).unwrap();

            push_tracking(id1).unwrap();
            assert_eq!(current_tracking().unwrap(), Some(id1));

            push_tracking(id2).unwrap();
            assert_eq!(current_tracking().unwrap(), Some(id2));

            let _deps2 = pop_tracking().unwrap();
            assert_eq!(current_tracking().unwrap(), Some(id1));

            let _deps1 = pop_tracking().unwrap();
            assert_eq!(current_tracking().unwrap(), None);
        });
    }

    // -- untrack tests --

    #[test]
    fn untrack_returns_closure_value() {
        with_test_runtime(|| {
            let val = untrack(|| 42);
            assert_eq!(val, 42);
        });
    }

    #[test]
    fn untrack_suppresses_tracking() {
        with_test_runtime(|| {
            let sig = crate::signal::create_signal(10).unwrap();
            let id = register_subscriber(|| {}).unwrap();

            push_tracking(id).unwrap();
            // Read inside untrack should NOT subscribe
            let val = untrack(|| sig.get().unwrap());
            assert_eq!(val, 10);
            let deps = pop_tracking().unwrap();

            // No dependencies recorded
            assert!(deps.is_empty());

            // Signal should have no subscribers
            let sub_count =
                crate::runtime::with_runtime(|rt| rt.signals[0].subscribers.len()).unwrap();
            assert_eq!(sub_count, 0);
        });
    }

    #[test]
    fn untrack_restores_tracking_after() {
        with_test_runtime(|| {
            let sig = crate::signal::create_signal(10).unwrap();
            let id = register_subscriber(|| {}).unwrap();

            push_tracking(id).unwrap();
            untrack(|| {});
            // Tracking should be restored — read registers subscriber
            let _val = sig.get().unwrap();
            let deps = pop_tracking().unwrap();

            assert_eq!(deps.len(), 1);
        });
    }

    #[test]
    fn untrack_works_without_runtime() {
        dispose_runtime();
        let val = untrack(|| 99);
        assert_eq!(val, 99);
    }

    #[test]
    fn untrack_nested() {
        with_test_runtime(|| {
            let sig = crate::signal::create_signal(5).unwrap();
            let id = register_subscriber(|| {}).unwrap();

            push_tracking(id).unwrap();
            let val = untrack(|| {
                let inner = untrack(|| sig.get().unwrap());
                inner + 1
            });
            assert_eq!(val, 6);
            let deps = pop_tracking().unwrap();
            assert!(deps.is_empty());
        });
    }

    #[test]
    fn current_tracking_works_during_immutable_borrow() {
        with_test_runtime(|| {
            let id = register_subscriber(|| {}).unwrap();
            push_tracking(id).unwrap();

            // current_tracking uses with_runtime (immutable) so it should
            // succeed even while another immutable borrow is active.
            with_runtime(|_rt| {
                let tracking = current_tracking();
                assert_eq!(tracking.unwrap(), Some(id));
            })
            .unwrap();

            pop_tracking().unwrap();
        });
    }

    #[test]
    fn untrack_panic_restores_tracking() {
        with_test_runtime(|| {
            let sig = crate::signal::create_signal(0).unwrap();
            let id = register_subscriber(|| {}).unwrap();

            push_tracking(id).unwrap();

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                untrack(|| {
                    panic!("untrack panic");
                });
            }));
            assert!(result.is_err());

            // Tracking should be restored
            assert_eq!(current_tracking().unwrap(), Some(id));

            // Signal reads should still register
            let _val = sig.get().unwrap();
            let deps = pop_tracking().unwrap();
            assert_eq!(deps.len(), 1);
        });
    }
}
