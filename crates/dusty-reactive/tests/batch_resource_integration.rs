//! Integration tests for Phase 5: untrack, batch, and resource.

use dusty_reactive::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

fn with_test_runtime(f: impl FnOnce()) {
    initialize_runtime();
    f();
    dispose_runtime();
}

// ---------------------------------------------------------------------------
// thread-local cleanup on dispose_runtime
// ---------------------------------------------------------------------------

/// After dispose_runtime, auxiliary thread-locals (fresheners, pending effects,
/// cleanup sink) must be cleared so a fresh runtime starts clean.
#[test]
fn dispose_runtime_clears_thread_locals() {
    // Phase 1: populate all auxiliary thread-locals
    initialize_runtime();
    let source = create_signal(1).unwrap();
    let _memo = create_memo(move || source.get().unwrap() * 2).unwrap();

    let effect_ran = Rc::new(Cell::new(0));
    let er = Rc::clone(&effect_ran);
    let _effect = create_effect(move || {
        let _val = source.get().unwrap();
        er.set(er.get() + 1);
    })
    .unwrap();
    assert_eq!(effect_ran.get(), 1);
    dispose_runtime();

    // Phase 2: re-initialize and verify no stale state interferes
    initialize_runtime();
    let source2 = create_signal(10).unwrap();
    let memo2 = create_memo(move || source2.get().unwrap() + 1).unwrap();

    assert_eq!(memo2.get().unwrap(), 11);

    let observed = Rc::new(Cell::new(0));
    let ob = Rc::clone(&observed);
    let memo2_for_effect = memo2.clone();
    let _effect2 = create_effect(move || {
        ob.set(memo2_for_effect.get().unwrap());
    })
    .unwrap();
    assert_eq!(observed.get(), 11);

    source2.set(20).unwrap();
    assert_eq!(observed.get(), 21);
    assert_eq!(memo2.get().unwrap(), 21);
    dispose_runtime();
}

// ---------------------------------------------------------------------------
// untrack integration
// ---------------------------------------------------------------------------

#[test]
fn effect_untrack_does_not_create_dependency() {
    with_test_runtime(|| {
        let a = create_signal(1).unwrap();
        let b = create_signal(100).unwrap();
        let observed = Rc::new(Cell::new(0));
        let ob = Rc::clone(&observed);

        let _effect = create_effect(move || {
            let va = a.get().unwrap();
            let vb = untrack(|| b.get().unwrap());
            ob.set(va + vb);
        })
        .unwrap();

        assert_eq!(observed.get(), 101);

        // Changing b should NOT re-run the effect (untracked)
        b.set(200).unwrap();
        assert_eq!(observed.get(), 101);

        // Changing a SHOULD re-run the effect (tracked)
        a.set(2).unwrap();
        // Effect reads b's current value (200) during re-run
        assert_eq!(observed.get(), 202);
    });
}

// ---------------------------------------------------------------------------
// batch integration
// ---------------------------------------------------------------------------

#[test]
fn batch_effect_sees_final_values() {
    with_test_runtime(|| {
        let a = create_signal(0).unwrap();
        let b = create_signal(0).unwrap();
        let observed_a = Rc::new(Cell::new(0));
        let observed_b = Rc::new(Cell::new(0));
        let oa = Rc::clone(&observed_a);
        let ob = Rc::clone(&observed_b);

        let _effect = create_effect(move || {
            oa.set(a.get().unwrap());
            ob.set(b.get().unwrap());
        })
        .unwrap();

        assert_eq!(observed_a.get(), 0);
        assert_eq!(observed_b.get(), 0);

        batch(|| {
            a.set(10).unwrap();
            b.set(20).unwrap();
        })
        .unwrap();

        // Effect sees both final values together
        assert_eq!(observed_a.get(), 10);
        assert_eq!(observed_b.get(), 20);
    });
}

#[test]
fn batch_memo_chain_evaluated_once() {
    with_test_runtime(|| {
        let source = create_signal(1).unwrap();
        let eval_count = Rc::new(Cell::new(0));
        let ec = Rc::clone(&eval_count);

        let doubled = create_memo(move || {
            ec.set(ec.get() + 1);
            source.get().unwrap() * 2
        })
        .unwrap();

        assert_eq!(doubled.get().unwrap(), 2);
        assert_eq!(eval_count.get(), 1);

        batch(|| {
            source.set(2).unwrap();
            source.set(3).unwrap();
            source.set(4).unwrap();
        })
        .unwrap();

        // Memo should evaluate just once for the final value
        assert_eq!(doubled.get().unwrap(), 8);
        assert_eq!(eval_count.get(), 2);
    });
}

/// Reading a memo inside a batch forces freshening. If the memo value changed,
/// `update_and_notify` must defer downstream notification until after the batch.
#[test]
fn batch_memo_read_inside_batch_defers_notification() {
    with_test_runtime(|| {
        let source = create_signal(1).unwrap();
        let doubled = create_memo(move || source.get().unwrap() * 2).unwrap();
        let doubled_for_batch = doubled.clone();

        let effect_count = Rc::new(Cell::new(0));
        let ec = Rc::clone(&effect_count);

        let _effect = create_effect(move || {
            let _val = doubled.get().unwrap();
            ec.set(ec.get() + 1);
        })
        .unwrap();

        assert_eq!(effect_count.get(), 1);

        batch(|| {
            source.set(10).unwrap();
            // Reading memo inside batch forces freshening
            let val = doubled_for_batch.get().unwrap();
            assert_eq!(val, 20);

            // Effect should NOT have re-run yet — still in the batch
            assert_eq!(effect_count.get(), 1);
        })
        .unwrap();

        // Effect should run exactly once after batch
        assert_eq!(effect_count.get(), 2);
    });
}

// ---------------------------------------------------------------------------
// resource integration
// ---------------------------------------------------------------------------

#[test]
fn resource_with_derived_source() {
    with_test_runtime(|| {
        let count = create_signal(3).unwrap();
        let doubled = create_memo(move || count.get().unwrap() * 2).unwrap();

        let resource = create_resource(
            move || doubled.get().unwrap(),
            |val, resolver| {
                resolver.resolve(val + 1);
            },
        )
        .unwrap();

        // Source is memo: doubled = 6, resource = 7
        assert_eq!(resource.get().unwrap(), Some(7));

        count.set(5).unwrap();
        // doubled = 10, resource = 11
        assert_eq!(resource.get().unwrap(), Some(11));
    });
}

#[test]
fn resource_in_scope_disposed_with_scope() {
    with_test_runtime(|| {
        let source = create_signal(1).unwrap();
        let fetch_count = Rc::new(Cell::new(0));
        let fc = Rc::clone(&fetch_count);

        let res_handle = Rc::new(RefCell::new(None));
        let rh = Rc::clone(&res_handle);

        let scope = create_scope(|_s| {
            let fc2 = Rc::clone(&fc);
            let resource = create_resource(
                move || source.get().unwrap(),
                move |val, resolver| {
                    fc2.set(fc2.get() + 1);
                    resolver.resolve(val);
                },
            )
            .unwrap();
            *rh.borrow_mut() = Some(resource);
        })
        .unwrap();

        assert_eq!(fetch_count.get(), 1);

        // Dispose scope — resource effect should be cleaned up
        dispose_scope(scope).unwrap();

        // Changing source should NOT trigger re-fetch
        source.set(2).unwrap();
        assert_eq!(fetch_count.get(), 1);
    });
}

#[test]
fn batch_with_resource() {
    with_test_runtime(|| {
        let source = create_signal(1).unwrap();
        let fetch_count = Rc::new(Cell::new(0));
        let fc = Rc::clone(&fetch_count);

        let resource = create_resource(
            move || source.get().unwrap(),
            move |val, resolver| {
                fc.set(fc.get() + 1);
                resolver.resolve(val * 10);
            },
        )
        .unwrap();

        assert_eq!(fetch_count.get(), 1);
        assert_eq!(resource.get().unwrap(), Some(10));

        // Batch multiple source changes — should only trigger one re-fetch
        batch(|| {
            source.set(2).unwrap();
            source.set(3).unwrap();
        })
        .unwrap();

        // Resource re-fetched once with the final value
        assert_eq!(fetch_count.get(), 2);
        assert_eq!(resource.get().unwrap(), Some(30));
    });
}

#[test]
fn resource_effect_chain() {
    with_test_runtime(|| {
        let source = create_signal(1).unwrap();

        let resource = create_resource(
            move || source.get().unwrap(),
            |val, resolver| {
                resolver.resolve(val * 100);
            },
        )
        .unwrap();

        // Memo that reads resource state
        let res_for_memo = resource.clone();
        let resource_val = create_memo(move || res_for_memo.get().unwrap().unwrap_or(0)).unwrap();

        // Effect that reads the memo
        let observed = Rc::new(Cell::new(0));
        let ob = Rc::clone(&observed);
        let _effect = create_effect(move || {
            ob.set(resource_val.get().unwrap());
        })
        .unwrap();

        assert_eq!(observed.get(), 100);

        source.set(2).unwrap();
        assert_eq!(observed.get(), 200);
    });
}
