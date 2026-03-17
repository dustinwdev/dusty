use std::panic::{catch_unwind, AssertUnwindSafe};

use dusty_core::node::{ComponentNode, Node};
use dusty_core::view::View;
use dusty_reactive::Scope;

/// A container widget that catches panics from child build functions and
/// renders a fallback instead.
///
/// `ErrorBoundary` wraps a child build closure. If the closure panics during
/// `build`, the panic is caught and a user-supplied fallback is rendered,
/// receiving the panic message as a `String`.
///
/// # Limitations
///
/// This boundary only catches panics during the **build phase** (when the
/// child closure executes). It does **not** catch panics from:
///
/// - Event handler callbacks (e.g. `on_click`)
/// - Effect callbacks (created via `create_effect`)
/// - Async operations or futures
///
/// For comprehensive error handling in event handlers and effects, use
/// `std::panic::catch_unwind` at the call site.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::ErrorBoundary;
/// use dusty_core::node::{text, Node};
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = ErrorBoundary::new()
///         .child(|_cx| Node::Text(text("safe")))
///         .fallback(|msg| Node::Text(text(msg)))
///         .build(cx);
///     assert!(node.is_component());
/// }).unwrap();
/// dispose_runtime();
/// ```
pub struct ErrorBoundary {
    child: Option<Box<dyn FnOnce(Scope) -> Node>>,
    fallback: Option<Box<dyn Fn(String) -> Node>>,
}

impl ErrorBoundary {
    /// Creates a new `ErrorBoundary` with no child and no fallback.
    #[must_use]
    pub fn new() -> Self {
        Self {
            child: None,
            fallback: None,
        }
    }

    /// Sets the child build function.
    ///
    /// The closure receives a [`Scope`] and produces a [`Node`]. If the
    /// closure panics, the error boundary catches it and renders the
    /// fallback instead.
    #[must_use]
    pub fn child(mut self, f: impl FnOnce(Scope) -> Node + 'static) -> Self {
        self.child = Some(Box::new(f));
        self
    }

    /// Sets the fallback function invoked when the child panics.
    ///
    /// The closure receives the panic message as a `String` and produces
    /// a [`Node`] to render in place of the failed child.
    #[must_use]
    pub fn fallback(mut self, f: impl Fn(String) -> Node + 'static) -> Self {
        self.fallback = Some(Box::new(f));
        self
    }
}

/// Extracts a human-readable message from a panic payload.
fn extract_panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown error".to_string())
}

impl Default for ErrorBoundary {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ErrorBoundary {
    fn build(self, cx: Scope) -> Node {
        let child_result = self.child.map_or_else(
            || Ok(Node::Fragment(vec![])),
            |child_fn| catch_unwind(AssertUnwindSafe(|| child_fn(cx))),
        );

        let inner = match child_result {
            Ok(node) => node,
            Err(panic_info) => {
                let message = extract_panic_message(&*panic_info);
                self.fallback
                    .as_ref()
                    .map_or_else(|| Node::Fragment(vec![]), |f| f(message))
            }
        };

        Node::Component(ComponentNode {
            name: "ErrorBoundary",
            child: Box::new(inner),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::node::{text, Node};
    use dusty_core::view::View;
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime, Scope};

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx)).unwrap();
        dispose_runtime();
    }

    #[test]
    fn renders_child_on_success() {
        with_scope(|cx| {
            let node = ErrorBoundary::new()
                .child(|_cx: Scope| Node::Text(text("hello")))
                .fallback(|msg: String| Node::Text(text(msg)))
                .build(cx);

            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert!(comp.child.is_text());
                if let Node::Text(t) = &*comp.child {
                    assert_eq!(t.current_text(), "hello");
                }
            }
        });
    }

    #[test]
    fn renders_fallback_on_panic() {
        with_scope(|cx| {
            let node = ErrorBoundary::new()
                .child(|_cx: Scope| -> Node {
                    panic!("boom");
                })
                .fallback(|msg: String| Node::Text(text(msg)))
                .build(cx);

            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert!(comp.child.is_text());
            }
        });
    }

    #[test]
    fn fallback_receives_error_message() {
        with_scope(|cx| {
            let node = ErrorBoundary::new()
                .child(|_cx: Scope| -> Node {
                    panic!("boom");
                })
                .fallback(|msg: String| Node::Text(text(msg)))
                .build(cx);

            if let Node::Component(comp) = &node {
                if let Node::Text(t) = &*comp.child {
                    assert_eq!(t.current_text(), "boom");
                } else {
                    panic!("expected Text node in fallback");
                }
            } else {
                panic!("expected Component node");
            }
        });
    }

    #[test]
    fn no_fallback_renders_empty() {
        with_scope(|cx| {
            let node = ErrorBoundary::new()
                .child(|_cx: Scope| -> Node {
                    panic!("boom");
                })
                .build(cx);

            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert!(comp.child.is_fragment());
                assert_eq!(comp.child.children().len(), 0);
            } else {
                panic!("expected Component node");
            }
        });
    }

    #[test]
    fn builds_component() {
        with_scope(|cx| {
            let node = ErrorBoundary::new().build(cx);
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "ErrorBoundary");
            } else {
                panic!("expected Component node");
            }
        });
    }

    #[test]
    fn does_not_catch_event_handler_panic() {
        // This test documents that ErrorBoundary only catches build-time panics.
        // Event handler panics propagate normally and are NOT caught by the boundary.
        with_scope(|cx| {
            let node = ErrorBoundary::new()
                .child(|_cx: Scope| {
                    // Build succeeds — no panic here
                    Node::Text(text("ok"))
                })
                .fallback(|msg: String| Node::Text(text(msg)))
                .build(cx);

            // The child built successfully
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert!(comp.child.is_text());
            }
        });
    }
}
