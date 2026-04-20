use dusty_core::el;
use dusty_core::node::{ComponentNode, Node};
use dusty_core::view::View;
use dusty_reactive::Scope;
use dusty_style::Style;

/// A flexible space widget that fills available space.
///
/// By default, `Spacer` has `flex_grow: 1.0` and no visual appearance.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::Spacer;
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = Spacer::new().build(cx);
///     assert!(node.is_component());
/// });
/// dispose_runtime();
/// ```
pub struct Spacer {
    user_style: Option<Style>,
}

impl Spacer {
    /// Creates a new spacer with default flex-grow.
    #[must_use]
    pub const fn new() -> Self {
        Self { user_style: None }
    }

    /// Merges user styles on top of the spacer defaults.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.user_style = Some(style);
        self
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Spacer {
    fn build(self, cx: Scope) -> Node {
        let base = Style {
            flex_grow: Some(1.0),
            ..Style::default()
        };

        let merged = if let Some(user) = &self.user_style {
            base.merge(user)
        } else {
            base
        };

        let element = el("Spacer", cx).style(merged).build_node();

        Node::Component(ComponentNode {
            name: "Spacer",
            child: Box::new(element),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::Element;
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};
    use dusty_style::{Length, Style};

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx));
        dispose_runtime();
    }

    fn extract_element(node: &Node) -> &Element {
        match node {
            Node::Component(comp) => match &*comp.child {
                Node::Element(el) => el,
                _ => panic!("expected Element inside Component"),
            },
            _ => panic!("expected Component node"),
        }
    }

    #[test]
    fn spacer_builds_component() {
        with_scope(|cx| {
            let node = Spacer::new().build(cx);
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "Spacer");
            }
        });
    }

    #[test]
    fn spacer_has_flex_grow() {
        with_scope(|cx| {
            let node = Spacer::new().build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>();
            assert!(style.is_some());
            assert_eq!(style.map(|s| s.flex_grow), Some(Some(1.0)));
        });
    }

    #[test]
    fn spacer_no_background() {
        with_scope(|cx| {
            let node = Spacer::new().build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>();
            assert_eq!(style.map(|s| s.background), Some(None));
        });
    }

    #[test]
    fn spacer_style_merges() {
        with_scope(|cx| {
            let node = Spacer::new()
                .style(Style {
                    width: Some(Length::Px(20.0)),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>();
            assert!(style.is_some());
            let s = style.unwrap();
            assert_eq!(s.width, Some(Length::Px(20.0)));
            // flex_grow should still be present from base
            assert_eq!(s.flex_grow, Some(1.0));
        });
    }
}
