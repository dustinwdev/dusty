# Copy Handles for Memo and Effect

## Status
Proposed

## Context

`Signal<T>`, `ReadSignal<T>`, and `WriteSignal<T>` are all `Copy`. They achieve this by being thin handles — a generational index (`SignalId`) plus `PhantomData` markers — with the actual data living in the thread-local runtime's arena (`Vec<SignalSlot>`).

`Memo<T>` and `Effect` are `Clone` but not `Copy`. They wrap `Rc<MemoInner<T>>` and `Rc<EffectInner>` respectively, which hold the computation closure, dirty flag, dependency list, cleanup functions, and subscriber ID. The `Rc` provides automatic cleanup via `Drop`: when the last handle is dropped, the inner state is freed and the freshener is unregistered.

This inconsistency means passing a `Memo` into a closure requires either `.clone()` or moving the original, while a `Signal` can be freely copied:

```rust
let count = create_signal(0);     // Copy — just works
let doubled = create_memo(move || count.get() * 2);  // moved into closure

// Later: doubled.clone() needed if used in multiple closures
let a = create_memo(move || doubled.get() + 1);      // moves doubled
let b = create_memo(move || doubled.get() * 10);      // ERROR: doubled already moved
// Fix: let doubled2 = doubled.clone(); before the second closure
```

The question is whether `Memo<T>` and `Effect` should also be `Copy`, like `Signal<T>`.

## Current Internal Architecture

**Signal<T>** (Copy):
- Handle: `SignalId { index: usize, generation: u64 }` + two `PhantomData` fields
- Storage: `Runtime.signals: Vec<SignalSlot>` (arena with free list)
- Cleanup: explicit `dispose_signal()` or scope-based disposal via registered disposers
- Size: 24 bytes (on 64-bit)

**Memo<T>** (Clone, not Copy):
- Handle: `SignalId` + `Rc<MemoInner<T>>` + `PhantomData`
- `MemoInner<T>` contains: `Box<dyn Fn() -> T>`, `Rc<Cell<bool>>` (dirty flag), `RefCell<SmallVec<[DepInfo; 4]>>` (deps), `SubscriberId`, `SignalId`, `Cell<bool>` (disposed)
- The memo's cached value is stored in the signal arena (as `Option<T>` in a `SignalSlot`)
- Freshener closures hold `Rc::downgrade(&state)` — they use `Weak` to detect when the memo is dropped
- `Drop for MemoInner<T>` unregisters the freshener if not explicitly disposed
- Scope disposal also registers a disposer that calls `dispose_memo()`

**Effect** (Clone, not Copy):
- Handle: `Rc<EffectInner>` + `PhantomData`
- `EffectInner` contains: `Box<dyn Fn()>`, `SubscriberId`, `Rc<Cell<bool>>` (dirty), `RefCell<Vec<DepInfo>>` (deps), `RefCell<Vec<Box<dyn FnOnce()>>>` (cleanups), `Cell<bool>` (disposed/running)
- The subscriber callback captures `Rc<RefCell<Option<Rc<EffectInner>>>>` for deferred re-execution
- Scope disposal registers a disposer that calls `dispose_effect()`

## Options

### Option A: Keep Memo and Effect as Clone-only (status quo)

`Memo<T>` and `Effect` remain backed by `Rc` and are `Clone` but not `Copy`.

**Pros:**
- `Rc`-based Drop provides automatic cleanup: when all handles are dropped, the freshener is unregistered and resources are freed, even if the user forgets explicit disposal
- Simpler internal implementation: the computation closure, dirty flag, dep list, and cleanup fns all live in `MemoInner`/`EffectInner` behind the `Rc`, with no need for an additional arena or slab
- `Weak` references in freshener closures naturally detect when the memo has been dropped, avoiding dangling callbacks
- The subscriber callback for effects captures `Rc<RefCell<Option<Rc<EffectInner>>>>`, giving it a direct (non-lookup) path to queue the effect for re-execution
- No additional arena management complexity (generation checks, free lists) beyond what signals already have
- In practice, most memos and effects are created once and captured by a single closure — the `Clone` cost is rarely paid

**Cons:**
- Inconsistent with `Signal<T>` — users must remember that signals are `Copy` but memos are not
- Passing memos into multiple closures requires explicit `.clone()`, adding boilerplate
- `Rc` is pointer-sized (8 bytes) plus the reference count overhead; generational indices are smaller and cache-friendlier in bulk
- The `Rc`-based approach creates a web of shared ownership (`Rc`, `Weak`, `Rc<RefCell<Option<Rc<...>>>>`) that is harder to reason about than flat arena storage
- `Rc` prevents the runtime from having a single unified arena for all reactive nodes

### Option B: Make Memo and Effect Copy via generational arena storage

Move all `MemoInner`/`EffectInner` state into the runtime arena (or a parallel arena), making `Memo<T>` and `Effect` thin `Copy` handles (like `Signal<T>`).

The handle would be something like:
```rust
pub struct Memo<T: 'static> {
    id: MemoId,              // generational index into memo arena
    signal_id: SignalId,     // index of cached-value slot in signal arena
    _marker: PhantomData<fn() -> T>,
    _not_send: PhantomData<*const ()>,
}

pub struct Effect {
    id: EffectId,            // generational index into effect arena
    _not_send: PhantomData<*const ()>,
}
```

**Pros:**
- Fully consistent API: `Signal`, `Memo`, and `Effect` are all `Copy` handles
- No `.clone()` boilerplate when capturing in multiple closures
- Enables a unified arena for all reactive nodes, improving cache locality and enabling bulk operations (dispose all, iterate all)
- Handle size is small and fixed (two `usize` + one `u64` generation), good for embedding in data structures
- Eliminates the `Rc`/`Weak`/`RefCell` reference web in favor of flat indexed storage

**Cons:**
- **Loss of automatic Drop cleanup.** `Copy` types cannot implement `Drop`. This means memos and effects would leak if not explicitly disposed (or cleaned up by scope disposal). Today, forgetting to dispose a memo is harmless because `Rc` drop handles it. With `Copy` handles, forgotten disposal means the computation closure, deps, and subscriber all remain allocated in the arena until the runtime is disposed or the scope is cleaned up.
- **Significant refactoring effort.** Requires:
  - New arena types in `Runtime` (`Vec<MemoSlot>`, `Vec<EffectSlot>`) with free lists and generation tracking
  - Moving computation closures (`Box<dyn Fn() -> T>`) into arena slots, which means type-erasing them (since the arena must be homogeneous or use `Box<dyn Any>`)
  - Rewriting freshener closures to look up memo state by ID instead of holding `Weak<MemoInner>`
  - Rewriting subscriber callbacks for effects to look up effect state by ID instead of holding `Rc<EffectInner>`
  - Rewriting `dispose_memo`/`dispose_effect` to mark arena slots dead and push to free list
  - Updating all cleanup logic (effect cleanups, dep tracking, subscriber unregistration) to work via arena lookups instead of direct `Rc` access
- **Type erasure complexity.** Memo computations are `Box<dyn Fn() -> T>`, which is generic over `T`. Storing these in a flat `Vec<MemoSlot>` requires type-erasing the closure (e.g., `Box<dyn Any>`) and downcasting on access, mirroring the pattern already used for signal values but now for closures too.
- **Subscriber callback indirection.** Effect subscriber callbacks currently capture a direct `Rc` to the effect state. With arena storage, they would need to capture an `EffectId` and look up the state at invocation time, adding a runtime lookup on every notification.
- **Scope-based disposal becomes mandatory.** Today, scopes are the *primary* disposal mechanism, but `Rc`-based Drop acts as a safety net. With `Copy` handles, scopes become the *only* disposal mechanism for memos/effects. Any memo or effect created outside a scope (or in a long-lived scope) will never be cleaned up unless explicitly disposed. This shifts a class of bugs from "harmless" to "memory leak."

## Recommendation

**Option A: Keep Memo and Effect as Clone-only (status quo).**

The ergonomic benefit of `Copy` handles is real but modest — it saves a `.clone()` call when passing a memo into multiple closures. The implementation cost is substantial and introduces real risks:

1. **Automatic cleanup is too valuable to lose.** The `Rc`-based Drop for memos is a safety net that prevents leaks when users forget explicit disposal. With the scope system not yet battle-tested, removing this safety net would be premature. Leaked closures holding signal references could cause subtle bugs (stale subscriptions, unexpected re-evaluations).

2. **The refactoring is large and cuts across the entire reactive core.** Moving memo/effect state into arenas means rewriting freshener registration, subscriber callbacks, dependency tracking, disposal, and cleanup — essentially the most complex parts of the reactive system. This level of churn should be justified by a proportional benefit.

3. **The ergonomic gap is narrowing.** If the panicking API change (see `panicking-api.md`) is adopted, the main source of boilerplate (`.unwrap()`) disappears. The remaining `.clone()` on memos is a minor annoyance by comparison, and one that IDEs handle well (auto-complete, quick-fix).

4. **Consistency is not the only design value.** `Signal<T>` is `Copy` because its internal structure is inherently just an index — there is no computation, no deps, no cleanup. Memos and effects have fundamentally richer internal state. The `Clone`-vs-`Copy` distinction communicates this difference honestly.

If the scope system matures and proves reliable as the sole disposal mechanism, this decision can be revisited. The arena infrastructure for signals already exists and could be extended. But today, the risk/reward ratio does not justify the change.
