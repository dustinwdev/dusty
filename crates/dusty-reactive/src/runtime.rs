//! Thread-local reactive runtime with arena-allocated signal storage.

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashSet;

use crate::error::{ReactiveError, Result};
use crate::subscriber::SubscriberId;

/// Generational index into the signal slab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalId {
    pub index: usize,
    pub generation: u64,
}

/// A single slot in the signal slab.
pub struct SignalSlot {
    pub value: Box<dyn Any>,
    pub generation: u64,
    pub subscribers: HashSet<SubscriberId>,
    pub alive: bool,
    /// Incremented each time the value changes. Used by memos to detect
    /// whether a dependency's value actually changed since last evaluation.
    pub version: u64,
}

/// Generational index into the scope slab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeId {
    pub index: usize,
    pub generation: u64,
}

/// A single slot in the scope slab.
pub struct ScopeSlot {
    pub generation: u64,
    pub alive: bool,
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub disposers: Vec<Box<dyn FnOnce()>>,
}

/// The reactive runtime. Holds all signal data for a single thread.
pub struct Runtime {
    pub signals: Vec<SignalSlot>,
    pub free_list: Vec<usize>,
    pub subscribers: Vec<Option<Box<dyn Fn()>>>,
    pub subscriber_generations: Vec<u64>,
    pub subscriber_free_list: Vec<usize>,
    pub tracking_stack: Vec<SubscriberId>,
    /// Parallel stack tracking which signals are read during each tracking scope.
    /// Each entry corresponds to the same index in `tracking_stack`.
    pub dependency_stack: Vec<Vec<SignalId>>,
    pub scopes: Vec<ScopeSlot>,
    pub scope_free_list: Vec<usize>,
    pub scope_stack: Vec<ScopeId>,
    pub batch_depth: usize,
    pub pending_batch_subscribers: HashSet<SubscriberId>,
}

impl Runtime {
    fn new() -> Self {
        Self {
            signals: Vec::new(),
            free_list: Vec::new(),
            subscribers: Vec::new(),
            subscriber_generations: Vec::new(),
            subscriber_free_list: Vec::new(),
            tracking_stack: Vec::new(),
            dependency_stack: Vec::new(),
            scopes: Vec::new(),
            scope_free_list: Vec::new(),
            scope_stack: Vec::new(),
            batch_depth: 0,
            pending_batch_subscribers: HashSet::new(),
        }
    }
}

thread_local! {
    static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) };
}

/// Initialize the reactive runtime on the current thread.
///
/// Must be called before any signal operations. Safe to call multiple times —
/// subsequent calls reset the runtime.
///
/// # Examples
///
/// ```
/// dusty_reactive::initialize_runtime();
/// // ... use signals ...
/// dusty_reactive::dispose_runtime();
/// ```
pub fn initialize_runtime() {
    RUNTIME.with(|rt| {
        *rt.borrow_mut() = Some(Runtime::new());
    });
}

/// Dispose of the reactive runtime on the current thread, freeing all signals.
///
/// After disposal, signal operations will return [`ReactiveError::NoRuntime`].
/// Also clears auxiliary thread-locals (freshener registry, pending effects,
/// cleanup sink) to prevent stale state from leaking into a future runtime.
pub fn dispose_runtime() {
    RUNTIME.with(|rt| {
        *rt.borrow_mut() = None;
    });
    crate::memo::clear_fresheners();
    crate::effect::clear_thread_locals();
}

/// Access the runtime immutably.
pub fn with_runtime<R>(f: impl FnOnce(&Runtime) -> R) -> Result<R> {
    RUNTIME.with(|rt| {
        let borrow = rt
            .try_borrow()
            .map_err(|_| ReactiveError::RuntimeBorrowError)?;
        let runtime = borrow.as_ref().ok_or(ReactiveError::NoRuntime)?;
        Ok(f(runtime))
    })
}

/// Access the runtime mutably.
pub fn with_runtime_mut<R>(f: impl FnOnce(&mut Runtime) -> R) -> Result<R> {
    RUNTIME.with(|rt| {
        let mut borrow = rt
            .try_borrow_mut()
            .map_err(|_| ReactiveError::RuntimeBorrowError)?;
        let runtime = borrow.as_mut().ok_or(ReactiveError::NoRuntime)?;
        Ok(f(runtime))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each test needs its own runtime since they share the thread-local.
    fn with_test_runtime(f: impl FnOnce()) {
        initialize_runtime();
        f();
        dispose_runtime();
    }

    #[test]
    fn initialize_and_dispose() {
        with_test_runtime(|| {
            // Runtime exists — should be able to access it
            let result = with_runtime(|_| ());
            assert!(result.is_ok());
        });
        // After dispose, runtime is gone
        let result = with_runtime(|_| ());
        assert_eq!(result.unwrap_err(), ReactiveError::NoRuntime);
    }

    #[test]
    fn no_runtime_error() {
        dispose_runtime(); // ensure clean
        let result = with_runtime(|_| ());
        assert_eq!(result.unwrap_err(), ReactiveError::NoRuntime);

        let result = with_runtime_mut(|_| ());
        assert_eq!(result.unwrap_err(), ReactiveError::NoRuntime);
    }

    #[test]
    fn double_init_resets() {
        initialize_runtime();
        // Create a signal slot manually to verify reset
        let _ = with_runtime_mut(|rt| {
            rt.signals.push(SignalSlot {
                value: Box::new(42_i32),
                generation: 0,
                subscribers: HashSet::new(),
                alive: true,
                version: 0,
            });
        });
        // Re-initialize should give us a fresh runtime
        initialize_runtime();
        let count = with_runtime(|rt| rt.signals.len()).unwrap();
        assert_eq!(count, 0);
        dispose_runtime();
    }

    #[test]
    fn with_runtime_returns_value() {
        with_test_runtime(|| {
            let val = with_runtime(|_| 42).unwrap();
            assert_eq!(val, 42);
        });
    }

    #[test]
    fn with_runtime_mut_returns_value() {
        with_test_runtime(|| {
            let val = with_runtime_mut(|_| "hello").unwrap();
            assert_eq!(val, "hello");
        });
    }
}
