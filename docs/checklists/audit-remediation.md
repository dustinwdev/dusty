# Audit Remediation Checklist

Findings from the comprehensive framework audit (April 2026). Work through in priority order; the top section is the "unbreak the framework" path.

Severity: 🔴 critical · 🟠 high · 🟡 medium · 🔵 low

---

## P0 — Unbreak the framework

These are small, surgical fixes that should noticeably restore basic UX. Tackle in order.

- [x] 🔴 **Wrap event dispatch and render in `catch_unwind`** in `crates/dusty-platform/src/runner.rs`. Recent panicking reactive API will tear down the UI thread on any signal misuse (disposed signal read, runtime borrow error). Cheapest fix to prevent cascade failures while addressing #2.
- [x] 🔴 **Fix click/drag state machine** (`crates/dusty-platform/src/runner.rs:243-283`):
  - [ ] ~~Emit `MouseDown`/`MouseUp` events on Press/Release~~ — *deferred to follow-up; requires new event types in `dusty-core`*
  - [x] Add ~4 px movement threshold before promoting hold-and-move to a drag
  - [x] Use release coordinates for `Click`, not press coordinates (`runner.rs:269-275`)
  - [ ] ~~Set `mouse_pressed` flag on plain Click press path~~ — *needs MouseDown/MouseUp events; deferred*
- [x] 🔴 **Hit-test must respect scroll offset and overflow clipping** (`crates/dusty-layout/src/hit_test.rs:60-126`):
  - [x] Propagate accumulated `scroll_offset` down the recursion (mirror `dusty-render/src/tree.rs:286-317`)
  - [x] Skip child recursion when point is outside the (visual-shifted) parent rect — implicit clipping that works for `Hidden`/`Scroll`/`Auto`. Per-overflow-mode behavior is a follow-up.
  - [x] Add tests with non-zero scroll offset and zero-offset baseline
- [x] 🔴 **Fix scroll-offset units in renderer for HiDPI** (`crates/dusty-render/src/tree.rs:286-317, 513-520`). **Audit was a false positive:** the renderer subtracts scroll in logical space *before* `scale_rect` applies `scale_factor`, so HiDPI is correct. Verified with new `scroll_view_translates_child_at_hidpi_2x` test at `scale_factor = 2.0`.
- [x] 🔴 **Translate event coordinates to widget-local in dispatch.** Added `HitTestResult::visual_origin` (layout origin minus accumulated ancestor scroll). `app.rs` dispatch sites now construct widget-local Click/Hover/Drag events. Drag latches the target's visual_origin at Start so Move/End translate against the same origin even after the cursor leaves the widget.
- [x] 🔴 **`ScrollView::on_scroll` must call `ctx.stop_propagation()`** (`crates/dusty-widgets/src/scroll_view.rs:131`). Without it, nested scroll views all scroll in lockstep on a single wheel event.
- [x] 🔴 **First-frame zero-size fallback** (`crates/dusty/src/app.rs:158-186`). On macOS the first `RedrawRequested` can fire before `Resized`; `inner_size()` returns `(0,0)` and layout produces an empty tree. Fall back to `WindowConfig::size()` when `inner_size().width == 0`.
- [x] 🔴 **`Signal::set` thrash mitigation** (`crates/dusty-reactive/src/signal.rs:506-517`). Changing `set`'s default semantics is a breaking API change (existing tests rely on always-notify). Instead, swapped widget call sites to `set_if_changed`: `text_input.rs`, `slider.rs`, `scroll_view.rs`, `checkbox.rs`, `toggle.rs`, `radio.rs`. **Follow-up:** consider adding `T: PartialEq` bound to `set` itself in a future major version.

---

## Reactive Runtime (dusty-reactive)

- [ ] 🔴 **`flush_pending_effects` silently drops the queue after 100 iterations** (`effect.rs:224-243`). Log/panic on exhaustion; surface via wake fn.
- [ ] 🔴 **`propagate_dirty` bypasses batching** (`memo.rs:259-278`). Doesn't check `batch_depth`; downstream effects of dirty memos can fire mid-batch. Push to `pending_batch_subscribers` when `batch_depth > 0`.
- [ ] 🟠 **Effect/Memo Rc cycle leaks** (`effect.rs:129-140`). Subscriber callback holds strong `Rc<EffectInner>` via `state_slot`. Use `Weak` in the callback, or implement `Drop` on `EffectInner`/`MemoInner` that calls dispose.
- [ ] 🟠 **`update_memo_slot` doesn't call `flush_pending_effects` or `invoke_wake_fn`** (`memo.rs:462-498`). Memo-driven updates won't trigger redraw. Mirror `set_and_notify`.
- [ ] 🟠 **Effect re-entrancy guard can swallow distinct writes during execution** (`effect.rs:255-258`). Replace running-flag with a generation counter so writes-during-write always trigger one more re-run.
- [ ] 🟡 **Subscriber free-list reuse leaves stale `SubscriberId`s in signal subscriber sets** (`subscriber.rs:23-40`, `signal.rs:331-378`). GC on notice during notify, or add a stat counter and periodic cleanup.
- [ ] 🟡 **`untrack` panic-restore drops saved state on `RuntimeBorrowError`** (`subscriber.rs:139-159`). Propagate the error or panic with a clearer message; don't silently skip restore.
- [ ] 🔵 **Improve diagnostic on `RuntimeBorrowError`** in `runtime.rs:177-185` — distinguish from `NoRuntime`.
- [ ] 🔵 **`pop_tracking` reallocates Vec from HashSet** (`subscriber.rs:100-112`). Use `into_iter().collect()`.

---

## Hit-Test, Layout, & Overflow (dusty-layout)

- [ ] 🔴 **Plumb scroll-position state into layout.** Taffy's `scroll_offset` and `content_size` are ignored. `extract_absolute` (`tree.rs:316-351`) doesn't subtract scroll offset on descent. Add per-scrollable-element state (signal or sidecar map keyed by `LayoutNodeId`); surface `content_size` on `Rect` so scrollbars can be drawn.
- [ ] 🔴 **No taffy cache reuse — full rebuild every frame** (`tree.rs:124` calls `taffy.clear()`). Maintain a stable `Node`-identity → taffy `NodeId` map; diff and `set_style`/`set_children`/`mark_dirty` only changed subtrees.
- [x] 🔴 **Style is missing `Percent`, `aspect_ratio`, grid `fr`** (`convert.rs:54-59`, `style.rs`). Introduce `Length { Px(f32), Percent(f32), Auto }`; replace bare `Option<f32>` size fields; route through `length_to_dimension` helpers; add `aspect_ratio: Option<f32>` to `Style`. **Landed:** `Length` + `LengthPercent` enums, `aspect_ratio` field, `_pct`/`_auto` builder methods, `MarginValue` retired (unified into `Length`). Grid `fr` **deferred** — requires a full `Display::Grid` rollout, scoped to a separate plan.
- [ ] 🟠 **Single `overflow` value (no per-axis)** (`convert.rs:152`). CSS allows `overflow-x` / `overflow-y` independently. Split into two fields; keep `overflow` as shorthand on builder.
- [ ] 🟠 **`align_content` missing from `Style`.** Multi-line `flex_wrap: Wrap` containers can't align rows.
- [ ] 🟠 **Measure callback collapses `MinContent` and `MaxContent`** (`tree.rs:158-172`). Distinguish the three `AvailableSpace` cases; document `TextMeasure` semantics for `max_width: None`.
- [ ] 🟡 **Measure ignores `known_dimensions.width` when wrapping** (`tree.rs:166-167`). Use `known_dimensions.width.or(max_width)` as wrap constraint.
- [ ] 🟡 **`Display::None` still allocates `LayoutNodeId` and rect** (`tree.rs:223-279`). Skip in tree walk and hit-test (still consume IDs to keep dispatch paths stable, or rework so IDs aren't allocated).
- [ ] 🟡 **Synthetic root for fragments has highest `LayoutNodeId`, not 0** (`tree.rs:142-147`). Document the invariant or assign root id explicitly; downstream code that assumes `LayoutNodeId(0) == root` will break.
- [ ] 🟡 **Expose `content_rect()` on layout result** so consumers can ask where content area starts (border + padding subtraction).
- [ ] 🟡 **`hit_test.rs` ignores z-order from `position: absolute`** — paint-order != child-order once absolute positioning lands. Will need a separate paint-order pass.
- [ ] 🔵 **`PartialEq` for `LayoutError::TaffyError(_)` ignores inner error contents** (`error.rs:37-46`). Two distinct taffy errors compare equal.

---

## Rendering Pipeline (dusty-render)

- [ ] 🔴 **Image draw commands silently dropped** (`renderer.rs:459`). Atlas is `R8Unorm` (incompatible with RGBA). Either add a separate RGBA image pipeline or change the atlas. Wire `ImageCache` decoded data to GPU upload.
- [ ] 🟠 **Glyph cache eviction wipes the entire atlas inline during render** (`glyph_cache.rs:174-190`). Defer eviction to between frames, or implement re-pack so survivors keep their UVs.
- [ ] 🟠 **Glyph cache not invalidated on `ScaleFactorChanged`.** Atlas glyphs rendered at 2x get reused at 1x after dragging across monitors.
- [ ] 🟠 **SDF rounded-box has no test with 4 distinct radii** (`shader.rs:116-136`). Quadrant mapping looks correct but is fragile — add a visual/unit test.
- [ ] 🟡 **Fix `shader.rs` doc comment** (`shader.rs:23`) — says "premultiplied" but pipeline uses straight alpha. Inconsistent doc → future bug.
- [ ] 🟡 **`text_pipeline` and SDF pass share the same uniform buffer** (`renderer.rs:384-393`). Latent bug if uniforms ever diverge.
- [ ] 🟡 **`eprintln!` on font borrow conflict drops text silently in production** (`tree.rs:542, 566`). Route through proper logging.
- [ ] 🔵 **`GradientData.stops: Vec` allocates per-frame** in `DrawPrimitive::Clone` (`primitive.rs:106-114`). Use a fixed-size array.
- [ ] 🔵 **`hash_string` for `ImageId` uses `DefaultHasher`** (run-randomized via SipHash). Once images render, ensure lookup uses `cache.source_to_id()`, not the hash.

---

## Event System (dusty-platform, dusty-core/event)

- [ ] 🔴 **Keyboard events dropped when nothing focused** (`app.rs:338-376`). No top-level fallback → no global hotkeys, no Esc-to-close. Dispatch to root with empty path when `focus_path.is_none()`.
- [ ] 🔴 **No "focusable" concept; focus set on every click**, even empty background (`app.rs:253-298`). Add a `focusable` flag to `Element`; only update focus when target is focusable; clear focus on click outside any focusable.
- [ ] 🟠 **No pointer enter/leave events** — only positional Hover. Track previous hover target; fire `pointer_enter`/`pointer_leave` on transitions.
- [ ] 🟠 **Modifier state never cleared on window unfocus** (`runner.rs:201-204`). Stale Shift/Ctrl after alt-tab. Reset `self.modifiers` on `WindowUnfocused`.
- [ ] 🟠 **Drag dispatch path captured at DragStart can become stale after re-render** (`app.rs:388, 401`). No node-identity check. Add identity validation or re-resolve target on each drag move.
- [ ] 🟡 **`NamedKey::Space` mapped to `" "` while `Key::space()` returns `Key("Space")`** (`key.rs:71` vs `event.rs:104`). Pick one. Anyone matching against `Key::space()` will never hit.
- [ ] 🟡 **Wheel `LineDelta × 40` is a magic constant** (`convert.rs:75`). Make configurable / OS-driven.
- [ ] 🟡 **Right/middle mouse buttons ignored** (`runner.rs:243-247`). No context menu, no middle-click pan.
- [ ] 🟡 **Tab order / Tab-key navigation does not exist.** Currently Tab is just delivered as `KeyDown` to focused widget.
- [ ] 🟡 **Replace stringly-typed key matching with an enum.** Widgets compare against `"Enter"`, `"ArrowLeft"` etc. — allocation-heavy and error-prone.
- [ ] 🔵 **`request_wake` swallows `try_borrow` failures silently** (`runner.rs:28`). Add debug log.
- [ ] 🔵 **`ScrollView::on_scroll` invoked even when `delta_x == delta_y == 0`.**
- [ ] 🔵 **`ScrollView` never clamps upper bound** (`scroll_view.rs:135` TODO). Needs `content_size` from layout first.

---

## Core / View / Widgets (dusty-core, dusty-widgets)

- [ ] 🔴 **No reconciliation exists.** `View::build` is `FnOnce`; the only update path is `Node::Dynamic::current_node()` re-walk. Define and implement a proper reconciler with stable element identity. This is the largest item — scope a dedicated branch. Blocks #2, #3, #4 below.
- [ ] 🔴 **No mount/unmount lifecycle** on `Element`. Effects from removed subtrees outlive their nodes. Add `on_mount`/`on_cleanup` to `ElementBuilder`; create per-element child scopes that dispose on unmount.
- [ ] 🔴 **No element identity / stable ID.** Position-based addressing breaks under sibling insertion. Defeats focus tracking, a11y identity, animation continuity, layout cache reuse.
- [ ] 🔴 **`For::key` accepted but never used** (`for_each.rs:128-142`). Implement keyed list diff with insert/move/remove. Per-item state must survive reorder.
- [ ] 🔴 **`ScrollView` doesn't actually scroll** (`scroll_view.rs:104-166`). Stores offset signal in custom_data, no consumer applied it. Pair with renderer/hit-test scroll-offset fixes above.
- [ ] 🟠 **`Show`/`MatchView` don't dispose prior branch scope.** Switching branches leaks effects/signals from the inactive branch. Wrap each branch in its own disposable scope.
- [ ] 🟠 **`Show`/`MatchView` have no automatic re-evaluation hook** (`show.rs:65` ignores `_cx`). Re-resolution depends on something outside calling `current_node()` again. Wire through `create_effect`.
- [ ] 🟠 **`ErrorBoundary` only catches build-time panics**, not events/effects/async (the common cases in a reactive UI). Catch handler panics in dispatch; provide reset API (currently `child` is `FnOnce` so reset impossible).
- [ ] 🟠 **`TextInput` gaps:**
  - [ ] No IME / composition support (`TextInputEvent` has no preedit state)
  - [ ] No undo/redo
  - [ ] No clipboard (Cmd-C/V/X) — only Cmd-A
  - [ ] Cursor positioning treats `e.x` as a byte offset (`text_input.rs:289`); needs cosmic-text glyph hit-testing
  - [ ] Selection deletion + paste interacts wrongly with `max_length` check (chars-vs-bytes)
  - [ ] `focused_signal` is local; element attr `"focused"` hardcoded `false` (line 236), never reactive
- [ ] 🟠 **`dispatch_event` indexing assumes no fragment flattening** (`event.rs:431-443`). If renderer ever flattens, target paths mis-target. Either forbid fragments from `Dynamic` resolutions, flatten consistently everywhere, or replace positional path with opaque ID.
- [ ] 🟠 **`eprintln!` on event-type mismatch is unbounded in production** (`element.rs:377-381`). Replace with structured warning channel; rate-limit or feature-gate.
- [ ] 🟡 **Tuple-arity ceiling = 12** in `view_seq.rs:43`. Bump to 16 or document.
- [ ] 🟡 **`col!`/`row!` hardcode `gap: 8.0`** with no override. Add an optional gap param.
- [ ] 🟡 **`Node` `PartialEq` is structurally lossy** (`node.rs:68-79`); `Element`/`Dynamic` always compare unequal. Future diff that uses `==` for "no change" is doomed — replace with explicit comparison helper.
- [ ] 🔵 **`extract_panic_message` only handles `&'static str` and `String`** (`error_boundary.rs:82-88`). `panic_any`-style payloads lose info.

---

## App / Cross-Layer Integration (dusty)

- [ ] 🔴 **`compute_layout` runs every frame, on every hover** (`app.rs:179-186, 303-318`). Cache layout; only invalidate on viewport-size change, theme change, or explicit tree-structure change.
- [ ] 🟠 **Memoize `Node::Dynamic` resolution per frame.** Currently resolved 3× per frame (layout, hit-test, render). Each `Text::dynamic`'s `format!` runs three times.
- [ ] 🟠 **Hover events trigger redraw on every cursor move** (`app.rs:303-318`). Dedup: only `request_redraw()` if `hovered_id` actually changed.
- [ ] 🟠 **`ScaleFactorChanged` payload ignored** (`app.rs:241-250`). Pair with glyph-cache scale invalidation above.
- [ ] 🟡 **Resize doesn't trigger immediate re-layout** — relies on follow-up `RedrawRequested`. Hover events between Resize and redraw use stale layout. Re-layout synchronously on Resized.

---

## Process / Hygiene

- [ ] 🟡 Replace ad-hoc `eprintln!` calls across the codebase with a proper logging facade (`log` or `tracing`). Locations: `glyph_cache.rs`, `tree.rs:542,566`, `element.rs:377-381`, others.
- [ ] 🟡 Add an integration test that boots a window with a simple counter app and verifies first-frame paint, click handling, and reactive update — would catch most P0 regressions.
- [ ] 🟡 Document logical-vs-physical pixel boundaries explicitly. Add a coordinate-space type (`Logical<T>`, `Physical<T>` newtypes) at API boundaries between layout, hit-test, and render.

---

## What's Working Well (do not regress)

- Crate layering and dependency flow are clean.
- `dusty-core::event::dispatch_event` correctly handles bubble/capture and `Node::Dynamic` walking.
- `dusty-layout::tree::extract_absolute` accumulates positions correctly (no double-counting of borders/padding).
- `Show`/`Match`/`For` API surface is well-shaped — needs implementation behind it, not redesign.
- Glyph atlas upload is correctly gated on `is_dirty()`.
- `IntoEventHandler` trait dispatch (recent refactor) is clean.
- Test coverage is broad — most subsystems will catch regressions once underlying bugs are fixed.
