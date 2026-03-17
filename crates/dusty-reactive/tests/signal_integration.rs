//! Integration tests for the reactive signal system.

use dusty_reactive::{
    create_signal, create_signal_split, dispose_runtime, dispose_signal, initialize_runtime,
    try_create_signal, ReactiveError,
};
use static_assertions::assert_not_impl_any;

// Compile-time assertions: all handle types must be !Send + !Sync
assert_not_impl_any!(dusty_reactive::Signal<i32>: Send, Sync);
assert_not_impl_any!(dusty_reactive::ReadSignal<i32>: Send, Sync);
assert_not_impl_any!(dusty_reactive::WriteSignal<i32>: Send, Sync);
assert_not_impl_any!(dusty_reactive::Memo<i32>: Send, Sync);
assert_not_impl_any!(dusty_reactive::Effect: Send, Sync);
assert_not_impl_any!(dusty_reactive::Resource<i32>: Send, Sync);
assert_not_impl_any!(dusty_reactive::Scope: Send, Sync);

fn with_runtime(f: impl FnOnce()) {
    initialize_runtime();
    f();
    dispose_runtime();
}

#[test]
fn full_lifecycle() {
    with_runtime(|| {
        let sig = create_signal(0);
        assert_eq!(sig.get(), 0);

        sig.set(10);
        assert_eq!(sig.get(), 10);

        sig.update(|n| *n += 5);
        assert_eq!(sig.get(), 15);

        let (read, write) = sig.split();
        write.set(100);
        assert_eq!(read.get(), 100);

        dispose_signal(sig);
        assert_eq!(sig.try_get().unwrap_err(), ReactiveError::SignalDisposed);
    });
}

#[test]
fn multiple_independent_signals() {
    with_runtime(|| {
        let a = create_signal(1);
        let b = create_signal("hello");
        let c = create_signal(vec![1, 2, 3]);

        a.set(2);
        assert_eq!(a.get(), 2);
        assert_eq!(b.get(), "hello");
        assert_eq!(c.with(|v| v.len()), 3);

        c.update(|v| v.push(4));
        assert_eq!(c.with(|v| v.len()), 4);
    });
}

#[test]
fn complex_types() {
    with_runtime(|| {
        #[derive(Clone, Debug, PartialEq)]
        struct AppState {
            count: u32,
            label: String,
            items: Vec<i64>,
        }

        let state = create_signal(AppState {
            count: 0,
            label: "initial".into(),
            items: vec![],
        });

        state.update(|s| {
            s.count += 1;
            s.label = "updated".into();
            s.items.push(42);
        });

        let snapshot = state.get();
        assert_eq!(snapshot.count, 1);
        assert_eq!(snapshot.label, "updated");
        assert_eq!(snapshot.items, vec![42]);
    });
}

#[test]
fn split_signals_share_state() {
    with_runtime(|| {
        let (read, write) = create_signal_split(0);

        write.set(42);
        assert_eq!(read.get(), 42);

        write.update(|n| *n *= 2);
        assert_eq!(read.get(), 84);

        // with() provides zero-clone access
        let is_even = read.with(|n| n % 2 == 0);
        assert!(is_even);
    });
}

#[test]
fn with_untracked_provides_ref_access() {
    with_runtime(|| {
        let sig = create_signal(String::from("hello world"));
        let word_count = sig.with_untracked(|s| s.split_whitespace().count());
        assert_eq!(word_count, 2);
    });
}

#[test]
fn slot_recycling_across_signals() {
    with_runtime(|| {
        let a = create_signal(1);
        let b = create_signal(2);

        dispose_signal(a);

        // New signal should reuse a's slot
        let c = create_signal(3);

        // b should still work
        assert_eq!(b.get(), 2);
        // c should work with its own value
        assert_eq!(c.get(), 3);
        // a should be dead
        assert_eq!(a.try_get().unwrap_err(), ReactiveError::SignalDisposed);
    });
}

#[test]
fn no_runtime_returns_error() {
    dispose_runtime();
    let result = try_create_signal(42);
    assert_eq!(result.unwrap_err(), ReactiveError::NoRuntime);
}

#[test]
fn runtime_reset_clears_signals() {
    initialize_runtime();
    let sig = create_signal(42);
    assert_eq!(sig.get(), 42);

    // Re-initialize clears everything
    initialize_runtime();
    // Old signal handle now refers to a dead/wrong generation slot
    // (the slot doesn't exist at all in the fresh runtime)
    assert_eq!(sig.try_get().unwrap_err(), ReactiveError::SignalDisposed);
    dispose_runtime();
}
