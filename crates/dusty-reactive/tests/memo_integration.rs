//! Integration tests for memos — cached derived computations.

use dusty_reactive::{
    create_memo, create_signal, dispose_memo, dispose_runtime, initialize_runtime, try_create_memo,
    try_dispose_memo, Memo, ReactiveError,
};
use std::cell::Cell;
use std::rc::Rc;

fn with_runtime(f: impl FnOnce()) {
    initialize_runtime();
    f();
    dispose_runtime();
}

#[test]
fn full_memo_lifecycle() {
    with_runtime(|| {
        let count = create_signal(1);
        let doubled = create_memo(move || count.get() * 2);

        assert_eq!(doubled.get(), 2);

        count.set(5);
        assert_eq!(doubled.get(), 10);

        count.set(0);
        assert_eq!(doubled.get(), 0);

        dispose_memo(&doubled);
        assert_eq!(doubled.try_get().unwrap_err(), ReactiveError::MemoDisposed);
    });
}

#[test]
fn memo_with_complex_types() {
    with_runtime(|| {
        #[derive(Clone, Debug, PartialEq)]
        struct User {
            name: String,
            age: u32,
        }

        let name = create_signal(String::from("Alice"));
        let age = create_signal(30_u32);

        let user = create_memo(move || User {
            name: name.get(),
            age: age.get(),
        });

        let u = user.get();
        assert_eq!(u.name, "Alice");
        assert_eq!(u.age, 30);

        name.set(String::from("Bob"));
        let u = user.get();
        assert_eq!(u.name, "Bob");
        assert_eq!(u.age, 30);

        age.set(25);
        let u = user.get();
        assert_eq!(u.name, "Bob");
        assert_eq!(u.age, 25);
    });
}

#[test]
fn memo_clone_shares_state() {
    with_runtime(|| {
        let source = create_signal(10);
        let m = create_memo(move || source.get() + 1);
        let m2 = m.clone();

        assert_eq!(m.get(), 11);
        assert_eq!(m2.get(), 11);

        source.set(20);
        assert_eq!(m.get(), 21);
        assert_eq!(m2.get(), 21);
    });
}

#[test]
fn memo_no_runtime_errors() {
    dispose_runtime();
    assert_eq!(try_create_memo(|| 0).unwrap_err(), ReactiveError::NoRuntime);
}

#[test]
fn multiple_independent_memos() {
    with_runtime(|| {
        let a = create_signal(1);
        let b = create_signal(2);

        let sum = create_memo(move || a.get() + b.get());
        let product = create_memo(move || a.get() * b.get());

        assert_eq!(sum.get(), 3);
        assert_eq!(product.get(), 2);

        a.set(10);
        assert_eq!(sum.get(), 12);
        assert_eq!(product.get(), 20);
    });
}

#[test]
fn diamond_dependency_correctness() {
    with_runtime(|| {
        let source = create_signal(2);
        let left = create_memo(move || source.get() + 1);
        let right = create_memo(move || source.get() * 10);
        let combined = create_memo(move || format!("{}:{}", left.get(), right.get()));

        assert_eq!(combined.get(), "3:20");

        source.set(5);
        assert_eq!(combined.get(), "6:50");
    });
}

#[test]
fn equality_check_prevents_cascading_reeval() {
    with_runtime(|| {
        let input = create_signal(0_i32);
        let sign = create_memo(move || input.get().signum());

        let eval_count = Rc::new(Cell::new(0_u32));
        let ec = Rc::clone(&eval_count);
        let label = create_memo(move || {
            ec.set(ec.get() + 1);
            match sign.get() {
                -1 => "negative",
                0 => "zero",
                _ => "positive",
            }
        });

        assert_eq!(label.get(), "zero");
        assert_eq!(eval_count.get(), 1);

        // 0 → 5: sign 0 → 1, label re-evals
        input.set(5);
        assert_eq!(label.get(), "positive");
        assert_eq!(eval_count.get(), 2);

        // 5 → 100: sign stays 1, label skips eval
        input.set(100);
        assert_eq!(label.get(), "positive");
        assert_eq!(eval_count.get(), 2);

        // 100 → -3: sign 1 → -1, label re-evals
        input.set(-3);
        assert_eq!(label.get(), "negative");
        assert_eq!(eval_count.get(), 3);

        // -3 → -99: sign stays -1, label skips eval
        input.set(-99);
        assert_eq!(label.get(), "negative");
        assert_eq!(eval_count.get(), 3);
    });
}

#[test]
fn dynamic_dependencies_switch_correctly() {
    with_runtime(|| {
        let use_first = create_signal(true);
        let first = create_signal(String::from("hello"));
        let second = create_signal(String::from("world"));

        let output = create_memo(move || {
            if use_first.get() {
                first.get()
            } else {
                second.get()
            }
        });

        assert_eq!(output.get(), "hello");

        second.set(String::from("WORLD"));
        // second is not a dependency, output unchanged
        assert_eq!(output.get(), "hello");

        use_first.set(false);
        assert_eq!(output.get(), "WORLD");

        first.set(String::from("HELLO"));
        // first is no longer a dependency
        assert_eq!(output.get(), "WORLD");
    });
}

#[test]
fn disposed_memo_double_dispose_errors() {
    with_runtime(|| {
        let m = create_memo(|| 42);
        assert_eq!(m.get(), 42);

        dispose_memo(&m);
        assert_eq!(
            try_dispose_memo(&m).unwrap_err(),
            ReactiveError::MemoDisposed
        );
    });
}

#[test]
fn memo_with_vec_dependency() {
    with_runtime(|| {
        let items = create_signal(vec![1, 2, 3]);
        let sum = create_memo(move || items.with(|v| v.iter().sum::<i32>()));
        let count = create_memo(move || items.with(|v| v.len()));

        assert_eq!(sum.get(), 6);
        assert_eq!(count.get(), 3);

        items.update(|v| v.push(4));
        assert_eq!(sum.get(), 10);
        assert_eq!(count.get(), 4);
    });
}

#[test]
fn deeply_nested_diamond() {
    with_runtime(|| {
        //     s
        //    / \
        //   a   b
        //    \ /
        //     c
        //    / \
        //   d   e
        //    \ /
        //     f
        let s = create_signal(1);
        let a = create_memo(move || s.get() + 1);
        let b = create_memo(move || s.get() * 2);
        let c = create_memo(move || a.get() + b.get());
        let c2 = c.clone();
        let d = create_memo(move || c.get() + 10);
        let e = create_memo(move || c2.get() * 3);
        let f = create_memo(move || d.get() + e.get());

        // s=1: a=2, b=2, c=4, d=14, e=12, f=26
        assert_eq!(f.get(), 26);

        s.set(3);
        // s=3: a=4, b=6, c=10, d=20, e=30, f=50
        assert_eq!(f.get(), 50);
    });
}

#[test]
fn memo_untracked_read_in_integration() {
    with_runtime(|| {
        let source = create_signal(5);
        let m = create_memo(move || source.get() * 2);

        // with_untracked should not register as a subscriber
        let val = m.with_untracked(|v| *v);
        assert_eq!(val, 10);

        // Verify memo still works normally after untracked read
        source.set(10);
        assert_eq!(m.get(), 20);
    });
}

#[test]
fn runtime_reset_invalidates_memos() {
    initialize_runtime();
    let m = create_memo(|| 42);
    assert_eq!(m.get(), 42);

    // Reset — old memo handle becomes invalid
    initialize_runtime();
    assert_eq!(m.try_get().unwrap_err(), ReactiveError::MemoDisposed);
    dispose_runtime();
}

#[test]
fn memo_chain_with_string_transformations() {
    with_runtime(|| {
        let input = create_signal(String::from("  Hello World  "));
        let trimmed = create_memo(move || input.get().trim().to_string());
        let lower = create_memo(move || trimmed.get().to_lowercase());
        let words: Memo<Vec<String>> =
            create_memo(move || lower.get().split_whitespace().map(String::from).collect());

        assert_eq!(words.get(), vec!["hello", "world"]);

        input.set(String::from(" Rust Programming "));
        assert_eq!(words.get(), vec!["rust", "programming"]);
    });
}

#[test]
fn memo_evaluation_count_across_multiple_changes() {
    with_runtime(|| {
        let a = create_signal(1);
        let b = create_signal(10);

        let eval_count = Rc::new(Cell::new(0_u32));
        let ec = Rc::clone(&eval_count);

        let sum = create_memo(move || {
            ec.set(ec.get() + 1);
            a.get() + b.get()
        });

        // Initial
        assert_eq!(sum.get(), 11);
        assert_eq!(eval_count.get(), 1);

        // Multiple reads without changes — no re-eval
        for _ in 0..5 {
            assert_eq!(sum.get(), 11);
        }
        assert_eq!(eval_count.get(), 1);

        // Change a
        a.set(2);
        assert_eq!(sum.get(), 12);
        assert_eq!(eval_count.get(), 2);

        // Change b
        b.set(20);
        assert_eq!(sum.get(), 22);
        assert_eq!(eval_count.get(), 3);

        // Change both (two separate sets)
        a.set(3);
        b.set(30);
        assert_eq!(sum.get(), 33);
        // Could be 4 or 5 depending on whether first set triggers eval —
        // since we read lazily, only the final get() triggers eval
        assert!(eval_count.get() <= 5);
    });
}

#[test]
fn memo_with_option_type() {
    with_runtime(|| {
        let index = create_signal(0_usize);
        let items = vec!["a", "b", "c"];

        let selected = create_memo(move || {
            let i = index.get();
            items.get(i).copied()
        });

        assert_eq!(selected.get(), Some("a"));

        index.set(2);
        assert_eq!(selected.get(), Some("c"));

        index.set(5);
        assert_eq!(selected.get(), None);
    });
}
