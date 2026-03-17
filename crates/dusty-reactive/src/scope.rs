//! Scopes — arena-based ownership for automatic cleanup of reactive primitives.
//!
//! A scope owns signals, memos, and effects created within it. When a scope
//! is disposed, all owned primitives are cleaned up in reverse (LIFO) order.
//!
//! # Examples
//!
//! ```
//! # dusty_reactive::initialize_runtime();
//! let scope = dusty_reactive::create_scope(|_s| {
//!     let sig = dusty_reactive::create_signal(42);
//!     assert_eq!(sig.get(), 42);
//! });
//!
//! dusty_reactive::dispose_scope(scope);
//! # dusty_reactive::dispose_runtime();
//! ```

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::error::{unwrap_reactive, ReactiveError, Result};
use crate::runtime::{with_runtime, with_runtime_mut, ScopeId, ScopeSlot};

/// A reactive scope that owns signals, memos, and effects.
///
/// `Scope` is `Copy` — it's a lightweight handle wrapping a `ScopeId`.
/// The actual ownership data lives in the thread-local runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scope {
    id: ScopeId,
    _not_send: PhantomData<*const ()>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a new root scope. Runs `f` with the scope active, so any signals,
/// memos, or effects created inside `f` are owned by this scope.
///
/// # Panics
///
/// Panics if no runtime is initialized. Use [`try_create_scope`] for a
/// fallible variant.
#[track_caller]
pub fn create_scope(f: impl FnOnce(Scope)) -> Scope {
    unwrap_reactive(try_create_scope(f), "create_scope")
}

/// Fallible version of [`create_scope`].
///
/// # Errors
///
/// Returns [`ReactiveError::NoRuntime`] if no runtime is initialized.
pub fn try_create_scope(f: impl FnOnce(Scope)) -> Result<Scope> {
    let id = alloc_scope(None)?;
    let scope = Scope {
        id,
        _not_send: PhantomData,
    };
    push_scope(id)?;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(scope)));

    pop_scope()?;

    match result {
        Ok(()) => Ok(scope),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

/// Create a child scope under `parent`. The child is registered in the
/// parent's children list and will be disposed when the parent is disposed.
///
/// # Panics
///
/// Panics if the parent scope is dead. Use [`try_create_child_scope`] for a
/// fallible variant.
#[track_caller]
pub fn create_child_scope(parent: Scope, f: impl FnOnce(Scope)) -> Scope {
    unwrap_reactive(try_create_child_scope(parent, f), "create_child_scope")
}

/// Fallible version of [`create_child_scope`].
///
/// # Errors
///
/// Returns [`ReactiveError::ScopeDisposed`] if the parent scope is dead.
pub fn try_create_child_scope(parent: Scope, f: impl FnOnce(Scope)) -> Result<Scope> {
    validate_scope(parent.id)?;
    let id = alloc_scope(Some(parent.id))?;

    // Register child in parent's children list
    with_runtime_mut(|rt| {
        let parent_slot = &mut rt.scopes[parent.id.index];
        parent_slot.children.push(id);
    })?;

    let scope = Scope {
        id,
        _not_send: PhantomData,
    };
    push_scope(id)?;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(scope)));

    pop_scope()?;

    match result {
        Ok(()) => Ok(scope),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

/// Dispose a scope: recursively dispose children (depth-first), run disposers
/// in reverse (LIFO) order, mark dead, and remove from parent's children list.
///
/// # Panics
///
/// Panics if the scope was already disposed. Use [`try_dispose_scope`] for a
/// fallible variant.
#[track_caller]
pub fn dispose_scope(scope: Scope) {
    unwrap_reactive(try_dispose_scope(scope), "dispose_scope");
}

/// Fallible version of [`dispose_scope`].
///
/// # Errors
///
/// Returns [`ReactiveError::ScopeDisposed`] if the scope was already disposed.
pub fn try_dispose_scope(scope: Scope) -> Result<()> {
    dispose_scope_inner(scope.id)
}

/// Store a value in the current scope's context, keyed by its [`TypeId`].
///
/// # Panics
///
/// Panics if no scope is currently active. Use [`try_provide_context`] for a
/// fallible variant.
///
/// # Examples
///
/// ```
/// # dusty_reactive::initialize_runtime();
/// let _scope = dusty_reactive::create_scope(|_s| {
///     dusty_reactive::provide_context(42_i32);
///     let val = dusty_reactive::use_context::<i32>();
///     assert_eq!(val, Some(42));
/// });
/// # dusty_reactive::dispose_runtime();
/// ```
#[track_caller]
pub fn provide_context<T: 'static>(value: T) {
    unwrap_reactive(try_provide_context(value), "provide_context");
}

/// Fallible version of [`provide_context`].
///
/// # Errors
///
/// Returns [`ReactiveError::ScopeDisposed`] if no scope is currently active.
pub fn try_provide_context<T: 'static>(value: T) -> Result<()> {
    let scope_id = current_scope()?;
    let id = scope_id.ok_or(ReactiveError::ScopeDisposed)?;
    with_runtime_mut(|rt| {
        let slot = &mut rt.scopes[id.index];
        if slot.alive && slot.generation == id.generation {
            slot.contexts.insert(TypeId::of::<T>(), Box::new(value));
        }
    })
}

/// Walk up the scope tree from the current scope, returning the first value
/// of type `T` found (cloned). Returns `None` if no scope is active or
/// no value of type `T` exists in any ancestor scope.
///
/// # Panics
///
/// Panics if no runtime is initialized. Use [`try_use_context`] for a
/// fallible variant.
///
/// # Examples
///
/// ```
/// # dusty_reactive::initialize_runtime();
/// let _scope = dusty_reactive::create_scope(|_s| {
///     dusty_reactive::provide_context("hello".to_string());
///     let val = dusty_reactive::use_context::<String>();
///     assert_eq!(val.as_deref(), Some("hello"));
/// });
/// # dusty_reactive::dispose_runtime();
/// ```
#[must_use]
#[track_caller]
pub fn use_context<T: Clone + 'static>() -> Option<T> {
    unwrap_reactive(try_use_context(), "use_context")
}

/// Fallible version of [`use_context`].
///
/// # Errors
///
/// Returns [`ReactiveError::NoRuntime`] if no runtime is initialized.
pub fn try_use_context<T: Clone + 'static>() -> Result<Option<T>> {
    with_runtime(|rt| {
        let mut scope_id = rt.scope_stack.last().copied();
        while let Some(id) = scope_id {
            if let Some(slot) = rt.scopes.get(id.index) {
                if slot.alive && slot.generation == id.generation {
                    if let Some(value) = slot.contexts.get(&TypeId::of::<T>()) {
                        if let Some(val) = value.downcast_ref::<T>() {
                            return Some(val.clone());
                        }
                    }
                    scope_id = slot.parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        None
    })
}

// ---------------------------------------------------------------------------
// Scope methods
// ---------------------------------------------------------------------------

impl Scope {
    /// Push this scope onto the scope stack, run `f`, then pop.
    /// Any reactive primitives created inside `f` will be owned by this scope.
    ///
    /// # Panics
    ///
    /// Panics if the scope is dead. Use [`Scope::try_run`] for a fallible variant.
    #[track_caller]
    pub fn run<R>(&self, f: impl FnOnce() -> R) -> R {
        unwrap_reactive(self.try_run(f), "Scope::run")
    }

    /// Fallible version of [`Scope::run`].
    ///
    /// # Errors
    ///
    /// Returns [`ReactiveError::ScopeDisposed`] if the scope is dead.
    pub fn try_run<R>(&self, f: impl FnOnce() -> R) -> Result<R> {
        validate_scope(self.id)?;
        push_scope(self.id)?;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

        pop_scope()?;

        match result {
            Ok(val) => Ok(val),
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    /// Create a child scope under this scope.
    ///
    /// # Panics
    ///
    /// Panics if this scope is dead. Use [`Scope::try_create_child`] for a
    /// fallible variant.
    #[must_use]
    #[track_caller]
    pub fn create_child(&self, f: impl FnOnce(Self)) -> Self {
        unwrap_reactive(self.try_create_child(f), "Scope::create_child")
    }

    /// Fallible version of [`Scope::create_child`].
    ///
    /// # Errors
    ///
    /// Returns [`ReactiveError::ScopeDisposed`] if this scope is dead.
    pub fn try_create_child(&self, f: impl FnOnce(Self)) -> Result<Self> {
        try_create_child_scope(*self, f)
    }

    /// Dispose this scope and all its owned primitives.
    ///
    /// # Panics
    ///
    /// Panics if the scope was already disposed. Use [`Scope::try_dispose`]
    /// for a fallible variant.
    #[track_caller]
    pub fn dispose(self) {
        unwrap_reactive(self.try_dispose(), "Scope::dispose");
    }

    /// Fallible version of [`Scope::dispose`].
    ///
    /// # Errors
    ///
    /// Returns [`ReactiveError::ScopeDisposed`] if the scope was already disposed.
    pub fn try_dispose(self) -> Result<()> {
        try_dispose_scope(self)
    }
}

// ---------------------------------------------------------------------------
// Internal (pub(crate)) API
// ---------------------------------------------------------------------------

/// Peek the scope stack to get the current active scope, if any.
///
/// Uses an immutable borrow — safe to call during notification callbacks.
pub(crate) fn current_scope() -> Result<Option<ScopeId>> {
    with_runtime(|rt| rt.scope_stack.last().copied())
}

/// Push a scope onto the scope stack.
pub(crate) fn push_scope(id: ScopeId) -> Result<()> {
    with_runtime_mut(|rt| {
        rt.scope_stack.push(id);
    })
}

/// Pop the current scope from the scope stack.
pub(crate) fn pop_scope() -> Result<()> {
    with_runtime_mut(|rt| {
        rt.scope_stack.pop();
    })
}

/// Register a disposer closure with the current active scope.
/// If no scope is active, this is a no-op (the primitive is unscoped).
pub(crate) fn register_disposer(disposer: Box<dyn FnOnce()>) -> Result<()> {
    let scope_id = current_scope()?;
    if let Some(id) = scope_id {
        with_runtime_mut(|rt| {
            if let Some(slot) = rt.scopes.get_mut(id.index) {
                if slot.alive && slot.generation == id.generation {
                    slot.disposers.push(disposer);
                }
            }
        })?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn alloc_scope(parent: Option<ScopeId>) -> Result<ScopeId> {
    with_runtime_mut(|rt| {
        if let Some(index) = rt.scope_free_list.pop() {
            let generation = rt.scopes[index].generation + 1;
            rt.scopes[index] = ScopeSlot {
                generation,
                alive: true,
                parent,
                children: Vec::new(),
                disposers: Vec::new(),
                contexts: HashMap::new(),
            };
            ScopeId { index, generation }
        } else {
            let index = rt.scopes.len();
            rt.scopes.push(ScopeSlot {
                generation: 0,
                alive: true,
                parent,
                children: Vec::new(),
                disposers: Vec::new(),
                contexts: HashMap::new(),
            });
            ScopeId {
                index,
                generation: 0,
            }
        }
    })
}

fn validate_scope(id: ScopeId) -> Result<()> {
    with_runtime(|rt| {
        let slot = rt
            .scopes
            .get(id.index)
            .ok_or(ReactiveError::ScopeDisposed)?;
        if !slot.alive || slot.generation != id.generation {
            return Err(ReactiveError::ScopeDisposed);
        }
        Ok(())
    })?
}

fn dispose_scope_inner(id: ScopeId) -> Result<()> {
    // Validate scope is alive
    validate_scope(id)?;

    // Collect children to dispose recursively (depth-first)
    let children = with_runtime_mut(|rt| {
        let slot = &rt.scopes[id.index];
        slot.children.clone()
    })?;

    for child in children {
        // Best-effort: child may already be disposed independently
        let _ = dispose_scope_inner(child);
    }

    // Take disposers and run them in reverse (LIFO) order
    let disposers = with_runtime_mut(|rt| {
        let slot = &mut rt.scopes[id.index];
        std::mem::take(&mut slot.disposers)
    })?;

    for disposer in disposers.into_iter().rev() {
        disposer();
    }

    // Mark dead, free slot, remove from parent's children list
    with_runtime_mut(|rt| {
        let parent = rt.scopes[id.index].parent;
        rt.scopes[id.index].alive = false;
        rt.scopes[id.index].children.clear();
        rt.scopes[id.index].contexts.clear();
        rt.scope_free_list.push(id.index);

        if let Some(parent_id) = parent {
            if let Some(parent_slot) = rt.scopes.get_mut(parent_id.index) {
                if parent_slot.alive && parent_slot.generation == parent_id.generation {
                    parent_slot.children.retain(|c| *c != id);
                }
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracking::with_test_runtime;
    use static_assertions::assert_not_impl_any;
    use std::cell::Cell;
    use std::rc::Rc;

    assert_not_impl_any!(Scope: Send, Sync);

    #[test]
    fn create_scope_and_access() {
        with_test_runtime(|| {
            let scope = create_scope(|_s| {});
            // Scope exists and is valid
            assert!(validate_scope(scope.id).is_ok());
        });
    }

    #[test]
    fn scope_is_copy() {
        with_test_runtime(|| {
            let scope = create_scope(|_s| {});
            let scope2 = scope;
            assert_eq!(scope, scope2);
        });
    }

    #[test]
    fn scope_disposes_signals() {
        with_test_runtime(|| {
            let sig_handle = Rc::new(Cell::new(None));
            let sh = Rc::clone(&sig_handle);

            let scope = create_scope(|_s| {
                let sig = crate::signal::create_signal(42);
                assert_eq!(sig.get(), 42);
                sh.set(Some(sig));
            });

            // Signal works before scope dispose
            let sig = sig_handle.get().unwrap();
            assert_eq!(sig.get(), 42);

            // Dispose scope — signal should be cleaned up
            dispose_scope(scope);
            assert_eq!(sig.try_get().unwrap_err(), ReactiveError::SignalDisposed);
        });
    }

    #[test]
    fn scope_disposes_memos() {
        with_test_runtime(|| {
            let source = crate::signal::create_signal(5);
            let memo_handle = Rc::new(RefCell::new(None));
            let mh = Rc::clone(&memo_handle);

            let scope = create_scope(|_s| {
                let m = crate::memo::create_memo(move || source.get() * 2);
                assert_eq!(m.get(), 10);
                *mh.borrow_mut() = Some(m);
            });

            let m = memo_handle.borrow().as_ref().unwrap().clone();
            assert_eq!(m.get(), 10);

            dispose_scope(scope);
            assert_eq!(m.try_get().unwrap_err(), ReactiveError::MemoDisposed);
        });
    }

    #[test]
    fn scope_disposal_reverse_order() {
        with_test_runtime(|| {
            let order = Rc::new(RefCell::new(Vec::new()));

            let scope = create_scope(|_s| {
                let o1 = Rc::clone(&order);
                register_disposer(Box::new(move || o1.borrow_mut().push(1))).unwrap();
                let o2 = Rc::clone(&order);
                register_disposer(Box::new(move || o2.borrow_mut().push(2))).unwrap();
                let o3 = Rc::clone(&order);
                register_disposer(Box::new(move || o3.borrow_mut().push(3))).unwrap();
            });

            dispose_scope(scope);
            assert_eq!(*order.borrow(), vec![3, 2, 1]);
        });
    }

    #[test]
    fn nested_scope_parent_disposes_children() {
        with_test_runtime(|| {
            let child_disposed = Rc::new(Cell::new(false));
            let cd = Rc::clone(&child_disposed);

            let parent = create_scope(|p| {
                let _child = create_child_scope(p, |_c| {
                    let cd2 = Rc::clone(&cd);
                    register_disposer(Box::new(move || cd2.set(true))).unwrap();
                });
            });

            assert!(!child_disposed.get());
            dispose_scope(parent);
            assert!(child_disposed.get());
        });
    }

    #[test]
    fn nested_scope_child_disposal_independent() {
        with_test_runtime(|| {
            let parent_marker = Rc::new(Cell::new(false));
            let child_marker = Rc::new(Cell::new(false));
            let pm = Rc::clone(&parent_marker);
            let cm = Rc::clone(&child_marker);

            let child_handle = Rc::new(Cell::new(None));
            let ch = Rc::clone(&child_handle);

            let parent = create_scope(|p| {
                let pm2 = Rc::clone(&pm);
                register_disposer(Box::new(move || pm2.set(true))).unwrap();

                let child = create_child_scope(p, |_c| {
                    let cm2 = Rc::clone(&cm);
                    register_disposer(Box::new(move || cm2.set(true))).unwrap();
                });
                ch.set(Some(child));
            });

            let child = child_handle.get().unwrap();

            // Disposing child should NOT dispose parent
            dispose_scope(child);
            assert!(child_marker.get());
            assert!(!parent_marker.get());

            // Parent should still be alive
            dispose_scope(parent);
            assert!(parent_marker.get());
        });
    }

    #[test]
    fn deeply_nested_scopes() {
        with_test_runtime(|| {
            let order = Rc::new(RefCell::new(Vec::new()));

            let root = create_scope(|s1| {
                let o = Rc::clone(&order);
                register_disposer(Box::new(move || o.borrow_mut().push("root"))).unwrap();

                let _mid = create_child_scope(s1, |s2| {
                    let o = Rc::clone(&order);
                    register_disposer(Box::new(move || o.borrow_mut().push("mid"))).unwrap();

                    let _leaf = create_child_scope(s2, |_s3| {
                        let o = Rc::clone(&order);
                        register_disposer(Box::new(move || o.borrow_mut().push("leaf"))).unwrap();
                    });
                });
            });

            dispose_scope(root);
            // Depth-first: leaf disposed first, then mid, then root
            assert_eq!(*order.borrow(), vec!["leaf", "mid", "root"]);
        });
    }

    #[test]
    fn double_dispose_scope_errors() {
        with_test_runtime(|| {
            let scope = create_scope(|_s| {});
            dispose_scope(scope);
            assert_eq!(
                try_dispose_scope(scope).unwrap_err(),
                ReactiveError::ScopeDisposed
            );
        });
    }

    #[test]
    fn scope_run_pushes_and_pops() {
        with_test_runtime(|| {
            let scope = create_scope(|_s| {});

            let sig_handle = Rc::new(Cell::new(None));
            let sh = Rc::clone(&sig_handle);

            scope.run(|| {
                let sig = crate::signal::create_signal(99);
                sh.set(Some(sig));
            });

            let sig = sig_handle.get().unwrap();
            assert_eq!(sig.get(), 99);

            dispose_scope(scope);
            assert_eq!(sig.try_get().unwrap_err(), ReactiveError::SignalDisposed);
        });
    }

    #[test]
    fn scope_run_panic_restores_scope_stack() {
        with_test_runtime(|| {
            let scope = create_scope(|_s| {});

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                scope.run(|| {
                    panic!("scope run panic");
                });
            }));
            assert!(result.is_err());

            // Scope stack should be clean — creating a new scope should work
            let sig_handle = Rc::new(Cell::new(None));
            let sh = Rc::clone(&sig_handle);

            let new_scope = create_scope(|_s| {
                let sig = crate::signal::create_signal(42);
                sh.set(Some(sig));
            });

            let sig = sig_handle.get().unwrap();
            assert_eq!(sig.get(), 42);

            dispose_scope(new_scope);
            assert_eq!(
                sig.try_get().unwrap_err(),
                crate::error::ReactiveError::SignalDisposed
            );
        });
    }

    #[test]
    fn create_scope_panic_restores_scope_stack() {
        with_test_runtime(|| {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                create_scope(|_s| {
                    panic!("create_scope panic");
                });
            }));
            assert!(result.is_err());

            // Scope stack should be clean
            let new_scope = create_scope(|_s| {
                let sig = crate::signal::create_signal(99);
                assert_eq!(sig.get(), 99);
            });
            dispose_scope(new_scope);
        });
    }

    // -----------------------------------------------------------------------
    // Context API tests
    // -----------------------------------------------------------------------

    #[test]
    fn provide_and_use_context_same_scope() {
        with_test_runtime(|| {
            let _scope = create_scope(|_s| {
                provide_context(42_i32);
                let val = use_context::<i32>();
                assert_eq!(val, Some(42));
            });
        });
    }

    #[test]
    fn use_context_walks_up_to_parent() {
        with_test_runtime(|| {
            let _scope = create_scope(|p| {
                provide_context(42_i32);
                let _child = create_child_scope(p, |_c| {
                    let val = use_context::<i32>();
                    assert_eq!(val, Some(42));
                });
            });
        });
    }

    #[test]
    fn use_context_walks_multiple_levels() {
        with_test_runtime(|| {
            let _scope = create_scope(|p| {
                provide_context("root".to_string());
                let _child = create_child_scope(p, |c| {
                    let _grandchild = create_child_scope(c, |_gc| {
                        let val = use_context::<String>();
                        assert_eq!(val, Some("root".to_string()));
                    });
                });
            });
        });
    }

    #[test]
    fn child_context_overrides_parent() {
        with_test_runtime(|| {
            let _scope = create_scope(|p| {
                provide_context(42_i32);
                let _child = create_child_scope(p, |_c| {
                    provide_context(99_i32);
                    let val = use_context::<i32>();
                    assert_eq!(val, Some(99));
                });
            });
        });
    }

    #[test]
    fn use_context_missing_type_returns_none() {
        with_test_runtime(|| {
            let _scope = create_scope(|_s| {
                provide_context(42_i32);
                let val = use_context::<String>();
                assert_eq!(val, None);
            });
        });
    }

    #[test]
    fn provide_context_without_scope_returns_error() {
        with_test_runtime(|| {
            let result = try_provide_context(42_i32);
            assert_eq!(result.unwrap_err(), ReactiveError::ScopeDisposed);
        });
    }

    #[test]
    fn use_context_without_scope_returns_none() {
        with_test_runtime(|| {
            let val = use_context::<i32>();
            assert_eq!(val, None);
        });
    }

    use std::cell::RefCell;
}
