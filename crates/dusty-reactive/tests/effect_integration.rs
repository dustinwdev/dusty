//! Integration tests for effects — reactive side effects.

use dusty_reactive::{
    create_effect, create_memo, create_signal, dispose_effect, dispose_runtime, initialize_runtime,
    on_cleanup, ReactiveError,
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
        let count = create_signal(0).unwrap();
        let observed = Rc::new(Cell::new(0));
        let ob = Rc::clone(&observed);

        let effect = create_effect(move || {
            ob.set(count.get().unwrap());
        })
        .unwrap();

        assert_eq!(observed.get(), 0);

        count.set(5).unwrap();
        assert_eq!(observed.get(), 5);

        count.set(42).unwrap();
        assert_eq!(observed.get(), 42);

        dispose_effect(&effect).unwrap();

        count.set(100).unwrap();
        assert_eq!(observed.get(), 42); // no re-run
    });
}

#[test]
fn effect_memo_dependency() {
    with_runtime(|| {
        let source = create_signal(2).unwrap();
        let doubled = create_memo(move || source.get().unwrap() * 2).unwrap();

        let observed = Rc::new(Cell::new(0));
        let ob = Rc::clone(&observed);

        let _effect = create_effect(move || {
            ob.set(doubled.get().unwrap());
        })
        .unwrap();

        assert_eq!(observed.get(), 4);

        source.set(5).unwrap();
        assert_eq!(observed.get(), 10);
    });
}

#[test]
fn multiple_effects_on_same_signal() {
    with_runtime(|| {
        let sig = create_signal(0).unwrap();

        let log_a = Rc::new(RefCell::new(Vec::<i32>::new()));
        let log_b = Rc::new(RefCell::new(Vec::<i32>::new()));
        let la = Rc::clone(&log_a);
        let lb = Rc::clone(&log_b);

        let _ea = create_effect(move || {
            la.borrow_mut().push(sig.get().unwrap());
        })
        .unwrap();

        let _eb = create_effect(move || {
            lb.borrow_mut().push(sig.get().unwrap());
        })
        .unwrap();

        assert_eq!(*log_a.borrow(), vec![0]);
        assert_eq!(*log_b.borrow(), vec![0]);

        sig.set(1).unwrap();
        assert_eq!(*log_a.borrow(), vec![0, 1]);
        assert_eq!(*log_b.borrow(), vec![0, 1]);
    });
}

#[test]
fn effect_with_cleanup_lifecycle() {
    with_runtime(|| {
        let count = create_signal(0).unwrap();
        let log = Rc::new(RefCell::new(Vec::<String>::new()));
        let l = Rc::clone(&log);

        let effect = create_effect(move || {
            let val = count.get().unwrap();
            let l2 = Rc::clone(&l);
            on_cleanup(move || {
                l2.borrow_mut().push(format!("cleanup-{val}"));
            });
            l.borrow_mut().push(format!("run-{val}"));
        })
        .unwrap();

        count.set(1).unwrap();
        count.set(2).unwrap();

        dispose_effect(&effect).unwrap();

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
        let effect = create_effect(|| {}).unwrap();
        dispose_effect(&effect).unwrap();
        assert_eq!(
            dispose_effect(&effect).unwrap_err(),
            ReactiveError::EffectDisposed
        );
    });
}

#[test]
fn effect_no_runtime_errors() {
    dispose_runtime();
    assert_eq!(create_effect(|| {}).unwrap_err(), ReactiveError::NoRuntime);
}

#[test]
fn dispose_memo_preserves_effect_on_other_signal() {
    with_runtime(|| {
        let sig = create_signal(1).unwrap();
        let memo = create_memo(move || sig.get().unwrap() * 2).unwrap();
        let memo_for_dispose = memo.clone();

        let other = create_signal(100).unwrap();
        let observed = Rc::new(Cell::new(0));
        let ob = Rc::clone(&observed);

        // Effect depends on both memo and other signal
        let _effect = create_effect(move || {
            let m = memo.get().unwrap_or(0);
            let o = other.get().unwrap();
            ob.set(m + o);
        })
        .unwrap();

        assert_eq!(observed.get(), 102); // 2 + 100

        // Dispose memo — effect should still work via 'other'
        dusty_reactive::dispose_memo(&memo_for_dispose).unwrap();

        // Changing 'other' should still trigger the effect
        other.set(200).unwrap();
        assert_eq!(observed.get(), 200); // 0 (memo disposed) + 200
    });
}
