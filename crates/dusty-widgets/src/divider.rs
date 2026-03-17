use dusty_core::el;
use dusty_core::node::{ComponentNode, Node};
use dusty_core::view::View;
use dusty_reactive::Scope;
use dusty_style::{Color, Style};

/// Orientation for the divider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// A horizontal line (stretches width, 1px height).
    Horizontal,
    /// A vertical line (stretches height, 1px width).
    Vertical,
}

/// A visual separator line.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::Divider;
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = Divider::horizontal().build(cx);
///     assert!(node.is_component());
/// });
/// dispose_runtime();
/// ```
pub struct Divider {
    orientation: Orientation,
    user_style: Option<Style>,
}

/// Default divider background color (gray).
const DEFAULT_DIVIDER_COLOR: Color = Color {
    r: 0.8,
    g: 0.8,
    b: 0.8,
    a: 1.0,
};

impl Divider {
    /// Creates a horizontal divider (1px height, full width).
    #[must_use]
    pub const fn horizontal() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            user_style: None,
        }
    }

    /// Creates a vertical divider (1px width, full height).
    #[must_use]
    pub const fn vertical() -> Self {
        Self {
            orientation: Orientation::Vertical,
            user_style: None,
        }
    }

    /// Merges user styles on top of divider defaults.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.user_style = Some(style);
        self
    }
}

impl View for Divider {
    fn build(self, cx: Scope) -> Node {
        let base = match self.orientation {
            Orientation::Horizontal => Style {
                height: Some(1.0),
                flex_grow: Some(1.0),
                background: Some(DEFAULT_DIVIDER_COLOR),
                ..Style::default()
            },
            Orientation::Vertical => Style {
                width: Some(1.0),
                flex_grow: Some(1.0),
                background: Some(DEFAULT_DIVIDER_COLOR),
                ..Style::default()
            },
        };

        let merged = if let Some(user) = &self.user_style {
            base.merge(user)
        } else {
            base
        };

        let element = el("Divider", cx)
            .attr(
                "orientation",
                match self.orientation {
                    Orientation::Horizontal => "horizontal",
                    Orientation::Vertical => "vertical",
                },
            )
            .style(merged)
            .build_node();

        Node::Component(ComponentNode {
            name: "Divider",
            child: Box::new(element),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::Element;
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};

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
    fn horizontal_height_one() {
        with_scope(|cx| {
            let node = Divider::horizontal().build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>();
            assert_eq!(style.map(|s| s.height), Some(Some(1.0)));
        });
    }

    #[test]
    fn vertical_width_one() {
        with_scope(|cx| {
            let node = Divider::vertical().build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>();
            assert_eq!(style.map(|s| s.width), Some(Some(1.0)));
        });
    }

    #[test]
    fn default_background() {
        with_scope(|cx| {
            let node = Divider::horizontal().build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>();
            assert!(style.is_some());
            assert!(style.unwrap().background.is_some());
        });
    }

    #[test]
    fn style_overrides() {
        with_scope(|cx| {
            let node = Divider::horizontal()
                .style(Style {
                    background: Some(Color::BLACK),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.background, Some(Color::BLACK));
        });
    }

    #[test]
    fn stretches() {
        with_scope(|cx| {
            let node = Divider::horizontal().build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.flex_grow, Some(1.0));
        });
    }
}
