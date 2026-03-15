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

- [x] `Signal<T>` — create, get, set, update
- [x] `ReadSignal<T>` / `WriteSignal<T>` split access
- [x] Subscriber tracking — signals know their dependents
- [x] Notification — changing a signal notifies subscribers
- [x] `with()` for zero-clone ref access (replaces `SignalGuard`)
- [x] Tests: create, read, write, update closure, multiple subscribers, drop cleanup

---

## Phase 3: Memos & Dependency Tracking

Crate: `dusty-reactive`

- [x] `Memo<T>` — cached derived computation
- [x] Auto-tracking — memo records which signals it reads during evaluation
- [x] Lazy re-evaluation — only recompute when a dependency changed AND value is read
- [x] Diamond dependency handling — memo depending on two signals that share a source
- [x] Tests: basic derivation, caching (doesn't recompute without change), diamond, chained memos

---

## Phase 4: Effects & Scopes

Crate: `dusty-reactive`

- [x] `Effect` — side effect that re-runs when dependencies change
- [x] Auto-tracking for effects (same mechanism as memos)
- [x] `Scope` — arena-based ownership for reactive primitives
- [x] Disposal — dropping a scope cleans up all signals/memos/effects within it
- [x] Nested scopes — child scope disposal doesn't affect parent
- [x] Tests: effect runs on change, effect cleanup, scope disposal, nested scopes

---

## Phase 5: Batching & Resources

Crate: `dusty-reactive`

- [x] `batch()` — coalesce multiple signal writes, notify once
- [x] `Resource<T>` — async data that integrates with signals
- [x] Resource states: loading, ready, error
- [x] Resource re-fetches when source signal changes
- [x] `untrack()` — read a signal without subscribing
- [x] Tests: batch coalesces notifications, resource lifecycle, untrack

---

## Phase 6: View Trait & Node Tree

Crate: `dusty-core`

- [x] `View` trait — core abstraction every renderable implements
- [x] `Node` enum — text, element, component, fragment
- [x] `Element` — tag/type, props, style, children, event handlers
- [x] `ViewSeq` trait — heterogeneous collections of views (tuples, Vec, Option)
- [x] Tree construction helpers
- [x] Tests: build simple trees, ViewSeq flattening, Option<View> renders or skips

---

## Phase 7: Event System

Crate: `dusty-core`

- [x] Core event types: Click, Hover, KeyDown, KeyUp, Focus, Blur, Scroll, TextInput
- [x] `EventHandler<E>` type — type-safe callback wrapper
- [x] Event propagation model: bubble by default, stop propagation
- [x] `on_click()`, `on_hover()`, etc. — builder methods on elements
- [x] Tests: handler invocation, propagation stops, event data correctness

---

## Phase 8: Style Types & Design Tokens

Crate: `dusty-style`

- [x] `Style` struct — all style properties (padding, margin, bg, fg, border, radius, shadow, font, etc.)
- [x] Style merge/cascade — later styles override earlier
- [x] Design tokens: `ColorScale` (50–950 per hue), `SpacingScale`, `RadiusScale`, `ShadowScale`
- [x] Default palette (Tailwind-inspired color system)
- [x] Tests: style merge precedence, token value correctness

---

## Phase 9: Utility Methods & Theming

Crate: `dusty-style`

- [x] Utility builder methods: `.p()`, `.px()`, `.py()`, `.m()`, `.bg_blue()`, `.text_white()`, `.rounded_md()`, `.shadow_lg()`, `.font_bold()`, etc.
- [x] State modifiers: `.hover()`, `.focus()`, `.active()`, `.disabled()`
- [x] Conditional: `.when(bool, |s| s.foo())`, `.apply(fn)`
- [x] `Theme` struct — swappable token sets
- [x] Theme propagation via context
- [x] Tests: method chaining produces correct styles, hover/conditional, theme override

---

## Phase 10: Layout Engine

Crate: `dusty-layout`

- [x] Taffy integration — convert Dusty styles to taffy styles
- [x] Layout computation: given a node tree + styles → position/size for each node
- [x] Flexbox: row, column, wrap, gap, align-items, justify-content
- [x] Sizing: fixed, percentage, min/max, flex-grow/shrink
- [x] Tests: row layout, column layout, nested flex, gap, alignment, wrapping

---

## Phase 11: Text Rendering

Crate: `dusty-text`

- [x] cosmic-text integration — font database, shaping, layout
- [x] `TextLayout` — measure text given font/size/constraints
- [x] Line wrapping, truncation with ellipsis
- [x] Rich text spans (bold, italic, color per-range)
- [x] Tests: measurement accuracy, wrapping behavior, rich text spans

---

## Phase 12: Platform — Windowing & Input

Crate: `dusty-platform`

- [x] winit integration — create window, run event loop
- [x] Translate winit events → Dusty events (keyboard, mouse, resize, close)
- [x] Window config: title, size, min/max size, resizable, decorations
- [x] DPI/scale factor handling
- [x] Clipboard read/write
- [x] Tests: event translation, window config, scale factor math

---

## Phase 13: Render — GPU Pipeline

Crate: `dusty-render`

- [x] wgpu setup: instance, adapter, device, surface, swap chain
- [x] Render primitives: filled rect, rounded rect, bordered rect
- [x] Color rendering, gradient support
- [x] Shadow rendering
- [x] Scissor/clipping for overflow
- [x] Tests: primitive output verification, clipping correctness

---

## Phase 14: Render — Text & Images

Crate: `dusty-render`

- [x] Text rasterization pipeline: cosmic-text glyphs → texture atlas → GPU quads
- [x] Glyph cache — atlas management, eviction
- [x] Image rendering — decode, upload to GPU texture, draw
- [x] Render tree: walk node tree → emit draw commands
- [x] Tests: atlas allocation, cache eviction, render tree traversal order

---

## Phase 15: Accessibility

Crate: `dusty-a11y`

- [x] accesskit integration — build accessibility tree from Dusty node tree
- [x] Role mapping: button → Button, text → StaticText, input → TextField, etc.
- [x] Labels, descriptions, states (focused, disabled, checked)
- [x] Live regions for dynamic content
- [x] Tests: tree generation correctness, role mapping, state updates

---

## Phase 16: Core Widgets — Display

Crate: `dusty-widgets`

- [x] `Text` — static and reactive text display
- [x] `Image` — image display with sizing modes (cover, contain, fill)
- [x] `Divider` — horizontal/vertical separator
- [x] `Spacer` — flexible space
- [x] `Canvas` — 2D drawing escape hatch (Frame API: paths, fills, strokes, transforms, text, images)
- [x] `Canvas` reactive integration — draw closure reads signals, caches geometry when deps unchanged
- [x] `Canvas` input — optional event handling for interactive canvases (click, drag, hover within bounds)
- [x] Tests: text reactivity, image sizing, divider orientation, canvas draw + reactive redraw + input

---

## Phase 17: Core Widgets — Interactive

Crate: `dusty-widgets`

- [x] `Button` — click handling, disabled state, variants
- [x] `TextInput` — text entry, cursor, selection, placeholder
- [x] `Checkbox`, `Radio`, `Toggle` — boolean/choice inputs
- [x] `Slider` — range input
- [x] Tests: button click fires handler, input state management, checkbox toggle

---

## Phase 18: Core Widgets — Containers

Crate: `dusty-widgets`

- [x] `ScrollView` — scrollable content area, scroll bars
- [x] `For` — keyed list reconciliation
- [x] `Show` / `Match` — conditional rendering
- [x] `ErrorBoundary` — catch component errors
- [x] `Suspense` — async loading fallback
- [x] Tests: scroll offset, keyed list diffing (add/remove/reorder), Show/Match toggles

---

## Phase 19: Proc Macros

Crate: `dusty-macros`

- [x] `#[component]` — generate prop builder from function signature
- [x] `#[prop(default)]`, `#[prop(optional)]`, `#[prop(into)]` attributes
- [x] `col![]`, `row![]` — layout container macros
- [x] `text!()`, `button!()` — widget construction macros
- [x] Tests: macro expansion correctness, compile-fail tests for bad usage

---

## Phase 20: Facade & App Builder

Crate: `dusty`

- [x] Re-export all public APIs
- [x] `prelude` module — common imports
- [x] `dusty::app()` builder — window config, theme, root component, run
- [x] Integration test: minimal app compiles and boots

---

## Phase 21: Devtools

Crate: `dusty-devtools`

- [x] Element inspector — overlay showing node boundaries, styles, tree
- [x] Performance profiler — frame times, re-render counts per signal
- [x] Accessibility auditor — flag missing labels, roles
- [x] Feature-gated behind `devtools` cargo feature

---

## Phase 22: Examples & Validation

- [x] Counter — minimal signal usage
- [x] Todo app — list, input, state management
- [x] Theme showcase — demonstrate theming and design tokens
- [x] Form — inputs, validation, submission
- [x] Dashboard — complex layout, multiple components, async data
- [x] Ensure all examples pass clippy, fmt, and run correctly
