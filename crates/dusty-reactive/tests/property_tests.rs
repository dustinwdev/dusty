//! Property-based tests for dusty-reactive primitives.

use dusty_reactive::*;
use proptest::prelude::*;

fn with_runtime<R>(f: impl FnOnce() -> R) -> R {
    initialize_runtime();
    let result = f();
    dispose_runtime();
    result
}

// ---------------------------------------------------------------------------
// Signal round-trip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn signal_get_set_round_trip(value in any::<i64>()) {
        let got = with_runtime(|| {
            let sig = create_signal(value).unwrap();
            sig.get().unwrap()
        });
        prop_assert_eq!(got, value);
    }

    #[test]
    fn signal_set_then_get_returns_new_value(init in any::<i32>(), new in any::<i32>()) {
        let got = with_runtime(|| {
            let sig = create_signal(init).unwrap();
            sig.set(new).unwrap();
            sig.get().unwrap()
        });
        prop_assert_eq!(got, new);
    }

    #[test]
    fn signal_update_applies_mutation(init in 0i32..1000, delta in 0i32..1000) {
        let got = with_runtime(|| {
            let sig = create_signal(init).unwrap();
            sig.update(|v| *v += delta).unwrap();
            sig.get().unwrap()
        });
        prop_assert_eq!(got, init + delta);
    }
}

// ---------------------------------------------------------------------------
// Memo consistency
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn memo_always_consistent_with_source(a in any::<i32>(), b in any::<i32>()) {
        let (first, second) = with_runtime(|| {
            let sig = create_signal(a).unwrap();
            let memo = create_memo(move || sig.get().unwrap().wrapping_mul(2)).unwrap();
            let first = memo.get().unwrap();

            sig.set(b).unwrap();
            let second = memo.get().unwrap();
            (first, second)
        });
        prop_assert_eq!(first, a.wrapping_mul(2));
        prop_assert_eq!(second, b.wrapping_mul(2));
    }

    #[test]
    fn memo_chain_consistent(val in -1000i32..1000) {
        let got = with_runtime(|| {
            let src = create_signal(val).unwrap();
            let m1 = create_memo(move || src.get().unwrap() + 1).unwrap();
            let m2 = create_memo(move || m1.get().unwrap() * 2).unwrap();
            m2.get().unwrap()
        });
        prop_assert_eq!(got, (val + 1) * 2);
    }
}

// ---------------------------------------------------------------------------
// Batch equivalence
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn batch_final_value_matches_last_set(a in any::<i32>(), b in any::<i32>()) {
        let got = with_runtime(|| {
            let sig = create_signal(0i32).unwrap();
            batch(|| {
                sig.set(a).unwrap();
                sig.set(b).unwrap();
            }).unwrap();
            sig.get().unwrap()
        });
        prop_assert_eq!(got, b);
    }
}

// ---------------------------------------------------------------------------
// Scope disposal completeness
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn scope_disposes_all_signals(count in 1usize..20) {
        with_runtime(|| {
            let mut handles = Vec::new();
            let scope = create_scope(|_cx| {
                for i in 0..count {
                    let sig = create_signal(i as i32).unwrap();
                    handles.push(sig);
                }
            }).unwrap();

            // All signals work before disposal
            for (i, sig) in handles.iter().enumerate() {
                assert_eq!(sig.get().unwrap(), i as i32);
            }

            dispose_scope(scope).unwrap();

            // All signals return error after disposal
            for sig in &handles {
                assert!(sig.get().is_err());
            }
        });
    }
}
