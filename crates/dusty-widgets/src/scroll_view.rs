use dusty_core::el;
use dusty_core::event::{EventContext, ScrollEvent};
use dusty_core::node::{ComponentNode, Node};
use dusty_core::view::{IntoView, View};
use dusty_core::view_seq::ViewSeq;
use dusty_reactive::{create_signal, Scope, Signal};
use dusty_style::{Overflow, Style};

/// Scroll axis for a [`ScrollView`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollAxis {
    /// Scrolls vertically only (default).
    #[default]
    Vertical,
    /// Scrolls horizontally only.
    Horizontal,
    /// Scrolls in both directions.
    Both,
}

/// A scrollable container widget.
///
/// Wraps child content in a scrollable region with configurable axis,
/// scroll offset tracking via a reactive signal, and an optional
/// user-provided scroll callback.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::ScrollView;
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = ScrollView::new().build(cx);
///     assert!(node.is_component());
/// }).unwrap();
/// dispose_runtime();
/// ```
pub struct ScrollView {
    axis: ScrollAxis,
    child: Option<Box<dyn FnOnce(Scope) -> Node>>,
    children: Option<Box<dyn FnOnce(Scope) -> Vec<Node>>>,
    user_style: Option<Style>,
    on_scroll: Option<Box<dyn Fn(f64, f64)>>,
}

impl ScrollView {
    /// Creates a new scroll view with default vertical axis.
    #[must_use]
    pub fn new() -> Self {
        Self {
            axis: ScrollAxis::default(),
            child: None,
            children: None,
            user_style: None,
            on_scroll: None,
        }
    }

    /// Sets the scroll axis.
    #[must_use]
    pub const fn axis(mut self, axis: ScrollAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Sets a single child view.
    #[must_use]
    pub fn child(mut self, child: impl IntoView + 'static) -> Self {
        self.child = Some(Box::new(move |cx| child.into_view(cx)));
        self
    }

    /// Sets multiple children from a view sequence.
    #[must_use]
    pub fn children(mut self, seq: impl ViewSeq + 'static) -> Self {
        self.children = Some(Box::new(move |cx| seq.build_seq(cx)));
        self
    }

    /// Merges user styles on top of scroll view defaults.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.user_style = Some(style);
        self
    }

    /// Registers a scroll callback receiving `(delta_x, delta_y)`.
    #[must_use]
    pub fn on_scroll(mut self, handler: impl Fn(f64, f64) + 'static) -> Self {
        self.on_scroll = Some(Box::new(handler));
        self
    }
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ScrollView {
    fn build(self, cx: Scope) -> Node {
        let base = Style {
            overflow: Some(Overflow::Scroll),
            ..Style::default()
        };

        let merged = if let Some(user) = &self.user_style {
            base.merge(user)
        } else {
            base
        };

        let signal: Signal<(f64, f64)> = match create_signal((0.0, 0.0)) {
            Ok(s) => s,
            Err(_) => return Node::Fragment(vec![]),
        };

        let axis_str = match self.axis {
            ScrollAxis::Vertical => "vertical",
            ScrollAxis::Horizontal => "horizontal",
            ScrollAxis::Both => "both",
        };

        let on_scroll = self.on_scroll;
        let mut builder = el("ScrollView", cx)
            .attr("axis", axis_str)
            .style(merged)
            .data(signal)
            .on_scroll(move |_ctx: &EventContext, e: &ScrollEvent| {
                let current = signal.get().unwrap_or((0.0, 0.0));
                let new_offset = (current.0 + e.delta_x, current.1 + e.delta_y);
                let _ = signal.set(new_offset);
                if let Some(ref cb) = on_scroll {
                    cb(e.delta_x, e.delta_y);
                }
            });

        if let Some(child_fn) = self.child {
            let child_node = child_fn(cx);
            builder = builder.child_node(child_node);
        }

        if let Some(children_fn) = self.children {
            let child_nodes = children_fn(cx);
            for child_node in child_nodes {
                builder = builder.child_node(child_node);
            }
        }

        let element = builder.build_node();

        Node::Component(ComponentNode {
            name: "ScrollView",
            child: Box::new(element),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::{AttributeValue, Element};
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx)).unwrap();
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
    fn builds_component() {
        with_scope(|cx| {
            let node = ScrollView::new().build(cx);
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "ScrollView");
            }
        });
    }

    #[test]
    fn default_vertical_axis() {
        with_scope(|cx| {
            let node = ScrollView::new().build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("axis"),
                Some(&AttributeValue::String("vertical".into()))
            );
        });
    }

    #[test]
    fn overflow_scroll_style() {
        with_scope(|cx| {
            let node = ScrollView::new().build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.overflow, Some(Overflow::Scroll));
        });
    }

    #[test]
    fn registers_scroll_handler() {
        with_scope(|cx| {
            let node = ScrollView::new().build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "scroll"));
        });
    }

    #[test]
    fn stores_scroll_offset_signal() {
        with_scope(|cx| {
            let node = ScrollView::new().build(cx);
            let el = extract_element(&node);
            assert!(el
                .custom_data()
                .downcast_ref::<Signal<(f64, f64)>>()
                .is_some());
        });
    }

    #[test]
    fn horizontal_axis() {
        with_scope(|cx| {
            let node = ScrollView::new().axis(ScrollAxis::Horizontal).build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("axis"),
                Some(&AttributeValue::String("horizontal".into()))
            );
        });
    }

    #[test]
    fn both_axis() {
        with_scope(|cx| {
            let node = ScrollView::new().axis(ScrollAxis::Both).build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("axis"),
                Some(&AttributeValue::String("both".into()))
            );
        });
    }

    #[test]
    fn wraps_children() {
        with_scope(|cx| {
            let node = ScrollView::new().child("hello").build(cx);
            let el = extract_element(&node);
            assert_eq!(el.children().len(), 1);
        });
    }

    #[test]
    fn style_merges() {
        with_scope(|cx| {
            let node = ScrollView::new()
                .style(Style {
                    width: Some(300.0),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.width, Some(300.0));
            assert_eq!(style.overflow, Some(Overflow::Scroll));
        });
    }
}
