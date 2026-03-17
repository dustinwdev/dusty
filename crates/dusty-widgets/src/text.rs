use dusty_core::el;
use dusty_core::node::{text, text_dynamic, ComponentNode, Node, TextNode};
use dusty_core::view::View;
use dusty_reactive::Scope;
use dusty_style::Style;

/// A text display widget.
///
/// Supports both static and reactive (dynamic) text content.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::Text;
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = Text::new("Hello, world!").build(cx);
///     assert!(node.is_component());
/// });
/// dispose_runtime();
/// ```
pub struct Text {
    content: TextContent,
    user_style: Option<Style>,
}

enum TextContent {
    Static(String),
    Dynamic(Box<dyn Fn() -> String>),
}

impl Text {
    /// Creates a text widget with static content.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self {
            content: TextContent::Static(s.into()),
            user_style: None,
        }
    }

    /// Creates a text widget with reactive (dynamic) content.
    ///
    /// The closure is called to compute the current text value. When used
    /// with signals, the renderer wraps this in an effect for automatic updates.
    #[must_use]
    pub fn dynamic(f: impl Fn() -> String + 'static) -> Self {
        Self {
            content: TextContent::Dynamic(Box::new(f)),
            user_style: None,
        }
    }

    /// Merges user styles on top of text widget defaults.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.user_style = Some(style);
        self
    }
}

impl View for Text {
    fn build(self, cx: Scope) -> Node {
        let base = Style::default();

        let merged = if let Some(user) = &self.user_style {
            base.merge(user)
        } else {
            base
        };

        let text_child: TextNode = match self.content {
            TextContent::Static(s) => text(s),
            TextContent::Dynamic(f) => text_dynamic(f),
        };

        let element = el("Text", cx)
            .style(merged)
            .child_text(text_child)
            .build_node();

        Node::Component(ComponentNode {
            name: "Text",
            child: Box::new(element),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::node::TextContent as NodeTextContent;
    use dusty_core::Element;
    use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime};

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
    fn static_text_child() {
        with_scope(|cx| {
            let node = Text::new("hello").build(cx);
            let el = extract_element(&node);
            assert_eq!(el.children().len(), 1);
            if let Node::Text(text_node) = &el.children()[0] {
                assert_eq!(text_node.current_text(), "hello");
            } else {
                panic!("expected Text child");
            }
        });
    }

    #[test]
    fn dynamic_reads_signal() {
        with_scope(|cx| {
            let count = create_signal(42i32);
            let node = Text::dynamic(move || format!("{}", count.get())).build(cx);
            let el = extract_element(&node);
            assert_eq!(el.children().len(), 1);
            if let Node::Text(text_node) = &el.children()[0] {
                assert_eq!(text_node.current_text(), "42");
                assert!(matches!(text_node.content, NodeTextContent::Dynamic(_)));
            } else {
                panic!("expected Text child");
            }
        });
    }

    #[test]
    fn style_applied() {
        with_scope(|cx| {
            let node = Text::new("styled")
                .style(Style {
                    foreground: Some(dusty_style::Color::BLACK),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.foreground, Some(dusty_style::Color::BLACK));
        });
    }

    #[test]
    fn style_merges() {
        with_scope(|cx| {
            let node = Text::new("test")
                .style(Style {
                    width: Some(100.0),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.width, Some(100.0));
        });
    }
}
