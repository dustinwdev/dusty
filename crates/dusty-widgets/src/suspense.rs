use dusty_core::node::{dynamic_node, ComponentNode, Node};
use dusty_core::view::View;
use dusty_reactive::Scope;

/// A container widget that shows a fallback while resources are loading,
/// then switches to the child content once ready.
///
/// `Suspense` is structurally similar to `Show` but with inverted semantics:
/// instead of showing content when a condition is true, it shows a loading
/// indicator until a readiness signal returns `true`.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::Suspense;
/// use dusty_core::view::View;
/// use dusty_core::node::{text, Node};
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = Suspense::new(|| true)
///         .child(|| Node::Text(text("Loaded!")))
///         .fallback(|| Node::Text(text("Loading...")))
///         .build(cx);
///     assert!(node.is_component());
/// });
/// dispose_runtime();
/// ```
pub struct Suspense {
    ready: Box<dyn Fn() -> bool>,
    child: Option<Box<dyn Fn() -> Node>>,
    fallback: Option<Box<dyn Fn() -> Node>>,
}

impl Suspense {
    /// Creates a new `Suspense` widget with the given readiness signal.
    ///
    /// When the closure returns `true`, the child content is displayed.
    /// When it returns `false`, the fallback is shown instead.
    #[must_use]
    pub fn new(ready: impl Fn() -> bool + 'static) -> Self {
        Self {
            ready: Box::new(ready),
            child: None,
            fallback: None,
        }
    }

    /// Sets the content shown when the readiness signal returns `true`.
    #[must_use]
    pub fn child(mut self, f: impl Fn() -> Node + 'static) -> Self {
        self.child = Some(Box::new(f));
        self
    }

    /// Sets the loading indicator shown while the readiness signal returns `false`.
    #[must_use]
    pub fn fallback(mut self, f: impl Fn() -> Node + 'static) -> Self {
        self.fallback = Some(Box::new(f));
        self
    }
}

impl View for Suspense {
    fn build(self, _cx: Scope) -> Node {
        let ready = self.ready;
        let child = self.child;
        let fallback = self.fallback;

        let dynamic = dynamic_node(move || {
            if (ready)() {
                child
                    .as_ref()
                    .map_or_else(|| Node::Fragment(vec![]), |f| f())
            } else {
                fallback
                    .as_ref()
                    .map_or_else(|| Node::Fragment(vec![]), |f| f())
            }
        });

        Node::Component(ComponentNode {
            name: "Suspense",
            child: Box::new(Node::Dynamic(dynamic)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::node::{text, Node};
    use dusty_core::view::View;
    use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime, Scope};

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx));
        dispose_runtime();
    }

    fn resolve_dynamic(node: &Node) -> Node {
        match node {
            Node::Component(comp) => match &*comp.child {
                Node::Dynamic(dn) => dn.current_node(),
                _ => panic!("expected Dynamic inside Component"),
            },
            _ => panic!("expected Component node"),
        }
    }

    #[test]
    fn shows_fallback_when_not_ready() {
        with_scope(|cx| {
            let node = Suspense::new(|| false)
                .child(|| Node::Text(text("Content")))
                .fallback(|| Node::Text(text("Loading...")))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_text());
            if let Node::Text(t) = resolved {
                assert_eq!(t.current_text(), "Loading...");
            }
        });
    }

    #[test]
    fn shows_child_when_ready() {
        with_scope(|cx| {
            let node = Suspense::new(|| true)
                .child(|| Node::Text(text("Content")))
                .fallback(|| Node::Text(text("Loading...")))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_text());
            if let Node::Text(t) = resolved {
                assert_eq!(t.current_text(), "Content");
            }
        });
    }

    #[test]
    fn reactive_ready_switches() {
        with_scope(|cx| {
            let ready = create_signal(false);
            let node = Suspense::new(move || ready.get())
                .child(|| Node::Text(text("Content")))
                .fallback(|| Node::Text(text("Loading...")))
                .build(cx);

            // Initially not ready -- should show fallback
            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_text());
            if let Node::Text(t) = &resolved {
                assert_eq!(t.current_text(), "Loading...");
            }

            // Set ready -- should now show child
            ready.set(true);
            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_text());
            if let Node::Text(t) = &resolved {
                assert_eq!(t.current_text(), "Content");
            }
        });
    }

    #[test]
    fn no_fallback_empty_while_loading() {
        with_scope(|cx| {
            let node = Suspense::new(|| false)
                .child(|| Node::Text(text("Content")))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_fragment());
            assert_eq!(resolved.children().len(), 0);
        });
    }

    #[test]
    fn builds_component() {
        with_scope(|cx| {
            let node = Suspense::new(|| true).build(cx);
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "Suspense");
            }
        });
    }
}
