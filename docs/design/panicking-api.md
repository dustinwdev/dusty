# Panicking vs Result-Returning Reactive API

## Status
Implemented

## Context

Every method on `Signal<T>`, `Memo<T>`, `Effect`, and `Resource<T>` currently returns `Result<T, ReactiveError>`. This means every read, write, and creation call requires `.unwrap()` or `?` propagation at the call site:

```rust
let count = create_signal(0)?;
let doubled = create_memo(move || count.get().unwrap() * 2)?;

create_effect(move || {
    let val = count.get().unwrap();
    log!("{val}");
})?;

count.set(5)?;
```

The `.unwrap()` calls inside closures are particularly painful because `?` cannot propagate out of the closure boundary, so users are forced into `.unwrap()` anyway. This pattern appears throughout the codebase (every memo computation, every effect body) and will dominate application code.

The question is whether to switch to a panicking-by-default API where `get()` panics on error and a separate `try_get()` returns `Result`.

## Error Landscape

The `ReactiveError` variants that the current API can produce:

| Variant | When | Bug or expected? |
|---------|------|-------------------|
| `NoRuntime` | Runtime not initialized | Programming bug |
| `SignalDisposed` | Signal used after disposal | Programming bug |
| `MemoDisposed` | Memo used after disposal | Programming bug |
| `EffectDisposed` | Effect used after disposal | Programming bug |
| `ResourceDisposed` | Resource used after disposal | Programming bug |
| `TypeMismatch` | Internal type confusion | Internal bug (unreachable via safe API) |
| `RuntimeBorrowError` | Re-entrant borrow | Programming bug |

Every single error variant represents a programming mistake, not a recoverable runtime condition. No user code should ever need to match on these errors and take a different code path. The correct response in all cases is to fix the code.

## Options

### Option A: Keep Result-returning API (status quo)

The API stays as-is: `get() -> Result<T>`, `set(val) -> Result<()>`, etc.

**Pros:**
- Explicit about fallibility; the type system forces acknowledgement of error cases
- No panics in library code, which aligns with the project's "no `unwrap()` in library code" standard
- Users who want to handle errors gracefully (e.g., logging + continuing instead of crashing) can do so
- Easier to test error conditions in unit tests (assert on `Err` variants rather than `#[should_panic]`)

**Cons:**
- Pervasive `.unwrap()` noise — every `get()` inside a memo or effect closure requires it, since `?` cannot escape closure boundaries
- The errors are never meaningfully handled: they represent programming bugs, not recoverable conditions. Wrapping them in `Result` implies recoverability that does not exist.
- Ergonomic cost compounds: a simple counter example needs 5+ unwrap/? calls
- Discourages adoption — the "hello world" experience feels heavy compared to Leptos, Dioxus, or Sycamore
- Memo computations (`create_memo(move || count.get().unwrap() * 2)`) are the most common pattern, and `.unwrap()` adds visual noise to what should be a clean derived expression
- Test code itself is littered with `.unwrap()` (see existing tests), demonstrating that even the framework authors never handle the `Err` path

### Option B: Panicking default API with `try_` fallible variants

Primary methods (`get`, `set`, `update`, `with`, `with_untracked`) panic on error. Fallible variants (`try_get`, `try_set`, `try_update`, `try_with`, `try_with_untracked`) return `Result`. Creation functions (`create_signal`, `create_memo`, `create_effect`) also get panicking defaults with `try_create_*` variants.

```rust
// Clean everyday usage
let count = create_signal(0);
let doubled = create_memo(move || count.get() * 2);

create_effect(move || {
    let val = count.get();
    log!("{val}");
});

count.set(5);

// Rare: when you genuinely need fallibility
if let Ok(val) = count.try_get() { ... }
```

**Pros:**
- Dramatically cleaner ergonomics, especially inside closures where `?` is unavailable
- Matches the semantic reality: these errors are programming bugs, and panicking on bugs is idiomatic Rust (like `Vec::index`, `RefCell::borrow`)
- Consistent with mature frameworks: Leptos 0.5+/0.6 panics by default, Solid.js throws, React throws. These projects made the same tradeoff for the same reasons.
- `try_*` variants remain available for the rare cases that need them (hot-reloading, plugin systems, testing)
- Reduces framework adoption friction — the "hello world" reads like pseudocode
- Closure bodies become pure expressions: `create_memo(move || a.get() + b.get())`

**Cons:**
- Panics are harder to debug in production if the runtime is somehow not initialized (though this is always a setup bug)
- Violates the project's current "no `unwrap()` in library code" rule — but the rule's intent is to prevent masking recoverable errors, which these are not
- Two API surfaces to maintain (`get`/`try_get`, `set`/`try_set`, etc.), though the panicking version is a thin wrapper
- Panic messages must be high-quality to compensate for the loss of type-level error handling (include signal ID, error variant, and a hint about what went wrong)

### Option C: Hybrid — panicking `get`/`set` only, Result for creation

Creation functions (`create_signal`, `create_memo`, `create_effect`) return `Result` because they are called at setup time where `?` propagation works naturally. Access methods (`get`, `set`, `update`, `with`) panic because they are called inside closures where `?` cannot propagate.

```rust
// Creation: Result (? works here, called in setup code)
let count = create_signal(0)?;
let doubled = create_memo(move || count.get() * 2)?;

// Access: panics (called inside closures, no ? available)
count.set(5);
```

**Pros:**
- Best of both worlds: `?` at the top level where it works, panic-free creation, clean closure bodies
- Nudges users toward correct initialization (check that the runtime exists) while keeping hot-path code clean
- Smaller API surface increase than full Option B (no `try_create_*` needed)

**Cons:**
- Inconsistent: some methods panic, others return `Result`, creating a mixed mental model
- `NoRuntime` on creation is still a programming bug, so even the `Result` on creation is arguably unnecessary
- Sycamore tried this hybrid approach and eventually moved fully to panicking in 0.9

## Recommendation

**Option B: Panicking default API with `try_` fallible variants.**

The decisive factor is the closure problem. In a reactive framework, the majority of signal reads happen inside `create_memo(move || ...)` and `create_effect(move || ...)` closures. The `?` operator cannot propagate errors out of these closures, so users are forced into `.unwrap()` regardless of the API design. The `Result` return type provides zero practical benefit in these contexts — it just adds noise.

Every error variant in `ReactiveError` represents a programming bug (no runtime, use-after-dispose, re-entrant borrow). Rust's convention for programming bugs is to panic, not to return `Result`. `Vec::index` panics on out-of-bounds. `RefCell::borrow` panics on re-entrancy. `Signal::get` should panic when the signal is disposed, for the same reason.

The `try_*` variants preserve the escape hatch for the few legitimate uses (testing error paths, plugin sandboxing, development tools) without taxing the common case.

Implementation plan:
1. Add `try_get`, `try_set`, `try_update`, `try_with`, `try_with_untracked` to all handle types, identical to today's `get`/`set`/etc.
2. Change `get`, `set`, `update`, `with`, `with_untracked` to unwrap internally with high-quality panic messages (include handle type, ID, and error variant).
3. Add `try_create_signal`, `try_create_memo`, `try_create_effect`, `try_create_resource`. Make the non-`try_` versions panicking.
4. Update all doc examples to use the panicking API.
5. Update tests: most switch to panicking API, dedicated error-path tests use `try_*`.
