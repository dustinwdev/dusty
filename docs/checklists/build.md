# Dusty Master Build Checklist

Each phase is scoped to complete in a single focused session.
Create per-phase design notes in `docs/design/` as needed.

---

## Phase 1: Workspace & Tooling

- [x] `git init`, `.gitignore`
- [x] Cargo workspace with all crate stubs (lib.rs only)
- [x] `rustfmt.toml`, `clippy.toml` / workspace lint config
- [x] Verify `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace` all pass on empty crates

---

## Phase 2: Signals

Crate: `dusty-reactive`

- [ ] `Signal<T>` — create, get, set, update
- [ ] `ReadSignal<T>` / `WriteSignal<T>` split access
- [ ] Subscriber tracking — signals know their dependents
- [ ] Notification — changing a signal notifies subscribers
- [ ] `SignalGuard` or similar for batched reads without cloning
- [ ] Tests: create, read, write, update closure, multiple subscribers, drop cleanup

---

## Phase 3: Memos & Dependency Tracking

Crate: `dusty-reactive`

- [ ] `Memo<T>` — cached derived computation
- [ ] Auto-tracking — memo records which signals it reads during evaluation
- [ ] Lazy re-evaluation — only recompute when a dependency changed AND value is read
- [ ] Diamond dependency handling — memo depending on two signals that share a source
- [ ] Tests: basic derivation, caching (doesn't recompute without change), diamond, chained memos

---

## Phase 4: Effects & Scopes

Crate: `dusty-reactive`

- [ ] `Effect` — side effect that re-runs when dependencies change
- [ ] Auto-tracking for effects (same mechanism as memos)
- [ ] `Scope` — arena-based ownership for reactive primitives
- [ ] Disposal — dropping a scope cleans up all signals/memos/effects within it
- [ ] Nested scopes — child scope disposal doesn't affect parent
- [ ] Tests: effect runs on change, effect cleanup, scope disposal, nested scopes

---

## Phase 5: Batching & Resources

Crate: `dusty-reactive`

- [ ] `batch()` — coalesce multiple signal writes, notify once
- [ ] `Resource<T>` — async data that integrates with signals
- [ ] Resource states: loading, ready, error
- [ ] Resource re-fetches when source signal changes
- [ ] `untrack()` — read a signal without subscribing
- [ ] Tests: batch coalesces notifications, resource lifecycle, untrack

---

## Phase 6: View Trait & Node Tree

Crate: `dusty-core`

- [ ] `View` trait — core abstraction every renderable implements
- [ ] `Node` enum — text, element, component, fragment
- [ ] `Element` — tag/type, props, style, children, event handlers
- [ ] `ViewSeq` trait — heterogeneous collections of views (tuples, Vec, Option)
- [ ] Tree construction helpers
- [ ] Tests: build simple trees, ViewSeq flattening, Option<View> renders or skips

---

## Phase 7: Event System

Crate: `dusty-core`

- [ ] Core event types: Click, Hover, KeyDown, KeyUp, Focus, Blur, Scroll, TextInput
- [ ] `EventHandler<E>` type — type-safe callback wrapper
- [ ] Event propagation model: bubble by default, stop propagation
- [ ] `on_click()`, `on_hover()`, etc. — builder methods on elements
- [ ] Tests: handler invocation, propagation stops, event data correctness

---

## Phase 8: Style Types & Design Tokens

Crate: `dusty-style`

- [ ] `Style` struct — all style properties (padding, margin, bg, fg, border, radius, shadow, font, etc.)
- [ ] Style merge/cascade — later styles override earlier
- [ ] Design tokens: `ColorScale` (50–950 per hue), `SpacingScale`, `RadiusScale`, `ShadowScale`
- [ ] Default palette (Tailwind-inspired color system)
- [ ] Tests: style merge precedence, token value correctness

---

## Phase 9: Utility Methods & Theming

Crate: `dusty-style`

- [ ] Utility builder methods: `.p()`, `.px()`, `.py()`, `.m()`, `.bg_blue()`, `.text_white()`, `.rounded_md()`, `.shadow_lg()`, `.font_bold()`, etc.
- [ ] State modifiers: `.hover()`, `.focus()`, `.active()`, `.disabled()`
- [ ] Conditional: `.when(bool, |s| s.foo())`, `.apply(fn)`
- [ ] `Theme` struct — swappable token sets
- [ ] Theme propagation via context
- [ ] Tests: method chaining produces correct styles, hover/conditional, theme override

---

## Phase 10: Layout Engine

Crate: `dusty-layout`

- [ ] Taffy integration — convert Dusty styles to taffy styles
- [ ] Layout computation: given a node tree + styles → position/size for each node
- [ ] Flexbox: row, column, wrap, gap, align-items, justify-content
- [ ] Sizing: fixed, percentage, min/max, flex-grow/shrink
- [ ] Tests: row layout, column layout, nested flex, gap, alignment, wrapping

---

## Phase 11: Text Rendering

Crate: `dusty-text`

- [ ] cosmic-text integration — font database, shaping, layout
- [ ] `TextLayout` — measure text given font/size/constraints
- [ ] Line wrapping, truncation with ellipsis
- [ ] Rich text spans (bold, italic, color per-range)
- [ ] Tests: measurement accuracy, wrapping behavior, rich text spans

---

## Phase 12: Platform — Windowing & Input

Crate: `dusty-platform`

- [ ] winit integration — create window, run event loop
- [ ] Translate winit events → Dusty events (keyboard, mouse, resize, close)
- [ ] Window config: title, size, min/max size, resizable, decorations
- [ ] DPI/scale factor handling
- [ ] Clipboard read/write
- [ ] Tests: event translation, window config, scale factor math

---

## Phase 13: Render — GPU Pipeline

Crate: `dusty-render`

- [ ] wgpu setup: instance, adapter, device, surface, swap chain
- [ ] Render primitives: filled rect, rounded rect, bordered rect
- [ ] Color rendering, gradient support
- [ ] Shadow rendering
- [ ] Scissor/clipping for overflow
- [ ] Tests: primitive output verification, clipping correctness

---

## Phase 14: Render — Text & Images

Crate: `dusty-render`

- [ ] Text rasterization pipeline: cosmic-text glyphs → texture atlas → GPU quads
- [ ] Glyph cache — atlas management, eviction
- [ ] Image rendering — decode, upload to GPU texture, draw
- [ ] Render tree: walk node tree → emit draw commands
- [ ] Tests: atlas allocation, cache eviction, render tree traversal order

---

## Phase 15: Accessibility

Crate: `dusty-a11y`

- [ ] accesskit integration — build accessibility tree from Dusty node tree
- [ ] Role mapping: button → Button, text → StaticText, input → TextField, etc.
- [ ] Labels, descriptions, states (focused, disabled, checked)
- [ ] Live regions for dynamic content
- [ ] Tests: tree generation correctness, role mapping, state updates

---

## Phase 16: Core Widgets — Display

Crate: `dusty-widgets`

- [ ] `Text` — static and reactive text display
- [ ] `Image` — image display with sizing modes (cover, contain, fill)
- [ ] `Divider` — horizontal/vertical separator
- [ ] `Spacer` — flexible space
- [ ] Tests: text reactivity, image sizing, divider orientation

---

## Phase 17: Core Widgets — Interactive

Crate: `dusty-widgets`

- [ ] `Button` — click handling, disabled state, variants
- [ ] `TextInput` — text entry, cursor, selection, placeholder
- [ ] `Checkbox`, `Radio`, `Toggle` — boolean/choice inputs
- [ ] `Slider` — range input
- [ ] Tests: button click fires handler, input state management, checkbox toggle

---

## Phase 18: Core Widgets — Containers

Crate: `dusty-widgets`

- [ ] `ScrollView` — scrollable content area, scroll bars
- [ ] `For` — keyed list reconciliation
- [ ] `Show` / `Match` — conditional rendering
- [ ] `ErrorBoundary` — catch component errors
- [ ] `Suspense` — async loading fallback
- [ ] Tests: scroll offset, keyed list diffing (add/remove/reorder), Show/Match toggles

---

## Phase 19: Proc Macros

Crate: `dusty-macros`

- [ ] `#[component]` — generate prop builder from function signature
- [ ] `#[prop(default)]`, `#[prop(optional)]`, `#[prop(into)]` attributes
- [ ] `col![]`, `row![]` — layout container macros
- [ ] `text!()`, `button!()` — widget construction macros
- [ ] Tests: macro expansion correctness, compile-fail tests for bad usage

---

## Phase 20: Facade & App Builder

Crate: `dusty`

- [ ] Re-export all public APIs
- [ ] `prelude` module — common imports
- [ ] `dusty::app()` builder — window config, theme, root component, run
- [ ] Integration test: minimal app compiles and boots

---

## Phase 21: Devtools

Crate: `dusty-devtools`

- [ ] Element inspector — overlay showing node boundaries, styles, tree
- [ ] Performance profiler — frame times, re-render counts per signal
- [ ] Accessibility auditor — flag missing labels, roles
- [ ] Feature-gated behind `devtools` cargo feature

---

## Phase 22: Examples & Validation

- [ ] Counter — minimal signal usage
- [ ] Todo app — list, input, state management
- [ ] Theme showcase — demonstrate theming and design tokens
- [ ] Form — inputs, validation, submission
- [ ] Dashboard — complex layout, multiple components, async data
- [ ] Ensure all examples pass clippy, fmt, and run correctly
