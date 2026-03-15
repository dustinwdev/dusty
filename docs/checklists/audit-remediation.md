# Audit Remediation Checklist

Combines findings from the initial `dusty-reactive` audit (2026-03-12) and the comprehensive full-workspace audit (2026-03-14). Each phase is scoped to complete in a single focused session.

---

## Phase 1: Soundness & Critical Safety (reactive)

*Completed 2026-03-12.*

- [x] **Fix `!Send + !Sync` on all handle types**
- [x] **Fix `batch()` panic safety**
- [x] **Fix effect execution panic safety**
- [x] **Fix memo evaluation panic safety**
- [x] **Fix `untrack` panic safety**
- [x] **Fix `Scope::run` / `create_scope` / `create_child_scope` panic safety**

---

## Phase 2: Correctness Bugs (reactive)

*Completed 2026-03-12.*

- [x] **Fix `dispose_memo` destroying downstream subscribers**
- [x] **Add generational index to `SubscriberId`**
- [x] **Make memo notification batch-aware**
- [x] **Fix `current_tracking()` to use immutable borrow**

---

## Phase 3: Thread-Local Cleanup & Error Reporting (reactive)

*Completed 2026-03-12.*

- [x] **Clear auxiliary thread-locals on `dispose_runtime()`**
- [x] **Add missing error variants**
- [x] **Fix downcast failure error path**

---

## Phase 4: Performance — Hot Path (reactive)

*Completed 2026-03-15.*

- [x] **Replace `Vec<SubscriberId>` with `HashSet<SubscriberId>` on `SignalSlot.subscribers`**
- [x] **Reduce runtime borrow cycles on `.get()`**
- [x] **Use `SmallVec` for subscriber collection in `set_and_notify`**

---

## Phase 5: Performance — Secondary (reactive)

- [x] **Move freshener registry into the runtime**
  - Fresheners now stored in `Runtime.fresheners` HashMap; eliminated separate thread-local

- [ ] **Replace `Box<dyn FnOnce()>` disposers with a `Disposer` enum**
  - Avoids heap allocation per signal/memo/effect creation for the common cases
  - Skipped: large refactor, low impact

- ~~**Optimize `untrack` to avoid Vec swap**~~
  - Skipped: `mem::take` on empty Vec is zero-alloc; no real gain

- [x] **Memo deps Vec → SmallVec<[DepInfo; 4]>** — `dusty-reactive/src/memo.rs`
  - Avoids heap allocation for memos with 1-4 deps (most common case)

- [x] **Effect cleanup Vec reuse** — `dusty-reactive/src/effect.rs`
  - Reuse existing Vec via `clear()` instead of allocating new `Some(Vec::new())`

- [x] **Resource generation overflow** — `dusty-reactive/src/resource.rs`
  - Changed `gen.get() + 1` to `gen.get().saturating_add(1)`

- [x] **Canvas borrow_mut panic guard** — `dusty-widgets/src/canvas/mod.rs`
  - Replaced `borrow_mut()` with `try_borrow_mut()` + graceful fallback

- [x] **TextSystem borrow_mut panics** — `dusty-text/src/system.rs`
  - `font_system_mut()` now returns `Result<RefMut>` with `BorrowConflict` error variant
  - `TextMeasure::measure` uses `try_borrow_mut()` with debug_assert fallback
  - `TextLayout::new/new_rich` return `Result` propagating borrow errors

- [x] **pop_tracking silent underflow** — `dusty-reactive/src/subscriber.rs`
  - Added `debug_assert!` on tracking stack underflow instead of silent `unwrap_or_default()`

---

## Phase 6: Code Deduplication (reactive)

*Completed 2026-03-15.*

- [x] **Extract shared tracking types to a `tracking` module**
  - Created `tracking.rs` with `unsubscribe_from_signals`, `notify_subscribers`, and `with_test_runtime`

- [x] **Extract `notify_subscribers` helper**
  - Replaced notification loops in signal.rs and memo.rs with shared `tracking::notify_subscribers`

- [x] **Extract `unsubscribe_from_deps` helper**
  - Replaced 5 unsubscribe loops in memo.rs (2x) and effect.rs (3x) with `tracking::unsubscribe_from_signals`

- [ ] **Extract `capture_deps` helper**
  - Skipped: push_tracking/catch_unwind/pop_tracking patterns differ enough between memo and effect

- [x] **Merge `update_and_notify` / `update_without_notify` in memo.rs**
  - Merged into `update_memo_slot(signal_id, value, notify: bool)`

- [x] **Consolidate `with_test_runtime` helper**
  - Defined once in `tracking.rs` under `#[cfg(test)]`; used by all 6 reactive module test suites

---

## Phase 7: Test Coverage Gaps (reactive)

*Completed 2026-03-15.*

- [x] **Add property-based tests with `proptest`**
  - `tests/property_tests.rs`: signal round-trip, memo consistency, batch equivalence, scope disposal

- [x] **Add edge case tests: effect creating signals/effects during execution**
  - `tests/edge_cases.rs`: effect_creates_signal_during_execution, effect_creates_effect_during_execution

- [x] **Add edge case tests: memo reading a disposed dependency**
  - `tests/edge_cases.rs`: memo_handles_disposed_dependency_gracefully

- [x] **Add cross-module interaction tests**
  - `tests/edge_cases.rs`: batch_inside_scope, untrack_inside_batch

- [x] **Add slot recycling stress test**
  - `tests/edge_cases.rs`: rapid_signal_create_dispose_does_not_corrupt (100 signals)

- [x] **Strengthen weak test assertions**
  - `tests/edge_cases.rs`: resource_state_tracked_exact_run_count, signal_set_during_batch_flush

---

## Phase 8: API Ergonomics (reactive, design decisions)

*Design docs written 2026-03-15.*

- [x] **Consider panicking default APIs** — `docs/design/panicking-api.md`
- [x] **Consider making `Memo<T>` and `Effect` `Copy`** — `docs/design/copy-handles.md`
- [ ] **Add missing reactive primitives for GUI layer** (`on`, `create_selector`, `get_untracked`, `map_array`)
  - Deferred to feature work

---

# Comprehensive Audit Findings (2026-03-14)

Findings from the full-workspace audit covering dusty-reactive, dusty-core, dusty-style, dusty-layout, dusty-text, and architecture.

---

## Phase 9: Critical Bugs

The highest priority fixes — correctness bugs that produce wrong behavior or data corruption.

- [x] **Fix subscriber double-free in `unregister_subscriber`** — `dusty-reactive/src/subscriber.rs:45-52`
  - Added `is_some()` guard before pushing to free list
  - Strengthened `double_unregister_does_not_corrupt_free_list` test

- [x] **Fix `Style::resolve()` leaking nested state overrides** — `dusty-style/src/style.rs:286-315`
  - Added second clearing of state fields after all merges complete
  - Added `resolve_clears_nested_state_from_overrides` test

- [x] **Fix `TextSpan::size` being silently ignored** — `dusty-text/src/rich.rs:28`, `dusty-text/src/convert.rs:70-90`
  - `span_to_cosmic()` never reads the `size` field
  - Setting `TextSpan::new("big").size(48.0)` has no effect
  - Removed the field entirely — cosmic-text uses per-buffer `Metrics` for sizing, not per-span

- [x] **Fix `line_height` semantic mismatch** — `dusty-text/src/convert.rs:61-67`, `dusty-style/src/font.rs:80`
  - Documented as "Line height multiplier" but explicit values treated as absolute pixels
  - `line_height: Some(1.5)` gives 1.5px instead of `font_size * 1.5`
  - Fixed: explicit values now treated as multipliers (`font_size * line_height`)

---

## Phase 10: High — Reactive Runtime

- [x] **Add `set_if_changed` or document always-notify behavior** — `dusty-reactive/src/signal.rs`
  - Added `set_if_changed` method on `Signal<T>` and `WriteSignal<T>` (returns `Result<bool>`)
  - Added doc comments on existing `set` noting it always notifies

- [x] **Fix effect re-entrancy guard silently dropping updates** — `dusty-reactive/src/effect.rs`
  - Moved `dirty.set(false)` to start of execution; re-queue if dirty re-set during execution
  - Test asserts signal converges to 3, effect runs exactly 4 times

- [x] **Fix `flush_pending_effects` swallowing all errors** — `dusty-reactive/src/effect.rs`
  - Replaced `let _ =` with `debug_assert!` on error

- [x] **Fix `initialize_runtime` dropping state without running cleanup** — `dusty-reactive/src/runtime.rs`
  - `initialize_runtime` now calls `dispose_runtime()` first

- [x] **Document/fix fragile nested borrow chain in `propagate_dirty`** — `dusty-reactive/src/memo.rs`
  - Added `// INVARIANT:` comment explaining collect-then-invoke ordering

---

## Phase 11: High — Core & Events

- [x] **Fix `ElementBuilder::attr` allowing duplicate attribute names** — `dusty-core/src/element.rs`
  - Changed to upsert: existing key's value is replaced in place

- [x] **Fix `PartialEq` on `f64`-containing types (NaN footgun)** — `dusty-core/src/element.rs`, `event.rs`
  - Manual `PartialEq` with `f64::total_cmp()` on `AttributeValue`, `ClickEvent`, `HoverEvent`, `ScrollEvent`

- [x] **Fix silent downcast failure on typed event handlers** — `dusty-core/src/element.rs`
  - Added `debug_assert!` on downcast failure in `on_event`

- [x] **Fix `children_mut()` returning `&mut Vec<Node>`** — `dusty-core/src/element.rs`
  - Returns `&mut [Node]` now; added `push_child()` method

---

## Phase 12: High — Style

- [x] **Fix palette stop methods silently ignoring invalid values** — `dusty-style/src/builder.rs`
  - Added `debug_assert!` on invalid palette stops in macro

- [x] **Add input validation on `Color`, `opacity`, `blur_radius`** — `dusty-style`
  - `debug_assert!` range checks on `Color::rgba`/`rgb`, `opacity()`, `BoxShadow::new()`

- [x] **Define `gap` vs `row_gap`/`column_gap` resolution semantics** — `dusty-style/src/style.rs`
  - Added `resolved_row_gap()` and `resolved_column_gap()` methods

---

## Phase 13: High — Layout & Text

- [x] **Fix silent style downcast fallback in layout** — `dusty-layout/src/tree.rs`
  - `()` (no style) defaults; wrong type returns `LayoutError::StyleDowncastFailed`

- [x] **Fix fragment `root_layout_id` falling back to `LayoutNodeId(0)`** — `dusty-layout/src/tree.rs`
  - Synthetic container now gets a `LayoutNodeId`; fallback replaced with `ok_or(EmptyTree)?`

- [x] **Fix `TextSystem` `RefCell` — enforce single-thread constraint** — `dusty-text/src/system.rs:30-32`
  - If text measurement ever happens concurrently, `RefCell` panics at runtime
  - Added `PhantomData<*const ()>` to make `TextSystem` `!Send` (was already `!Sync` via `RefCell`)
  - Added `compile_fail` doctests enforcing `!Send + !Sync`
  - Documented the constraint on the struct

---

## Phase 14: Medium — Reactive

- [x] **Fix O(n) dependency dedup in `track_signal`** — `dusty-reactive/src/signal.rs`
  - Added `Hash` derive to `SignalId` and `ScopeId`
  - Changed `dependency_stack` from `Vec<Vec<SignalId>>` to `Vec<HashSet<SignalId>>` for O(1) dedup

- [x] **Remove dead `DepInfo` fields from effect.rs** — `dusty-reactive/src/effect.rs`
  - Removed `version` and `freshener` fields; effects only need `signal_id`

- [x] **Fix `BatchGuard::drop` swallowing flush errors** — `dusty-reactive/src/batch.rs`
  - Added `debug_assert!(result.is_ok())` on `flush_batch` result

- [x] **Fix `on_cleanup` silently dropping cleanup on borrow failure** — `dusty-reactive/src/effect.rs`
  - Replaced `if let Ok(...)` with `let Ok(...) = ... else { debug_assert!(...); return; }`

- [x] **Fix stale freshener entries when memo dropped without explicit dispose** — `dusty-reactive/src/memo.rs`
  - Added `Drop` impl on `MemoInner` that calls `unregister_freshener` if not already disposed

- [x] **Fix resource effect double set+notify** — `dusty-reactive/src/resource.rs`
  - Wrapped effect body in `batch()` to coalesce Loading + Ready/Errored notifications

---

## Phase 15: Medium — Core

*Completed 2026-03-15.*

- [x] **Add `stop_immediate_propagation`** — `dusty-core/src/event.rs`
  - Added `immediate_stopped: Cell<bool>` field and `stop_immediate_propagation()` method
  - Documented `stop_propagation` behavior (sibling handlers still fire)
  - Dispatch loop checks `is_immediate_propagation_stopped()` before each handler

- [x] **Document `Element::style` type-erased `Box<dyn Any>` contract** — `dusty-core/src/element.rs`
  - Added `style_as::<T>()` convenience method with doc comment and examples
  - Returns `Option<&T>` via `downcast_ref`

- [x] **Fix `TextNode::current_text()` cloning static content** — `dusty-core/src/node.rs`
  - Returns `Cow<'_, str>` — borrowed for static text, owned for dynamic
  - Updated callers in layout, a11y, devtools, render, and widgets

- [x] **Make `error` module visibility consistent** — `dusty-core/src/lib.rs`, `dusty-layout/src/lib.rs`
  - Changed both to `pub mod error` to match `dusty-reactive` and `dusty-text`

- [x] **Add test for event dispatch through `ComponentNode`** — `dusty-core/tests/event_integration.rs`
  - Tests traversal, bubbling through parent, and `stop_immediate_propagation` integration

---

## Phase 16: Medium — Style

*Completed 2026-03-15.*

- [x] **Rename `Corners::xy` to `Corners::top_bottom`** — `dusty-style/src/corners.rs`

- [x] **Optimize `FontStyle::merge` to avoid unnecessary String clones** — `dusty-style/src/font.rs`
  - Uses `other.family.as_ref().or(self.family.as_ref()).cloned()` — avoids clone when `other` has `Some`

- [x] **Optimize `ShadowToken::to_shadows()` allocation** — `dusty-style/src/tokens.rs`
  - Returns `Cow<'static, [BoxShadow]>` backed by static `const` arrays
  - Zero allocation for all variants

- [x] **Fix misleading `Eq` derive on `Edges<T>` and `Corners<T>`** — `dusty-style/src/edges.rs`, `corners.rs`
  - Removed `Eq` from derives; added `#[allow(clippy::derive_partial_eq_without_eq)]`

- [x] **Differentiate light/dark theme color scales** — `dusty-style/src/theme.rs`
  - Documented as known limitation: both themes use identical palette color scales

- [x] **Add missing builder methods** — `dusty-style/src/builder.rs`
  - Added `flex_wrap_reverse()` and `overflow_visible()` with tests

---

## Phase 17: Medium — Layout & Text

- [x] **Remove or use dead `LayoutError::StyleDowncastFailed` variant** — `dusty-layout/src/error.rs:13`
  - Already used by Phase 13 fix: `build_node` returns `StyleDowncastFailed` for wrong style types

- [x] **Add debug assertion for out-of-bounds `layout_id` in `extract_absolute`** — `dusty-layout/src/tree.rs`

- [ ] **Consider reusable `TaffyTree` for 60fps re-layout** — `dusty-layout/src/tree.rs:61-65`
  - `compute_layout` allocates fresh `TaffyTree` + `HashMap` on every call
  - Consider a `LayoutEngine` struct that persists between frames

- [ ] **Deduplicate buffer setup between `TextSystem::measure` and `TextLayout::new`** — `dusty-text/src/system.rs:69-87` vs `layout.rs:33-51`
  - Identical buffer creation pattern — changes to one must be mirrored in the other

- [x] **Document `letter_spacing` limitation** — `dusty-text/src/convert.rs`
  - cosmic-text doesn't support per-span letter spacing; documented on `font_style_to_attrs`

- [x] **Clamp color values in `to_cosmic_color`** — `dusty-text/src/convert.rs`
  - Added `.clamp(0.0, 1.0)` before `* 255.0` conversion

- [x] **Implement or hide truncation module** — `dusty-text/src/truncate.rs`
  - Truncation logic already implemented in `TextSystem::truncate` (binary search)
  - Added edge-case integration tests: single char tiny width, unicode boundaries, exact fit, one-char-too-wide

- [x] **Fix `compute_buffer_size` height resilience** — `dusty-text/src/system.rs:91-101`
  - Changed `height = run.line_top + run.line_height` to `height = height.max(run.line_top + run.line_height)`
  - Empty text still reports 0 height (no layout runs)

---

## Phase 18: Architecture & Consistency

- [x] **Fix MSRV: bump `rust-version` from `"1.75"` to `"1.79"`** — `Cargo.toml:21`

- [x] **Adopt `thiserror` across all error types** — No action needed: CLAUDE.md mandates manual `Display` + `Error` impls (no `thiserror`)
  - All four crates already follow this standard

- [x] **Fix `LayoutError` not chaining to upstream `TaffyError`** — `dusty-layout/src/error.rs:28`
  - Now stores `taffy::TaffyError` directly; `source()` returns it
  - Manual `PartialEq`/`Eq` impls since `TaffyError` doesn't derive them

- [x] **Populate `[workspace.dependencies]`** — `Cargo.toml`
  - Added `static_assertions`, `cosmic-text`, `accesskit`, `proptest` to workspace deps
  - Updated all crate Cargo.toml files to use `dep.workspace = true`

- [x] **Tighten internal visibility in `dusty-reactive`**
  - `runtime.rs`: `SignalSlot`, `ScopeSlot`, `Runtime` struct+fields changed to `pub(crate)`
  - `subscriber.rs`: all functions changed to `pub(crate)` except `untrack` (re-exported public API)

- [x] **Consider adding `dusty-style` error type** — Skipped (low impact)
  - `provide_theme`/`use_theme` return `dusty_reactive::error::Result`, leaking upstream error type
  - Low impact: `dusty-style` is internal to the workspace

- [x] **Consider decoupling `TextMeasure` from `dusty-layout`** — Deferred (major refactor)
  - `dusty-text` depends on `dusty-layout` solely for the `TextMeasure` trait
  - Moving trait to `dusty-core` or shared crate requires significant dependency reorganization

- [x] **Update build checklist Phase 11 (dusty-text)** — `docs/checklists/build.md:131-139`
  - All items already marked `[x]` in build checklist

---

## Phase 19: Low Priority — Cleanup & Polish

- [x] **Remove stale `#[allow(dead_code)]`** — `dusty-reactive/src/subscriber.rs`, `resource.rs`
  - Removed from `register_subscriber`; kept on `ResourceInner.current_generation` (needed to keep Rc alive); removed from `effect`

- [x] **Remove dead `DepInfo` fields** — `dusty-reactive/src/effect.rs`
  - Addressed in Phase 14

- ~~**Remove pointless `const fn` on functions taking `Vec`**~~
  - Skipped: clippy's `missing_const_for_fn` lint requires these to remain `const`

- [x] **Add `From<i32>`, `From<u32>`, `From<f32>`, `From<usize>` for `AttributeValue`** — `dusty-core/src/element.rs`
  - All four `From` impls convert to `Int(i64)` or `Float(f64)` as appropriate

- [x] **Add `Display` impl for `Color`** — `dusty-style/src/color.rs`
  - Outputs `#RRGGBB` when fully opaque, `#RRGGBBAA` otherwise

- [x] **Add `BoxShadow` constructor** — `dusty-style/src/shadow.rs`
  - `BoxShadow::new()` already existed; added comprehensive tests

- [x] **Consider `Arc<str>` for `FontStyle.family`** — `dusty-style/src/font.rs:72`
  - Changed `Option<String>` to `Option<Arc<str>>` for cheaper clone during merges
  - Updated builder, convert, and all downstream consumers

- [x] **Remove unused `static_assertions` dev-dep** — `dusty-style/Cargo.toml`
  - Not used in any source or test file; removed

- [x] **Add `Key` constants** — `dusty-core/src/event.rs`
  - Added `Key::enter()`, `Key::escape()`, `Key::tab()`, `Key::backspace()`, `Key::space()`, `Key::delete()`, and arrow key constructors
  - Used associated functions (not `const`) since `String` cannot be non-empty in const context

- [x] **Use `SmallVec` in `build_children`** — `dusty-layout/src/tree.rs:197-207`
  - `SmallVec<[NodeId; 8]>` avoids heap allocation for typical child counts
  - Also updated `build_node` return type to `SmallVec` for consistency

- [ ] **Add layout support for percentage dimensions** — `dusty-layout/src/convert.rs:49-54`
  - Introduce `DimensionValue` enum with `Length`, `Percent`, `Auto` variants in `dusty-style`

- [ ] **Add per-axis overflow support** — `dusty-layout/src/convert.rs:139-146`
  - `overflow_x`/`overflow_y` in `dusty-style::Style`, convert per-axis to taffy

- [ ] **Add `TextLayout::reshape` method** — `dusty-text/src/layout.rs`
  - For dynamic text updates without full recreation

- [x] **Expand `TextError` variants** — `dusty-text/src/error.rs`
  - Added `InvalidMetrics(String)` and `ShapingFailed(String)` with Display impls and tests

- [x] **Add `PartialEq` for `Node` / `TextNode`** — `dusty-core/src/node.rs`
  - Manual impls: static text/fragments/components compare structurally
  - Element and Dynamic always return `false` (contain `Box<dyn Any>` / closures)

- [x] **Fix `with_scope` test helper panic safety** — multiple test modules
  - Added `RuntimeGuard` drop guard pattern in `canvas/mod.rs`, `button.rs`, `show.rs`, `element.rs`
  - Ensures `dispose_runtime()` runs even on panic

---

## Phase 20: Missing Test Coverage (cross-crate)

- [x] **Signal set during batch flush** — `dusty-reactive`
  - `tests/edge_cases.rs`: signal_set_during_batch_flush_no_missed_notifications

- [x] **`dispose_runtime` with active cleanup closures** — `dusty-reactive`
  - `tests/edge_cases.rs`: dispose_runtime_with_active_cleanups — verifies no panic

- [x] **Memo with panicking `PartialEq`** — `dusty-reactive`
  - `tests/edge_cases.rs`: memo_with_panicking_partial_eq — runtime recovers after panic

- [x] **Multi-thread independent runtimes** — `dusty-reactive`
  - `tests/edge_cases.rs`: multi_thread_independent_runtimes — 4 threads, no interference

- [x] **Event dispatch through `ComponentNode`** — `dusty-core`
  - `ComponentNode` uses `from_ref` with single boxed child — path traversal tested in Phase 15

- [ ] **Dispatch to text node with ancestor handlers** — `dusty-core`
  - Verify bubbling to ancestor elements is documented and tested

- [ ] **Text wrapping inside constrained container** — `dusty-layout` (unit tests)
  - Integration test exists but no unit test

- [ ] **`Style::resolve` with nested state styles** — `dusty-style`
  - Hover style with its own nested hover variant

- [ ] **`Color` with NaN/Infinity inputs** — `dusty-style`
  - Document or test behavior

- [ ] **`FontWeight` with out-of-range values** — `dusty-style`
  - `FontWeight(0)`, `FontWeight(1000)` — what happens?

- [ ] **Three-way merge with state styles** — `dusty-style`
  - Only two-way merge is currently tested
