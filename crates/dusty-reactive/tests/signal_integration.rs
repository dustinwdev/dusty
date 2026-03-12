//! Integration tests for the reactive signal system.

use dusty_reactive::{
    create_signal, create_signal_split, dispose_runtime, dispose_signal, initialize_runtime,
    ReactiveError,
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
        let sig = create_signal(0).unwrap();
        assert_eq!(sig.get().unwrap(), 0);

        sig.set(10).unwrap();
        assert_eq!(sig.get().unwrap(), 10);

        sig.update(|n| *n += 5).unwrap();
        assert_eq!(sig.get().unwrap(), 15);

        let (read, write) = sig.split();
        write.set(100).unwrap();
        assert_eq!(read.get().unwrap(), 100);

        dispose_signal(sig).unwrap();
        assert_eq!(sig.get().unwrap_err(), ReactiveError::SignalDisposed);
    });
}

#[test]
fn multiple_independent_signals() {
    with_runtime(|| {
        let a = create_signal(1).unwrap();
        let b = create_signal("hello").unwrap();
        let c = create_signal(vec![1, 2, 3]).unwrap();

        a.set(2).unwrap();
        assert_eq!(a.get().unwrap(), 2);
        assert_eq!(b.get().unwrap(), "hello");
        assert_eq!(c.with(|v| v.len()).unwrap(), 3);

        c.update(|v| v.push(4)).unwrap();
        assert_eq!(c.with(|v| v.len()).unwrap(), 4);
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
        })
        .unwrap();

        state
            .update(|s| {
                s.count += 1;
                s.label = "updated".into();
                s.items.push(42);
            })
            .unwrap();

        let snapshot = state.get().unwrap();
        assert_eq!(snapshot.count, 1);
        assert_eq!(snapshot.label, "updated");
        assert_eq!(snapshot.items, vec![42]);
    });
}

#[test]
fn split_signals_share_state() {
    with_runtime(|| {
        let (read, write) = create_signal_split(0).unwrap();

        write.set(42).unwrap();
        assert_eq!(read.get().unwrap(), 42);

        write.update(|n| *n *= 2).unwrap();
        assert_eq!(read.get().unwrap(), 84);

        // with() provides zero-clone access
        let is_even = read.with(|n| n % 2 == 0).unwrap();
        assert!(is_even);
    });
}

#[test]
fn with_untracked_provides_ref_access() {
    with_runtime(|| {
        let sig = create_signal(String::from("hello world")).unwrap();
        let word_count = sig
            .with_untracked(|s| s.split_whitespace().count())
            .unwrap();
        assert_eq!(word_count, 2);
    });
}

#[test]
fn slot_recycling_across_signals() {
    with_runtime(|| {
        let a = create_signal(1).unwrap();
        let b = create_signal(2).unwrap();

        dispose_signal(a).unwrap();

        // New signal should reuse a's slot
        let c = create_signal(3).unwrap();

        // b should still work
        assert_eq!(b.get().unwrap(), 2);
        // c should work with its own value
        assert_eq!(c.get().unwrap(), 3);
        // a should be dead
        assert_eq!(a.get().unwrap_err(), ReactiveError::SignalDisposed);
    });
}

#[test]
fn no_runtime_returns_error() {
    dispose_runtime();
    let result = create_signal(42);
    assert_eq!(result.unwrap_err(), ReactiveError::NoRuntime);
}

#[test]
fn runtime_reset_clears_signals() {
    initialize_runtime();
    let sig = create_signal(42).unwrap();
    assert_eq!(sig.get().unwrap(), 42);

    // Re-initialize clears everything
    initialize_runtime();
    // Old signal handle now refers to a dead/wrong generation slot
    // (the slot doesn't exist at all in the fresh runtime)
    assert_eq!(sig.get().unwrap_err(), ReactiveError::SignalDisposed);
    dispose_runtime();
}
