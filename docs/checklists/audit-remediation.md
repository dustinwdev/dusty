# Audit Remediation Checklist

Phased plan to address findings from the `dusty-reactive` comprehensive audit (2026-03-12). Each phase is scoped to complete in a single focused session.

---

## Phase 1: Soundness & Critical Safety

The two issues that can cause unsound behavior or permanent runtime corruption.

- [x] **Fix `!Send + !Sync` on all handle types**
  - Add `PhantomData<*const ()>` (or `Rc<()>`) to `Signal<T>`, `ReadSignal<T>`, `WriteSignal<T>` to make them `!Send + !Sync`
  - Add the same marker to `Memo<T>`, `Effect`, `Resource<T>`, `Scope`
  - Replace the no-op `signal_not_send` test with `static_assertions::assert_not_impl_any!` compile-time checks for all handle types
  - Add `static_assertions` as a dev-dependency

- [x] **Fix `batch()` panic safety**
  - Wrap `batch_depth` decrement + flush in a drop guard so it runs on unwind
  - Add test: panic inside `batch`, catch with `catch_unwind`, verify signal notifications still work afterward

- [x] **Fix effect execution panic safety**
  - Wrap `push_tracking`/`pop_tracking` in `execute_effect` with a drop guard
  - Reset `state.running` to `false` on unwind
  - Restore `CLEANUP_SINK` on unwind
  - Add test: panic inside effect closure, verify tracking stack is clean and effect can be disposed

- [x] **Fix memo evaluation panic safety**
  - Wrap `push_tracking`/`pop_tracking` in `evaluate_memo` with a drop guard
  - Add test: panic inside memo computation, verify tracking stack is clean and memo returns error on next `.get()`

- [x] **Fix `untrack` panic safety**
  - Restore tracking stack and dependency stack on unwind (drop guard)
  - Add test: panic inside `untrack` closure, verify tracking is restored

- [x] **Fix `Scope::run` / `create_scope` / `create_child_scope` panic safety**
  - Wrap `push_scope`/`pop_scope` in drop guards
  - Add test: panic inside `scope.run()`, verify scope stack is clean

---

## Phase 2: Correctness Bugs

Logic bugs that produce wrong behavior in specific scenarios.

- [x] **Fix `dispose_memo` destroying downstream subscribers**
  - Change `dispose_memo` to only remove downstream subscribers from the memo's signal slot subscriber list, NOT call `unregister_subscriber` on them
  - Downstream memos/effects should continue to work with their other dependencies
  - Add test: effect depends on memo M and signal S, dispose M, verify effect still responds to S changes

- [x] **Add generational index to `SubscriberId`**
  - Add `generation: u64` field to `SubscriberId`
  - Track generation in subscriber storage (add a parallel `Vec<u64>` or wrap in a struct)
  - Check generation on every subscriber invocation (notification loops)
  - Check generation on `unregister_subscriber` to prevent double-free of subscriber slots
  - Add test: unregister subscriber, register new one at same slot, verify old ID does not invoke new callback
  - Add test: double `unregister_subscriber` does not corrupt free list

- [x] **Make memo notification batch-aware**
  - `propagate_dirty` in `memo.rs` uses `invoke_subscriber` with generation check; only called when `batch_depth == 0` by construction
  - `update_and_notify` in `memo.rs` checks `batch_depth` and defers notification if batching
  - `ensure_fresh_inner` checks dep versions when inside a batch even if dirty flag is not set (subscriber callback deferred)
  - Add test: batch containing signal writes that trigger memo freshening, verify subscribers notified exactly once after batch

- [x] **Fix `current_tracking()` to use immutable borrow**
  - Change `with_runtime_mut` to `with_runtime` in `current_tracking()` (subscriber.rs)
  - Same fix for `current_scope()` and `validate_scope()` in scope.rs
  - Add test (or verify existing): subscriber callback that reads a signal during notification does not hit `RuntimeBorrowError`

---

## Phase 3: Thread-Local Cleanup & Error Reporting

Leak fixes and error quality improvements.

- [x] **Clear auxiliary thread-locals on `dispose_runtime()`**
  - Clear `FRESHENERS` map in memo.rs
  - Clear `PENDING_EFFECTS` vec in effect.rs
  - Clear `CLEANUP_SINK` in effect.rs
  - Add test: initialize runtime, create memos/effects, dispose runtime, re-initialize, verify no stale fresheners or pending effects interfere

- [x] **Add missing error variants**
  - Add `MemoDisposed`, `EffectDisposed`, `ResourceDisposed` to `ReactiveError` (or a single `Disposed { kind: &'static str }`)
  - Add `TypeMismatch` variant for downcast failures in `with_signal_value` and `set_and_notify`
  - Update `Display` impl with appropriate messages
  - Update all call sites to use the correct variant
  - Update tests that assert on `ReactiveError::SignalDisposed` for memos/effects/resources

- [x] **Fix downcast failure error path**
  - In `with_signal_value` and `set_and_notify`, return `TypeMismatch` instead of `SignalDisposed` when `downcast_ref`/`downcast_mut` fails
  - Add a doc comment explaining this should be unreachable through the safe API

---

## Phase 4: Performance — Hot Path

Optimize the `signal.get()` and `signal.set()` hot paths.

- [ ] **Replace `Vec<SubscriberId>` with `HashSet<SubscriberId>` on `SignalSlot.subscribers`**
  - Fixes O(n) `.contains()` in `track_signal` (every `.get()`)
  - Fixes O(n) `.retain()` in unsubscribe-from-deps (every effect/memo re-eval)
  - Update all iteration sites (notification loops clone or iterate the set)
  - Update `flush_batch` dedup — with `HashSet` subscribers, the `seen` dedup in `flush_batch` can use the same approach or `pending_batch_subscribers` can be a `HashSet` directly

- [ ] **Reduce runtime borrow cycles on `.get()`**
  - Change `current_tracking()` to use `with_runtime` (if not done in Phase 2)
  - Combine tracking + value read into a single `with_runtime` call where possible, or at minimum reduce from 3 borrow cycles to 2
  - Benchmark before/after with a micro-benchmark (e.g., 1M signal reads in a loop)

- [ ] **Use `SmallVec` for subscriber collection in `set_and_notify`**
  - Replace `slot.subscribers.clone()` with collection into a `SmallVec<[SubscriberId; 8]>` scratch buffer
  - Avoids heap allocation for signals with <= 8 subscribers (the common case)
  - Add `smallvec` as a dependency
  - Apply same optimization in `update_and_notify` (memo.rs) and `propagate_dirty` (memo.rs)

---

## Phase 5: Performance — Secondary

Optimizations with moderate impact.

- [ ] **Move freshener registry into the runtime**
  - Add an `Option<FreshenerFn>` field (or a flag + registry Vec) to `SignalSlot` or `Runtime`
  - Remove the `FRESHENERS` thread-local `HashMap` in memo.rs
  - Update `register_freshener`, `unregister_freshener`, `get_freshener` to use the runtime
  - Eliminates second thread-local access + HashMap lookup on every memo `.get()`

- [ ] **Replace `Box<dyn FnOnce()>` disposers with a `Disposer` enum**
  - Define `enum Disposer { Signal(SignalId), Memo(SignalId, ...), Effect(...), Custom(Box<dyn FnOnce()>) }`
  - Store `Vec<Disposer>` in `ScopeSlot` instead of `Vec<Box<dyn FnOnce()>>`
  - Avoids heap allocation per signal/memo/effect creation for the common cases

- [ ] **Optimize `untrack` to avoid Vec swap**
  - Instead of `std::mem::take` on tracking/dependency stacks, record the stack length and truncate on restore
  - Avoids allocating/deallocating two Vecs per `untrack` call

---

## Phase 6: Code Deduplication

Extract shared patterns to reduce repetition and improve maintainability.

- [ ] **Extract shared tracking types to a `tracking` module**
  - Move `DepInfo` struct (currently duplicated in memo.rs and effect.rs) to a shared `pub(crate) mod tracking`
  - Move `FreshenerFn` type alias to the same module
  - Update imports in memo.rs and effect.rs

- [ ] **Extract `notify_subscribers` helper**
  - Create `pub(crate) fn notify_subscribers(subs: &[SubscriberId]) -> Result<()>` in subscriber.rs
  - Replace the 4 duplicated notification loops in signal.rs, memo.rs (2x), and batch.rs

- [ ] **Extract `unsubscribe_from_deps` helper**
  - Create `pub(crate) fn unsubscribe_from_deps(deps: &[DepInfo], sub_id: SubscriberId)` in the tracking module
  - Replace the 4 duplicated unsubscribe loops in memo.rs (2x) and effect.rs (2x)

- [ ] **Extract `capture_deps` helper**
  - Create `pub(crate) fn capture_deps(signal_ids: &[SignalId]) -> Result<Vec<DepInfo>>` in the tracking module
  - Replace the 2 duplicated dependency capture blocks in memo.rs and effect.rs

- [ ] **Merge `update_and_notify` / `update_without_notify` in memo.rs**
  - Combine into a single function with a `notify: bool` parameter

- [ ] **Consolidate `with_test_runtime` helper**
  - Define once in a `#[cfg(test)]` test utilities module
  - Re-export for integration tests via `#[doc(hidden)] pub mod test_util`

---

## Phase 7: Test Coverage Gaps

Fill missing test scenarios.

- [ ] **Add property-based tests with `proptest`**
  - Add `proptest` as a dev-dependency
  - Signal get/set round-trip: `forall x: T, signal.set(x); signal.get() == x`
  - Memo consistency: for arbitrary sequences of signal updates, `memo.get()` always equals the derived value
  - Batch equivalence: applying N signal writes inside `batch` produces the same final state as outside `batch`
  - Scope disposal completeness: for arbitrary scope trees, disposing root leaves all handles returning error

- [ ] **Add edge case tests: effect creating signals/effects during execution**
  - Effect that calls `create_signal` during its body — verify the new signal is owned by the correct scope
  - Effect that calls `create_effect` during its body — verify nested effect runs and is independently disposable

- [ ] **Add edge case tests: memo reading a disposed dependency**
  - Memo depends on signal S, dispose S, then call `memo.get()` — verify it returns an error (not a panic)

- [ ] **Add cross-module interaction tests**
  - `batch` + `scope`: dispose a scope while a batch is in progress
  - `scope.run()` + memo/effect: create memos and effects via `scope.run()`, verify they are owned by the scope
  - `untrack` inside `batch` and `batch` inside `untrack`
  - Effect that depends on a memo whose upstream memo is disposed

- [ ] **Add slot recycling stress test**
  - Create and dispose hundreds of signals/memos/effects in a loop
  - Verify generation counters and free lists remain consistent
  - Verify old handles always return errors, new handles always work

- [ ] **Strengthen weak test assertions**
  - `effect_reentrance_guard`: assert exact expected `run_count` instead of `< 200`
  - `resource_state_is_tracked`: assert exact run counts
  - `memo_evaluation_count_across_multiple_changes`: assert exact eval count instead of `<= 5`

---

## Phase 8: API Ergonomics (Design Decision)

These are design questions that may change the public API. Discuss before implementing.

- [ ] **Consider panicking default APIs**
  - Add `get`, `set`, `update` as panicking methods (like Leptos)
  - Rename current methods to `try_get`, `try_set`, `try_update`
  - `NoRuntime` and `SignalDisposed` become panics (programmer errors), not recoverable errors
  - This is a breaking API change — decide before other crates depend on `dusty-reactive`

- [ ] **Consider making `Memo<T>` `Copy`**
  - Store `MemoInner` in a thread-local registry keyed by `SignalId` (already in a signal slot)
  - `Memo<T>` becomes a lightweight `Copy` handle like `Signal<T>`
  - Improves ergonomics: no `.clone()` needed when capturing in multiple closures

- [ ] **Consider making `Effect` `Copy`**
  - Same approach — store `EffectInner` in a thread-local registry
  - `Effect` becomes a `Copy` handle

- [ ] **Add missing reactive primitives for GUI layer**
  - `on(deps, fn)` — explicit dependency declaration
  - `create_selector` — optimized keyed comparison for list rendering
  - `Signal::get_untracked()` convenience method (cloning variant of `with_untracked`)
  - `map_array` / `index_array` — reactive list transformations
  - These can be added incrementally as `dusty-core` and `dusty-widgets` need them
