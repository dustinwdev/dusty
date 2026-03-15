//! Resources — reactive async data that integrates with signals.
//!
//! A resource wraps an async or sync data-fetching operation, tracks a source
//! signal, and automatically re-fetches when the source changes. The resource
//! state (`Loading`, `Ready`, `Errored`) is itself a signal, so effects and
//! memos can react to resource changes.
//!
//! Since `dusty-reactive` has zero async runtime dependencies, the fetcher
//! receives a [`ResourceResolver`] callback to report completion. This works
//! with any executor.
//!
//! # Examples
//!
//! ```
//! # dusty_reactive::initialize_runtime();
//! let id = dusty_reactive::create_signal(1u32).unwrap();
//!
//! let resource = dusty_reactive::create_resource(
//!     move || id.get().unwrap(),
//!     |source_val, resolver| {
//!         // Sync fetcher for demonstration
//!         resolver.resolve(source_val * 10);
//!     },
//! ).unwrap();
//!
//! assert_eq!(resource.get().unwrap(), Some(10));
//!
//! id.set(2).unwrap();
//! assert_eq!(resource.get().unwrap(), Some(20));
//! # dusty_reactive::dispose_runtime();
//! ```

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;

use crate::effect::{create_effect, Effect};
use crate::error::{ReactiveError, Result};
use crate::signal::{create_signal, Signal};

/// The state of a resource's data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState<T> {
    /// Initial state before the first fetch completes.
    Unresolved,
    /// A fetch is in progress.
    Loading,
    /// The fetch succeeded with a value.
    Ready(T),
    /// The fetch failed with an error message.
    Errored(String),
}

/// A handle passed to the fetcher to report completion.
///
/// Each resolver is tied to a specific generation — if the source changes
/// before the fetcher completes, the resolver becomes stale and its
/// `resolve`/`reject` calls are silently ignored.
pub struct ResourceResolver<T: 'static> {
    generation: u64,
    state_signal: Signal<ResourceState<T>>,
    current_generation: Rc<Cell<u64>>,
}

impl<T: Clone + PartialEq + 'static> ResourceResolver<T> {
    /// Report a successful fetch result.
    ///
    /// If the source has changed since this resolver was created (generation
    /// mismatch), the call is silently ignored.
    pub fn resolve(&self, value: T) {
        if self.generation == self.current_generation.get() {
            let _ = self.state_signal.set(ResourceState::Ready(value));
        }
    }

    /// Report a fetch failure.
    ///
    /// If the source has changed since this resolver was created (generation
    /// mismatch), the call is silently ignored.
    pub fn reject(&self, error: impl Into<String>) {
        if self.generation == self.current_generation.get() {
            let _ = self.state_signal.set(ResourceState::Errored(error.into()));
        }
    }
}

/// A reactive resource that fetches data based on a source signal.
///
/// `Resource<T>` is `Clone` (via `Rc`).
pub struct Resource<T: 'static> {
    inner: Rc<ResourceInner<T>>,
    _not_send: PhantomData<*const ()>,
}

impl<T: 'static> std::fmt::Debug for Resource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("disposed", &self.inner.disposed.get())
            .finish_non_exhaustive()
    }
}

impl<T: 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            _not_send: PhantomData,
        }
    }
}

struct ResourceInner<T: 'static> {
    state_signal: Signal<ResourceState<T>>,
    /// Kept alive to share generation tracking with resolvers via `Rc::clone`.
    #[allow(dead_code)]
    current_generation: Rc<Cell<u64>>,
    effect: RefCell<Option<Effect>>,
    disposed: Cell<bool>,
}

/// Create a resource that fetches data based on a reactive source.
///
/// The `source` closure is tracked — when any signal it reads changes, the
/// resource re-fetches. The `fetcher` receives the source value and a
/// [`ResourceResolver`] to report completion.
///
/// # Errors
///
/// Returns [`ReactiveError::NoRuntime`] if no runtime is initialized.
pub fn create_resource<S, T>(
    source: impl Fn() -> S + 'static,
    fetcher: impl Fn(S, ResourceResolver<T>) + 'static,
) -> Result<Resource<T>>
where
    S: 'static,
    T: Clone + PartialEq + 'static,
{
    let state_signal = create_signal(ResourceState::<T>::Unresolved)?;
    let current_generation = Rc::new(Cell::new(0u64));

    let gen = Rc::clone(&current_generation);

    let effect = create_effect(move || {
        let source_val = source();

        // Batch so that Loading + Ready/Errored coalesce into one notification.
        let _ = crate::batch::batch(|| {
            // Increment generation
            let new_gen = gen.get().saturating_add(1);
            gen.set(new_gen);

            // Set to Loading
            let _ = state_signal.set(ResourceState::Loading);

            let resolver = ResourceResolver {
                generation: new_gen,
                state_signal,
                current_generation: Rc::clone(&gen),
            };

            fetcher(source_val, resolver);
        });
    })?;

    let inner = Rc::new(ResourceInner {
        state_signal,
        current_generation,
        effect: RefCell::new(Some(effect)),
        disposed: Cell::new(false),
    });

    Ok(Resource {
        inner,
        _not_send: PhantomData,
    })
}

/// Dispose a resource, cleaning up its internal effect and signal.
///
/// # Errors
///
/// Returns [`ReactiveError::ResourceDisposed`] if already disposed.
pub fn dispose_resource<T: Clone + PartialEq + 'static>(resource: &Resource<T>) -> Result<()> {
    if resource.inner.disposed.get() {
        return Err(ReactiveError::ResourceDisposed);
    }
    resource.inner.disposed.set(true);

    if let Some(effect) = resource.inner.effect.borrow_mut().take() {
        let _ = crate::effect::dispose_effect(&effect);
    }

    Ok(())
}

impl<T: Clone + PartialEq + 'static> Resource<T> {
    /// Get the full resource state. Registers the caller as a subscriber.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the resource is disposed.
    pub fn state(&self) -> Result<ResourceState<T>> {
        if self.inner.disposed.get() {
            return Err(ReactiveError::ResourceDisposed);
        }
        self.inner.state_signal.get().map_err(|e| match e {
            ReactiveError::SignalDisposed => ReactiveError::ResourceDisposed,
            other => other,
        })
    }

    /// Get the value if ready, `None` otherwise. Registers the caller as a
    /// subscriber.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the resource is disposed.
    pub fn get(&self) -> Result<Option<T>> {
        match self.state()? {
            ResourceState::Ready(val) => Ok(Some(val)),
            _ => Ok(None),
        }
    }

    /// Returns `true` if the resource is currently loading. Tracked.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the resource is disposed.
    pub fn loading(&self) -> Result<bool> {
        Ok(matches!(self.state()?, ResourceState::Loading))
    }

    /// Get the error message if in error state, `None` otherwise. Tracked.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime is unavailable or the resource is disposed.
    pub fn error(&self) -> Result<Option<String>> {
        match self.state()? {
            ResourceState::Errored(msg) => Ok(Some(msg)),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::dispose_runtime;
    use crate::signal::create_signal;
    use crate::tracking::with_test_runtime;
    use static_assertions::assert_not_impl_any;
    use std::cell::Cell;
    use std::rc::Rc;

    assert_not_impl_any!(Resource<i32>: Send, Sync);

    #[test]
    fn resource_sync_fetcher_resolves_immediately() {
        with_test_runtime(|| {
            let source = create_signal(5).unwrap();
            let resource = create_resource(
                move || source.get().unwrap(),
                |val, resolver| {
                    resolver.resolve(val * 2);
                },
            )
            .unwrap();

            assert_eq!(resource.state().unwrap(), ResourceState::Ready(10));
        });
    }

    #[test]
    fn resource_get_returns_value_when_ready() {
        with_test_runtime(|| {
            let resource = create_resource(
                || 42,
                |val, resolver| {
                    resolver.resolve(val);
                },
            )
            .unwrap();

            assert_eq!(resource.get().unwrap(), Some(42));
        });
    }

    #[test]
    fn resource_get_returns_none_when_loading() {
        with_test_runtime(|| {
            let resource = create_resource(
                || 1,
                |_val, _resolver: ResourceResolver<i32>| {
                    // Don't resolve — stays loading
                },
            )
            .unwrap();

            assert_eq!(resource.get().unwrap(), None);
            assert!(resource.loading().unwrap());
        });
    }

    #[test]
    fn resource_loading_returns_true_when_loading() {
        with_test_runtime(|| {
            let resource = create_resource(
                || 1,
                |_val, _resolver: ResourceResolver<i32>| {
                    // Don't resolve
                },
            )
            .unwrap();

            assert!(resource.loading().unwrap());
        });
    }

    #[test]
    fn resource_error_state() {
        with_test_runtime(|| {
            let resource = create_resource(
                || 1,
                |_val, resolver: ResourceResolver<i32>| {
                    resolver.reject("fetch failed");
                },
            )
            .unwrap();

            assert_eq!(
                resource.state().unwrap(),
                ResourceState::Errored("fetch failed".to_string())
            );
            assert_eq!(resource.error().unwrap(), Some("fetch failed".to_string()));
        });
    }

    #[test]
    fn resource_refetches_when_source_changes() {
        with_test_runtime(|| {
            let source = create_signal(1).unwrap();
            let resource = create_resource(
                move || source.get().unwrap(),
                |val, resolver| {
                    resolver.resolve(val * 10);
                },
            )
            .unwrap();

            assert_eq!(resource.get().unwrap(), Some(10));

            source.set(2).unwrap();
            assert_eq!(resource.get().unwrap(), Some(20));

            source.set(5).unwrap();
            assert_eq!(resource.get().unwrap(), Some(50));
        });
    }

    #[test]
    fn resource_stale_resolver_ignored() {
        with_test_runtime(|| {
            let source = create_signal(1).unwrap();
            let saved_resolver: Rc<RefCell<Option<ResourceResolver<i32>>>> =
                Rc::new(RefCell::new(None));
            let sr = Rc::clone(&saved_resolver);

            let resource = create_resource(
                move || source.get().unwrap(),
                move |val, resolver| {
                    if val == 1 {
                        // Save resolver for later, don't resolve yet
                        *sr.borrow_mut() = Some(resolver);
                    } else {
                        resolver.resolve(val * 10);
                    }
                },
            )
            .unwrap();

            // First fetch didn't resolve
            assert!(resource.loading().unwrap());

            // Change source — triggers re-fetch with val=2
            source.set(2).unwrap();
            assert_eq!(resource.get().unwrap(), Some(20));

            // Now resolve the stale resolver from val=1
            if let Some(stale) = saved_resolver.borrow_mut().take() {
                stale.resolve(999);
            }

            // Stale resolve should have been ignored
            assert_eq!(resource.get().unwrap(), Some(20));
        });
    }

    #[test]
    fn resource_state_is_tracked() {
        with_test_runtime(|| {
            let source = create_signal(1).unwrap();
            let resource = create_resource(
                move || source.get().unwrap(),
                |val, resolver| {
                    resolver.resolve(val);
                },
            )
            .unwrap();

            let run_count = Rc::new(Cell::new(0));
            let rc = Rc::clone(&run_count);
            let res = resource.clone();

            let _effect = crate::effect::create_effect(move || {
                let _state = res.state().unwrap();
                rc.set(rc.get() + 1);
            })
            .unwrap();

            assert!(run_count.get() >= 1);
            let before = run_count.get();

            source.set(2).unwrap();
            assert!(run_count.get() > before);
        });
    }

    #[test]
    fn resource_get_is_tracked() {
        with_test_runtime(|| {
            let source = create_signal(1).unwrap();
            let resource = create_resource(
                move || source.get().unwrap(),
                |val, resolver| {
                    resolver.resolve(val * 100);
                },
            )
            .unwrap();

            let observed = Rc::new(Cell::new(0));
            let ob = Rc::clone(&observed);
            let res = resource.clone();

            let _effect = crate::effect::create_effect(move || {
                if let Some(val) = res.get().unwrap() {
                    ob.set(val);
                }
            })
            .unwrap();

            assert_eq!(observed.get(), 100);

            source.set(2).unwrap();
            assert_eq!(observed.get(), 200);
        });
    }

    #[test]
    fn resource_dispose_cleans_up() {
        with_test_runtime(|| {
            let source = create_signal(1).unwrap();
            let fetch_count = Rc::new(Cell::new(0));
            let fc = Rc::clone(&fetch_count);

            let resource = create_resource(
                move || source.get().unwrap(),
                move |val, resolver| {
                    fc.set(fc.get() + 1);
                    resolver.resolve(val);
                },
            )
            .unwrap();

            assert_eq!(fetch_count.get(), 1);

            dispose_resource(&resource).unwrap();

            // Changing source should NOT trigger re-fetch
            source.set(2).unwrap();
            assert_eq!(fetch_count.get(), 1);

            // State should return error after dispose
            assert_eq!(
                resource.state().unwrap_err(),
                ReactiveError::ResourceDisposed
            );
        });
    }

    #[test]
    fn resource_no_runtime_returns_error() {
        dispose_runtime();
        let result = create_resource(
            || 1,
            |_val, resolver: ResourceResolver<i32>| {
                resolver.resolve(1);
            },
        );
        assert!(result.is_err());
    }
}
