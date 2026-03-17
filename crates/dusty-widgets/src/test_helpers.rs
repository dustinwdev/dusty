//! Shared test utilities for widget unit tests.

use dusty_core::{Element, Node};
use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime, Scope};

/// Drop guard that ensures `dispose_runtime()` runs even if a test panics.
/// This prevents runtime state from leaking into subsequent tests.
pub(crate) struct RuntimeGuard;

impl Drop for RuntimeGuard {
    fn drop(&mut self) {
        dispose_runtime();
    }
}

/// Initializes a reactive runtime, runs `f` inside a scope, then disposes
/// the runtime. Panics propagate after cleanup thanks to [`RuntimeGuard`].
pub(crate) fn with_scope(f: impl FnOnce(Scope)) {
    initialize_runtime();
    let _guard = RuntimeGuard;
    create_scope(|cx| f(cx));
}

/// Extracts the inner [`Element`] from a `Component(Element)` node.
///
/// # Panics
///
/// Panics if `node` is not `Component` wrapping an `Element`.
pub(crate) fn extract_element(node: &Node) -> &Element {
    match node {
        Node::Component(comp) => match &*comp.child {
            Node::Element(el) => el,
            _ => panic!("expected Element inside Component"),
        },
        _ => panic!("expected Component node"),
    }
}
