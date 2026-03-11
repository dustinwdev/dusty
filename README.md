# Dusty

Reactive, declarative GUI framework for Rust. Desktop-first.

Inspired by React's component model, Solid.js's fine-grained reactivity, and Tailwind's utility styling.

## Architecture

Layered crate workspace — each layer is a separate crate with clean trait boundaries:

| Crate | Purpose |
|-------|---------|
| `dusty` | Facade — re-exports everything |
| `dusty-reactive` | Signals, memos, effects, resources |
| `dusty-core` | View trait, node types, events |
| `dusty-style` | Utility styling + theme engine |
| `dusty-layout` | Flexbox layout |
| `dusty-widgets` | Built-in widget library |
| `dusty-text` | Text shaping/rendering |
| `dusty-a11y` | Accessibility tree |
| `dusty-render` | Scene graph + GPU rendering |
| `dusty-platform` | Windowing, input, event loop |
| `dusty-macros` | Proc macros (`#[component]`, `col![]`, etc.) |
| `dusty-devtools` | Inspector, profiler |

## Building

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## License

MIT OR Apache-2.0
