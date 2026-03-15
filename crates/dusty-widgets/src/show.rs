use dusty_core::node::{dynamic_node, ComponentNode, Node};
use dusty_core::view::View;
use dusty_reactive::Scope;

/// A conditional container widget that renders a child when a reactive
/// condition is true, or an optional fallback when false.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, create_signal, dispose_runtime};
/// use dusty_widgets::Show;
/// use dusty_core::view::View;
/// use dusty_core::node::{text, Node};
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let visible = create_signal(true).unwrap();
///     let node = Show::new(move || visible.get().unwrap_or(false))
///         .child(|| Node::Text(text("shown")))
///         .fallback(|| Node::Text(text("hidden")))
///         .build(cx);
///     assert!(node.is_component());
/// }).unwrap();
/// dispose_runtime();
/// ```
pub struct Show {
    when: Box<dyn Fn() -> bool>,
    child: Option<Box<dyn Fn() -> Node>>,
    fallback: Option<Box<dyn Fn() -> Node>>,
}

impl Show {
    /// Creates a new `Show` widget with the given reactive condition.
    ///
    /// When the condition returns `true`, the child (if set) is rendered.
    /// When it returns `false`, the fallback (if set) is rendered.
    /// If neither child nor fallback is set for the active branch, an empty
    /// fragment is produced.
    #[must_use]
    pub fn new(when: impl Fn() -> bool + 'static) -> Self {
        Self {
            when: Box::new(when),
            child: None,
            fallback: None,
        }
    }

    /// Sets the child closure rendered when the condition is true.
    #[must_use]
    pub fn child(mut self, f: impl Fn() -> Node + 'static) -> Self {
        self.child = Some(Box::new(f));
        self
    }

    /// Sets the fallback closure rendered when the condition is false.
    #[must_use]
    pub fn fallback(mut self, f: impl Fn() -> Node + 'static) -> Self {
        self.fallback = Some(Box::new(f));
        self
    }
}

impl View for Show {
    fn build(self, _cx: Scope) -> Node {
        let when = self.when;
        let child = self.child;
        let fallback = self.fallback;

        let dynamic = dynamic_node(move || {
            if (when)() {
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
            name: "Show",
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

    /// Drop guard that ensures `dispose_runtime()` runs even if a test panics.
    /// This prevents runtime state from leaking into subsequent tests.
    struct RuntimeGuard;

    impl Drop for RuntimeGuard {
        fn drop(&mut self) {
            dispose_runtime();
        }
    }

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        let _guard = RuntimeGuard;
        create_scope(|cx| f(cx)).unwrap();
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
    fn true_renders_child() {
        with_scope(|cx| {
            let node = Show::new(|| true)
                .child(|| Node::Text(text("visible")))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_text());
            if let Node::Text(t) = resolved {
                assert_eq!(t.current_text(), "visible");
            }
        });
    }

    #[test]
    fn false_renders_fallback() {
        with_scope(|cx| {
            let node = Show::new(|| false)
                .child(|| Node::Text(text("child")))
                .fallback(|| Node::Text(text("fallback")))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_text());
            if let Node::Text(t) = resolved {
                assert_eq!(t.current_text(), "fallback");
            }
        });
    }

    #[test]
    fn false_no_fallback_empty() {
        with_scope(|cx| {
            let node = Show::new(|| false)
                .child(|| Node::Text(text("child")))
                .build(cx);

            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_fragment());
            assert_eq!(resolved.children().len(), 0);
        });
    }

    #[test]
    fn reactive_condition_switches() {
        with_scope(|cx| {
            let sig = create_signal(true).unwrap();
            let node = Show::new(move || sig.get().unwrap_or(false))
                .child(|| Node::Text(text("on")))
                .fallback(|| Node::Text(text("off")))
                .build(cx);

            // Initially true — should render child
            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_text());
            if let Node::Text(t) = resolved {
                assert_eq!(t.current_text(), "on");
            }

            // Flip to false — should render fallback
            let _ = sig.set(false);
            let resolved = resolve_dynamic(&node);
            assert!(resolved.is_text());
            if let Node::Text(t) = resolved {
                assert_eq!(t.current_text(), "off");
            }
        });
    }

    #[test]
    fn builds_component() {
        with_scope(|cx| {
            let node = Show::new(|| true).build(cx);
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "Show");
            }
        });
    }
}
