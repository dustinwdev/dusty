# Dusty Architecture

## Core Concept

Dusty is a reactive GUI framework where:

1. **Components** are functions that take `Scope` + props, return `impl View`
2. **Signals** provide fine-grained reactivity — no virtual DOM diffing
3. **Views** build a retained widget tree once; signals surgically update nodes
4. **Styling** uses type-safe Tailwind-inspired utility methods

## Rendering Pipeline

```
Signal change
  → notify subscribers (O(1) lookup)
  → update affected nodes in retained tree
  → mark dirty regions
  → re-layout dirty subtrees (taffy)
  → re-render dirty regions (wgpu)
```

Full tree diff is never needed. Signals track their dependents directly.

## Component Model

- Components are functions: `fn(Scope, props...) -> impl View`
- `#[component]` macro generates prop builder
- Children via `Children` type (type-erased view collection)
- Named slots via explicit `impl View` props
- Context for dependency injection through the tree
- Custom hooks: functions that take `Scope`, return reactive state

## Reactive Primitives

| Primitive | Purpose | React Equivalent |
|-----------|---------|-----------------|
| `Signal<T>` | Read/write reactive state | `useState` |
| `Memo<T>` | Cached derived value | `useMemo` |
| `Effect` | Side effect on change | `useEffect` |
| `Resource<T>` | Async data fetching | React Query |

Key difference from React: dependency tracking is automatic. No dependency arrays.
Signals know their subscribers. Updates are O(subscribers), not O(tree size).

## Styling System

Type-safe utility methods that mirror Tailwind's API:

```rust
text!("Hello")
    .px(4).py(2)
    .bg_blue(500)
    .rounded_md()
    .hover(|s| s.bg_blue(600))
```

Design token system: colors, spacing, radii, shadows, typography are all
configurable via `Theme`. Ships with sensible defaults.

## Key Design Decisions

1. **Signals over Elm-style messages** — less boilerplate, fine-grained updates
2. **Builder methods over DSL/markup** — works with rust-analyzer, no new syntax
3. **wgpu over native widgets** — full control, consistent cross-platform
4. **taffy for layout** — proven flexbox impl, used by Dioxus and Bevy
5. **cosmic-text for text** — handles shaping, bidi, wrapping; battle-tested
6. **accesskit from day one** — retrofitting accessibility is nearly impossible
