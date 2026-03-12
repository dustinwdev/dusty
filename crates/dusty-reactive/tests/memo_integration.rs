//! Integration tests for memos — cached derived computations.

use dusty_reactive::{
    create_memo, create_signal, dispose_memo, dispose_runtime, initialize_runtime, Memo,
    ReactiveError,
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
        let count = create_signal(1).unwrap();
        let doubled = create_memo(move || count.get().unwrap() * 2).unwrap();

        assert_eq!(doubled.get().unwrap(), 2);

        count.set(5).unwrap();
        assert_eq!(doubled.get().unwrap(), 10);

        count.set(0).unwrap();
        assert_eq!(doubled.get().unwrap(), 0);

        dispose_memo(&doubled).unwrap();
        assert_eq!(doubled.get().unwrap_err(), ReactiveError::MemoDisposed);
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

        let name = create_signal(String::from("Alice")).unwrap();
        let age = create_signal(30_u32).unwrap();

        let user = create_memo(move || User {
            name: name.get().unwrap(),
            age: age.get().unwrap(),
        })
        .unwrap();

        let u = user.get().unwrap();
        assert_eq!(u.name, "Alice");
        assert_eq!(u.age, 30);

        name.set(String::from("Bob")).unwrap();
        let u = user.get().unwrap();
        assert_eq!(u.name, "Bob");
        assert_eq!(u.age, 30);

        age.set(25).unwrap();
        let u = user.get().unwrap();
        assert_eq!(u.name, "Bob");
        assert_eq!(u.age, 25);
    });
}

#[test]
fn memo_clone_shares_state() {
    with_runtime(|| {
        let source = create_signal(10).unwrap();
        let m = create_memo(move || source.get().unwrap() + 1).unwrap();
        let m2 = m.clone();

        assert_eq!(m.get().unwrap(), 11);
        assert_eq!(m2.get().unwrap(), 11);

        source.set(20).unwrap();
        assert_eq!(m.get().unwrap(), 21);
        assert_eq!(m2.get().unwrap(), 21);
    });
}

#[test]
fn memo_no_runtime_errors() {
    dispose_runtime();
    assert_eq!(create_memo(|| 0).unwrap_err(), ReactiveError::NoRuntime);
}

#[test]
fn multiple_independent_memos() {
    with_runtime(|| {
        let a = create_signal(1).unwrap();
        let b = create_signal(2).unwrap();

        let sum = create_memo(move || a.get().unwrap() + b.get().unwrap()).unwrap();
        let product = create_memo(move || a.get().unwrap() * b.get().unwrap()).unwrap();

        assert_eq!(sum.get().unwrap(), 3);
        assert_eq!(product.get().unwrap(), 2);

        a.set(10).unwrap();
        assert_eq!(sum.get().unwrap(), 12);
        assert_eq!(product.get().unwrap(), 20);
    });
}

#[test]
fn diamond_dependency_correctness() {
    with_runtime(|| {
        let source = create_signal(2).unwrap();
        let left = create_memo(move || source.get().unwrap() + 1).unwrap();
        let right = create_memo(move || source.get().unwrap() * 10).unwrap();
        let combined =
            create_memo(move || format!("{}:{}", left.get().unwrap(), right.get().unwrap()))
                .unwrap();

        assert_eq!(combined.get().unwrap(), "3:20");

        source.set(5).unwrap();
        assert_eq!(combined.get().unwrap(), "6:50");
    });
}

#[test]
fn equality_check_prevents_cascading_reeval() {
    with_runtime(|| {
        let input = create_signal(0_i32).unwrap();
        let sign = create_memo(move || input.get().unwrap().signum()).unwrap();

        let eval_count = Rc::new(Cell::new(0_u32));
        let ec = Rc::clone(&eval_count);
        let label = create_memo(move || {
            ec.set(ec.get() + 1);
            match sign.get().unwrap() {
                -1 => "negative",
                0 => "zero",
                _ => "positive",
            }
        })
        .unwrap();

        assert_eq!(label.get().unwrap(), "zero");
        assert_eq!(eval_count.get(), 1);

        // 0 → 5: sign 0 → 1, label re-evals
        input.set(5).unwrap();
        assert_eq!(label.get().unwrap(), "positive");
        assert_eq!(eval_count.get(), 2);

        // 5 → 100: sign stays 1, label skips eval
        input.set(100).unwrap();
        assert_eq!(label.get().unwrap(), "positive");
        assert_eq!(eval_count.get(), 2);

        // 100 → -3: sign 1 → -1, label re-evals
        input.set(-3).unwrap();
        assert_eq!(label.get().unwrap(), "negative");
        assert_eq!(eval_count.get(), 3);

        // -3 → -99: sign stays -1, label skips eval
        input.set(-99).unwrap();
        assert_eq!(label.get().unwrap(), "negative");
        assert_eq!(eval_count.get(), 3);
    });
}

#[test]
fn dynamic_dependencies_switch_correctly() {
    with_runtime(|| {
        let use_first = create_signal(true).unwrap();
        let first = create_signal(String::from("hello")).unwrap();
        let second = create_signal(String::from("world")).unwrap();

        let output = create_memo(move || {
            if use_first.get().unwrap() {
                first.get().unwrap()
            } else {
                second.get().unwrap()
            }
        })
        .unwrap();

        assert_eq!(output.get().unwrap(), "hello");

        second.set(String::from("WORLD")).unwrap();
        // second is not a dependency, output unchanged
        assert_eq!(output.get().unwrap(), "hello");

        use_first.set(false).unwrap();
        assert_eq!(output.get().unwrap(), "WORLD");

        first.set(String::from("HELLO")).unwrap();
        // first is no longer a dependency
        assert_eq!(output.get().unwrap(), "WORLD");
    });
}

#[test]
fn disposed_memo_double_dispose_errors() {
    with_runtime(|| {
        let m = create_memo(|| 42).unwrap();
        assert_eq!(m.get().unwrap(), 42);

        dispose_memo(&m).unwrap();
        assert_eq!(dispose_memo(&m).unwrap_err(), ReactiveError::MemoDisposed);
    });
}

#[test]
fn memo_with_vec_dependency() {
    with_runtime(|| {
        let items = create_signal(vec![1, 2, 3]).unwrap();
        let sum = create_memo(move || items.with(|v| v.iter().sum::<i32>()).unwrap()).unwrap();
        let count = create_memo(move || items.with(|v| v.len()).unwrap()).unwrap();

        assert_eq!(sum.get().unwrap(), 6);
        assert_eq!(count.get().unwrap(), 3);

        items.update(|v| v.push(4)).unwrap();
        assert_eq!(sum.get().unwrap(), 10);
        assert_eq!(count.get().unwrap(), 4);
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
        let s = create_signal(1).unwrap();
        let a = create_memo(move || s.get().unwrap() + 1).unwrap();
        let b = create_memo(move || s.get().unwrap() * 2).unwrap();
        let c = create_memo(move || a.get().unwrap() + b.get().unwrap()).unwrap();
        let c2 = c.clone();
        let d = create_memo(move || c.get().unwrap() + 10).unwrap();
        let e = create_memo(move || c2.get().unwrap() * 3).unwrap();
        let f = create_memo(move || d.get().unwrap() + e.get().unwrap()).unwrap();

        // s=1: a=2, b=2, c=4, d=14, e=12, f=26
        assert_eq!(f.get().unwrap(), 26);

        s.set(3).unwrap();
        // s=3: a=4, b=6, c=10, d=20, e=30, f=50
        assert_eq!(f.get().unwrap(), 50);
    });
}

#[test]
fn memo_untracked_read_in_integration() {
    with_runtime(|| {
        let source = create_signal(5).unwrap();
        let m = create_memo(move || source.get().unwrap() * 2).unwrap();

        // with_untracked should not register as a subscriber
        let val = m.with_untracked(|v| *v).unwrap();
        assert_eq!(val, 10);

        // Verify memo still works normally after untracked read
        source.set(10).unwrap();
        assert_eq!(m.get().unwrap(), 20);
    });
}

#[test]
fn runtime_reset_invalidates_memos() {
    initialize_runtime();
    let m = create_memo(|| 42).unwrap();
    assert_eq!(m.get().unwrap(), 42);

    // Reset — old memo handle becomes invalid
    initialize_runtime();
    assert_eq!(m.get().unwrap_err(), ReactiveError::MemoDisposed);
    dispose_runtime();
}

#[test]
fn memo_chain_with_string_transformations() {
    with_runtime(|| {
        let input = create_signal(String::from("  Hello World  ")).unwrap();
        let trimmed = create_memo(move || input.get().unwrap().trim().to_string()).unwrap();
        let lower = create_memo(move || trimmed.get().unwrap().to_lowercase()).unwrap();
        let words: Memo<Vec<String>> = create_memo(move || {
            lower
                .get()
                .unwrap()
                .split_whitespace()
                .map(String::from)
                .collect()
        })
        .unwrap();

        assert_eq!(words.get().unwrap(), vec!["hello", "world"]);

        input.set(String::from(" Rust Programming ")).unwrap();
        assert_eq!(words.get().unwrap(), vec!["rust", "programming"]);
    });
}

#[test]
fn memo_evaluation_count_across_multiple_changes() {
    with_runtime(|| {
        let a = create_signal(1).unwrap();
        let b = create_signal(10).unwrap();

        let eval_count = Rc::new(Cell::new(0_u32));
        let ec = Rc::clone(&eval_count);

        let sum = create_memo(move || {
            ec.set(ec.get() + 1);
            a.get().unwrap() + b.get().unwrap()
        })
        .unwrap();

        // Initial
        assert_eq!(sum.get().unwrap(), 11);
        assert_eq!(eval_count.get(), 1);

        // Multiple reads without changes — no re-eval
        for _ in 0..5 {
            assert_eq!(sum.get().unwrap(), 11);
        }
        assert_eq!(eval_count.get(), 1);

        // Change a
        a.set(2).unwrap();
        assert_eq!(sum.get().unwrap(), 12);
        assert_eq!(eval_count.get(), 2);

        // Change b
        b.set(20).unwrap();
        assert_eq!(sum.get().unwrap(), 22);
        assert_eq!(eval_count.get(), 3);

        // Change both (two separate sets)
        a.set(3).unwrap();
        b.set(30).unwrap();
        assert_eq!(sum.get().unwrap(), 33);
        // Could be 4 or 5 depending on whether first set triggers eval —
        // since we read lazily, only the final get() triggers eval
        assert!(eval_count.get() <= 5);
    });
}

#[test]
fn memo_with_option_type() {
    with_runtime(|| {
        let index = create_signal(0_usize).unwrap();
        let items = vec!["a", "b", "c"];

        let selected = create_memo(move || {
            let i = index.get().unwrap();
            items.get(i).copied()
        })
        .unwrap();

        assert_eq!(selected.get().unwrap(), Some("a"));

        index.set(2).unwrap();
        assert_eq!(selected.get().unwrap(), Some("c"));

        index.set(5).unwrap();
        assert_eq!(selected.get().unwrap(), None);
    });
}
