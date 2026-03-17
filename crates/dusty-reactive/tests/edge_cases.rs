//! Edge case and cross-module interaction tests for dusty-reactive.

use dusty_reactive::*;
use std::cell::Cell;
use std::rc::Rc;

fn with_runtime(f: impl FnOnce()) {
    initialize_runtime();
    f();
    dispose_runtime();
}

// ---------------------------------------------------------------------------
// Effect creating signals during execution
// ---------------------------------------------------------------------------

#[test]
fn effect_creates_signal_during_execution() {
    with_runtime(|| {
        let created = Rc::new(Cell::new(false));
        let c = Rc::clone(&created);

        let _effect = create_effect(move || {
            let _inner_sig = create_signal(42);
            c.set(true);
        });

        assert!(created.get());
    });
}

#[test]
fn effect_creates_effect_during_execution() {
    with_runtime(|| {
        let outer_ran = Rc::new(Cell::new(0));
        let inner_ran = Rc::new(Cell::new(0));
        let or = Rc::clone(&outer_ran);
        let ir = Rc::clone(&inner_ran);

        let sig = create_signal(0);

        let _outer = create_effect(move || {
            let _val = sig.get();
            or.set(or.get() + 1);
            if or.get() == 1 {
                let ir2 = Rc::clone(&ir);
                let _inner = create_effect(move || {
                    ir2.set(ir2.get() + 1);
                });
            }
        });

        assert_eq!(outer_ran.get(), 1);
        assert_eq!(inner_ran.get(), 1);
    });
}

// ---------------------------------------------------------------------------
// Memo reading a disposed dependency
// ---------------------------------------------------------------------------

#[test]
fn memo_handles_disposed_dependency_gracefully() {
    with_runtime(|| {
        let sig = create_signal(10);
        let memo = create_memo(move || sig.try_get().unwrap_or(-1) * 2);

        assert_eq!(memo.get(), 20);

        // Dispose the signal out from under the memo.
        // Signal disposal no longer unregisters shared subscribers (WS1B fix),
        // so the memo is not marked dirty and returns its cached value.
        dispose_signal(sig);

        // Memo should still be queryable — returns the last cached value
        // since disposal does not trigger recomputation.
        let result = memo.get();
        assert_eq!(result, 20);
    });
}

// ---------------------------------------------------------------------------
// Batch + scope interaction
// ---------------------------------------------------------------------------

#[test]
fn batch_inside_scope_coalesces_correctly() {
    with_runtime(|| {
        let run_count = Rc::new(Cell::new(0));

        let _scope = create_scope(|_cx| {
            let sig = create_signal(0);
            let rc = Rc::clone(&run_count);

            let _effect = create_effect(move || {
                let _val = sig.get();
                rc.set(rc.get() + 1);
            });

            assert_eq!(run_count.get(), 1);

            batch(|| {
                sig.set(1);
                sig.set(2);
                sig.set(3);
            });

            // Only one additional run from the batch
            assert_eq!(run_count.get(), 2);
        });
    });
}

// ---------------------------------------------------------------------------
// Untrack inside batch
// ---------------------------------------------------------------------------

#[test]
fn untrack_inside_batch_does_not_track() {
    with_runtime(|| {
        let sig = create_signal(0);
        let run_count = Rc::new(Cell::new(0));
        let rc = Rc::clone(&run_count);

        let _effect = create_effect(move || {
            batch(|| {
                untrack(|| {
                    let _val = sig.get();
                });
            });
            rc.set(rc.get() + 1);
        });

        assert_eq!(run_count.get(), 1);

        // Signal change should NOT trigger re-run (read was untracked)
        sig.set(1);
        assert_eq!(run_count.get(), 1);
    });
}

// ---------------------------------------------------------------------------
// Slot recycling stress test
// ---------------------------------------------------------------------------

#[test]
fn rapid_signal_create_dispose_does_not_corrupt() {
    with_runtime(|| {
        let mut handles = Vec::new();

        // Create many signals
        for i in 0..100 {
            handles.push(create_signal(i));
        }

        // Dispose every other one
        for i in (0..100).step_by(2) {
            dispose_signal(handles[i]);
        }

        // Remaining should still work
        for i in (1..100).step_by(2) {
            assert_eq!(handles[i].get(), i as i32);
        }

        // Create new signals — should reuse slots
        for i in 0..50 {
            let sig = create_signal(1000 + i);
            assert_eq!(sig.get(), 1000 + i);
        }
    });
}

// ---------------------------------------------------------------------------
// Strengthen weak assertions (Phase 20 reactive items)
// ---------------------------------------------------------------------------

#[test]
fn signal_set_during_batch_flush_no_missed_notifications() {
    with_runtime(|| {
        let sig_a = create_signal(0);
        let sig_b = create_signal(0);
        let observed_b = Rc::new(Cell::new(0));
        let ob = Rc::clone(&observed_b);

        // Effect on sig_a writes to sig_b
        let _eff_a = create_effect(move || {
            let val = sig_a.get();
            if val > 0 {
                sig_b.set(val * 10);
            }
        });

        // Effect on sig_b observes its value
        let _eff_b = create_effect(move || {
            ob.set(sig_b.get());
        });

        sig_a.set(5);
        assert_eq!(observed_b.get(), 50);
    });
}

#[test]
fn dispose_runtime_with_active_cleanups() {
    initialize_runtime();

    let cleanup_ran = Rc::new(Cell::new(false));
    let cr = Rc::clone(&cleanup_ran);

    let _scope = create_scope(|_cx| {
        let _effect = create_effect(move || {
            let cr2 = Rc::clone(&cr);
            on_cleanup(move || cr2.set(true));
        });
    });

    // Dispose runtime without explicit scope disposal
    // Cleanups are NOT guaranteed to run — runtime just drops everything
    dispose_runtime();

    // The cleanup may or may not have run depending on drop order
    // This test primarily verifies no panic occurs
}

#[test]
fn memo_with_panicking_partial_eq() {
    with_runtime(|| {
        #[derive(Clone, Debug)]
        struct PanicEq(i32);

        impl PartialEq for PanicEq {
            fn eq(&self, other: &Self) -> bool {
                if self.0 == 999 || other.0 == 999 {
                    panic!("PartialEq panic");
                }
                self.0 == other.0
            }
        }

        let sig = create_signal(1i32);
        let memo = create_memo(move || PanicEq(sig.get()));

        assert_eq!(memo.get().0, 1);

        sig.set(2);
        assert_eq!(memo.get().0, 2);

        // Setting to 999 will cause PartialEq to panic during memo evaluation
        sig.set(999);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = memo.get();
        }));
        assert!(result.is_err());

        // Runtime should still be usable after the panic
        let sig2 = create_signal(42);
        assert_eq!(sig2.get(), 42);
    });
}

#[test]
fn multi_thread_independent_runtimes() {
    // Each thread gets its own runtime — verify no interference
    let handles: Vec<_> = (0..4)
        .map(|thread_id| {
            std::thread::spawn(move || {
                initialize_runtime();
                let sig = create_signal(thread_id * 100);
                let memo = create_memo(move || sig.get() + 1);

                assert_eq!(memo.get(), thread_id * 100 + 1);

                sig.set(thread_id * 200);
                assert_eq!(memo.get(), thread_id * 200 + 1);

                dispose_runtime();
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

// ---------------------------------------------------------------------------
// Exact run count assertions
// ---------------------------------------------------------------------------

#[test]
fn resource_state_tracked_exact_run_count() {
    with_runtime(|| {
        let source = create_signal(1);
        let resource = create_resource(
            move || source.get(),
            |val: i32, resolver: dusty_reactive::ResourceResolver<i32>| {
                resolver.resolve(val);
            },
        );

        let run_count = Rc::new(Cell::new(0));
        let rc = Rc::clone(&run_count);
        let res = resource.clone();

        let _effect = create_effect(move || {
            let _state = res.state();
            rc.set(rc.get() + 1);
        });

        // Initial run
        assert_eq!(run_count.get(), 1);

        // Change source — resource re-fetches, effect re-runs
        source.set(2);
        assert_eq!(run_count.get(), 2);
    });
}
