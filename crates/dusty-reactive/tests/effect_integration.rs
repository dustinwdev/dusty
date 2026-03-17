//! Integration tests for effects — reactive side effects.

use dusty_reactive::{
    create_effect, create_memo, create_signal, dispose_effect, dispose_runtime, initialize_runtime,
    on_cleanup, try_create_effect, try_dispose_effect, ReactiveError,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn with_runtime(f: impl FnOnce()) {
    initialize_runtime();
    f();
    dispose_runtime();
}

#[test]
fn effect_signal_full_lifecycle() {
    with_runtime(|| {
        let count = create_signal(0);
        let observed = Rc::new(Cell::new(0));
        let ob = Rc::clone(&observed);

        let effect = create_effect(move || {
            ob.set(count.get());
        });

        assert_eq!(observed.get(), 0);

        count.set(5);
        assert_eq!(observed.get(), 5);

        count.set(42);
        assert_eq!(observed.get(), 42);

        dispose_effect(&effect);

        count.set(100);
        assert_eq!(observed.get(), 42); // no re-run
    });
}

#[test]
fn effect_memo_dependency() {
    with_runtime(|| {
        let source = create_signal(2);
        let doubled = create_memo(move || source.get() * 2);

        let observed = Rc::new(Cell::new(0));
        let ob = Rc::clone(&observed);

        let _effect = create_effect(move || {
            ob.set(doubled.get());
        });

        assert_eq!(observed.get(), 4);

        source.set(5);
        assert_eq!(observed.get(), 10);
    });
}

#[test]
fn multiple_effects_on_same_signal() {
    with_runtime(|| {
        let sig = create_signal(0);

        let log_a = Rc::new(RefCell::new(Vec::<i32>::new()));
        let log_b = Rc::new(RefCell::new(Vec::<i32>::new()));
        let la = Rc::clone(&log_a);
        let lb = Rc::clone(&log_b);

        let _ea = create_effect(move || {
            la.borrow_mut().push(sig.get());
        });

        let _eb = create_effect(move || {
            lb.borrow_mut().push(sig.get());
        });

        assert_eq!(*log_a.borrow(), vec![0]);
        assert_eq!(*log_b.borrow(), vec![0]);

        sig.set(1);
        assert_eq!(*log_a.borrow(), vec![0, 1]);
        assert_eq!(*log_b.borrow(), vec![0, 1]);
    });
}

#[test]
fn effect_with_cleanup_lifecycle() {
    with_runtime(|| {
        let count = create_signal(0);
        let log = Rc::new(RefCell::new(Vec::<String>::new()));
        let l = Rc::clone(&log);

        let effect = create_effect(move || {
            let val = count.get();
            let l2 = Rc::clone(&l);
            on_cleanup(move || {
                l2.borrow_mut().push(format!("cleanup-{val}"));
            });
            l.borrow_mut().push(format!("run-{val}"));
        });

        count.set(1);
        count.set(2);

        dispose_effect(&effect);

        assert_eq!(
            *log.borrow(),
            vec![
                "run-0",
                "cleanup-0",
                "run-1",
                "cleanup-1",
                "run-2",
                "cleanup-2"
            ]
        );
    });
}

#[test]
fn effect_dispose_errors_on_double_dispose() {
    with_runtime(|| {
        let effect = create_effect(|| {});
        dispose_effect(&effect);
        assert_eq!(
            try_dispose_effect(&effect).unwrap_err(),
            ReactiveError::EffectDisposed
        );
    });
}

#[test]
fn effect_no_runtime_errors() {
    dispose_runtime();
    assert_eq!(
        try_create_effect(|| {}).unwrap_err(),
        ReactiveError::NoRuntime
    );
}

#[test]
fn dispose_memo_preserves_effect_on_other_signal() {
    with_runtime(|| {
        let sig = create_signal(1);
        let memo = create_memo(move || sig.get() * 2);
        let memo_for_dispose = memo.clone();

        let other = create_signal(100);
        let observed = Rc::new(Cell::new(0));
        let ob = Rc::clone(&observed);

        // Effect depends on both memo and other signal
        // Memo may be disposed while effect is still alive, so use try_get
        let _effect = create_effect(move || {
            let m = memo.try_get().unwrap_or(0);
            let o = other.get();
            ob.set(m + o);
        });

        assert_eq!(observed.get(), 102); // 2 + 100

        // Dispose memo — effect should still work via 'other'
        dusty_reactive::dispose_memo(&memo_for_dispose);

        // Changing 'other' should still trigger the effect
        other.set(200);
        assert_eq!(observed.get(), 200); // 0 (memo disposed) + 200
    });
}
