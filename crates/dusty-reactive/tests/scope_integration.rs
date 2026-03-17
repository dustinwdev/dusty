//! Integration tests for scopes — arena-based ownership and cleanup.

use dusty_reactive::{
    create_child_scope, create_effect, create_memo, create_scope, create_signal, dispose_runtime,
    dispose_scope, initialize_runtime, on_cleanup, try_dispose_scope, ReactiveError,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn with_runtime(f: impl FnOnce()) {
    initialize_runtime();
    f();
    dispose_runtime();
}

#[test]
fn scope_owns_full_reactive_graph() {
    with_runtime(|| {
        let sig_handle = Rc::new(Cell::new(None));
        let memo_handle: Rc<RefCell<Option<dusty_reactive::Memo<i32>>>> =
            Rc::new(RefCell::new(None));
        let effect_ran = Rc::new(Cell::new(0));

        let sh = Rc::clone(&sig_handle);
        let mh = Rc::clone(&memo_handle);
        let er = Rc::clone(&effect_ran);

        let scope = create_scope(|_s| {
            let sig = create_signal(5);
            sh.set(Some(sig));

            let m = create_memo(move || sig.get() * 2);
            *mh.borrow_mut() = Some(m);

            let er2 = Rc::clone(&er);
            let _eff = create_effect(move || {
                let _val = sig.get();
                er2.set(er2.get() + 1);
            });
        });

        let sig = sig_handle.get().unwrap();
        let memo = memo_handle.borrow().as_ref().unwrap().clone();

        // Everything works before disposal
        assert_eq!(sig.get(), 5);
        assert_eq!(memo.get(), 10);
        assert!(effect_ran.get() >= 1);

        let runs_before = effect_ran.get();

        // Dispose scope — everything should be cleaned up
        dispose_scope(scope);

        assert_eq!(sig.try_get().unwrap_err(), ReactiveError::SignalDisposed);
        assert_eq!(memo.try_get().unwrap_err(), ReactiveError::MemoDisposed);

        // Effect should not re-run (its signal is disposed anyway)
        assert_eq!(effect_ran.get(), runs_before);
    });
}

#[test]
fn nested_scope_isolation() {
    with_runtime(|| {
        let parent_marker = Rc::new(Cell::new(false));
        let child_marker = Rc::new(Cell::new(false));

        let pm = Rc::clone(&parent_marker);
        let cm = Rc::clone(&child_marker);

        let child_handle = Rc::new(Cell::new(None));
        let ch = Rc::clone(&child_handle);

        let parent = create_scope(|p| {
            let _parent_sig = create_signal("parent");

            let child = create_child_scope(p, |_c| {
                let _child_sig = create_signal("child");
                let cm2 = Rc::clone(&cm);
                let _eff = create_effect(move || {
                    cm2.set(true);
                });
            });
            ch.set(Some(child));

            let pm2 = Rc::clone(&pm);
            let _eff = create_effect(move || {
                pm2.set(true);
            });
        });

        assert!(parent_marker.get());
        assert!(child_marker.get());

        // Dispose child — parent should still work
        let child = child_handle.get().unwrap();
        dispose_scope(child);

        // Parent scope and its effect should still be valid
        dispose_scope(parent);
    });
}

#[test]
fn scope_with_effect_cleanup() {
    with_runtime(|| {
        let count = create_signal(0);
        let log = Rc::new(RefCell::new(Vec::<String>::new()));
        let l = Rc::clone(&log);

        let scope = create_scope(|_s| {
            let _eff = create_effect(move || {
                let val = count.get();
                let l2 = Rc::clone(&l);
                on_cleanup(move || {
                    l2.borrow_mut().push(format!("cleanup-{val}"));
                });
                l.borrow_mut().push(format!("run-{val}"));
            });
        });

        count.set(1);
        assert_eq!(*log.borrow(), vec!["run-0", "cleanup-0", "run-1"]);

        // Disposing scope should run the effect's final cleanup
        dispose_scope(scope);
        assert_eq!(
            *log.borrow(),
            vec!["run-0", "cleanup-0", "run-1", "cleanup-1"]
        );
    });
}

#[test]
fn scope_double_dispose_errors() {
    with_runtime(|| {
        let scope = create_scope(|_s| {});
        dispose_scope(scope);
        assert_eq!(
            try_dispose_scope(scope).unwrap_err(),
            ReactiveError::ScopeDisposed
        );
    });
}

#[test]
fn parent_dispose_cascades_to_children() {
    with_runtime(|| {
        let order = Rc::new(RefCell::new(Vec::<&str>::new()));
        let o = Rc::clone(&order);

        let parent = create_scope(|p| {
            let o2 = Rc::clone(&o);
            let _eff = create_effect(move || {
                let o3 = Rc::clone(&o2);
                on_cleanup(move || o3.borrow_mut().push("parent-cleanup"));
            });

            let _child = create_child_scope(p, |_c| {
                let o4 = Rc::clone(&o);
                let _eff = create_effect(move || {
                    let o5 = Rc::clone(&o4);
                    on_cleanup(move || o5.borrow_mut().push("child-cleanup"));
                });
            });
        });

        dispose_scope(parent);

        let log = order.borrow();
        // Child should be disposed before parent (depth-first)
        assert!(log.contains(&"child-cleanup"));
        assert!(log.contains(&"parent-cleanup"));
        let child_pos = log.iter().position(|&s| s == "child-cleanup").unwrap();
        let parent_pos = log.iter().position(|&s| s == "parent-cleanup").unwrap();
        assert!(child_pos < parent_pos);
    });
}
