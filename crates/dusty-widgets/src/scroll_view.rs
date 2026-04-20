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
/// });
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

        let signal: Signal<(f64, f64)> = create_signal((0.0, 0.0));

        let axis_str = match self.axis {
            ScrollAxis::Vertical => "vertical",
            ScrollAxis::Horizontal => "horizontal",
            ScrollAxis::Both => "both",
        };

        let on_scroll = self.on_scroll;
        let scroll_axis = self.axis;
        let mut builder = el("ScrollView", cx)
            .attr("axis", axis_str)
            .style(merged)
            .data(signal)
            .on_scroll(move |ctx: &EventContext, e: &ScrollEvent| {
                let current = signal.get();
                let raw = (current.0 + e.delta_x, current.1 + e.delta_y);
                // Clamp: lower bound to 0.0, zero out irrelevant axis
                // TODO: clamp upper bound when content_size is available from layout
                let clamped = match scroll_axis {
                    ScrollAxis::Vertical => (0.0, raw.1.max(0.0)),
                    ScrollAxis::Horizontal => (raw.0.max(0.0), 0.0),
                    ScrollAxis::Both => (raw.0.max(0.0), raw.1.max(0.0)),
                };
                signal.set_if_changed(clamped);
                if let Some(ref cb) = on_scroll {
                    cb(e.delta_x, e.delta_y);
                }
                // Stop propagation so a wheel event consumed by an inner
                // ScrollView does not also scroll every ancestor ScrollView.
                ctx.stop_propagation();
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
    use dusty_style::Length;

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
    fn scroll_zeros_irrelevant_axis_vertical() {
        with_scope(|cx| {
            let node = ScrollView::new().axis(ScrollAxis::Vertical).build(cx);
            let el = extract_element(&node);

            // Simulate a scroll event
            let ctx = dusty_core::event::EventContext::new(vec![]);
            let event = dusty_core::event::ScrollEvent {
                delta_x: 10.0,
                delta_y: 20.0,
            };
            for handler in el.event_handlers() {
                if handler.name() == "scroll" {
                    handler.invoke(&ctx, &event);
                }
            }

            // Read the signal from custom data
            let sig = el
                .custom_data()
                .downcast_ref::<Signal<(f64, f64)>>()
                .unwrap();
            let offset = sig.get();
            assert!(
                (offset.0).abs() < f64::EPSILON,
                "x should be zeroed for vertical"
            );
            assert!((offset.1 - 20.0).abs() < f64::EPSILON, "y should be 20.0");
        });
    }

    #[test]
    fn zero_delta_scroll_does_not_notify_subscribers() {
        // After P0-#8: ScrollView uses set_if_changed. A scroll event whose
        // clamped result equals the current offset must not fire subscribers.
        use std::cell::Cell;
        use std::rc::Rc;

        with_scope(|cx| {
            let node = ScrollView::new().axis(ScrollAxis::Vertical).build(cx);
            let el = extract_element(&node);
            let sig = el
                .custom_data()
                .downcast_ref::<Signal<(f64, f64)>>()
                .unwrap();

            // Effect runs once on subscribe.
            let count = Rc::new(Cell::new(0u32));
            let count_for_effect = count.clone();
            let sig_copy = *sig;
            let _effect = dusty_reactive::create_effect(move || {
                let _ = sig_copy.get();
                count_for_effect.set(count_for_effect.get() + 1);
            });
            assert_eq!(count.get(), 1, "initial subscribe runs effect once");

            // Scroll with delta (0, 0) → clamped == current; should not notify.
            let ctx = dusty_core::event::EventContext::new(vec![]);
            let event = dusty_core::event::ScrollEvent {
                delta_x: 0.0,
                delta_y: 0.0,
            };
            for handler in el.event_handlers() {
                if handler.name() == "scroll" {
                    handler.invoke(&ctx, &event);
                }
            }
            assert_eq!(count.get(), 1, "no-op scroll must not notify subscribers");

            // A real scroll DOES notify.
            let event2 = dusty_core::event::ScrollEvent {
                delta_x: 0.0,
                delta_y: 5.0,
            };
            for handler in el.event_handlers() {
                if handler.name() == "scroll" {
                    handler.invoke(&ctx, &event2);
                }
            }
            assert_eq!(count.get(), 2, "real scroll notifies once");
        });
    }

    #[test]
    fn scroll_clamps_negative_offset() {
        with_scope(|cx| {
            let node = ScrollView::new().axis(ScrollAxis::Vertical).build(cx);
            let el = extract_element(&node);

            // Simulate a scroll with negative delta
            let ctx = dusty_core::event::EventContext::new(vec![]);
            let event = dusty_core::event::ScrollEvent {
                delta_x: 0.0,
                delta_y: -10.0,
            };
            for handler in el.event_handlers() {
                if handler.name() == "scroll" {
                    handler.invoke(&ctx, &event);
                }
            }

            let sig = el
                .custom_data()
                .downcast_ref::<Signal<(f64, f64)>>()
                .unwrap();
            let offset = sig.get();
            assert!(offset.1 >= 0.0, "scroll offset should not go negative");
        });
    }

    #[test]
    fn style_merges() {
        with_scope(|cx| {
            let node = ScrollView::new()
                .style(Style {
                    width: Some(Length::Px(300.0)),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.width, Some(Length::Px(300.0)));
            assert_eq!(style.overflow, Some(Overflow::Scroll));
        });
    }

    #[test]
    fn nested_scroll_inner_consumes_event() {
        // P0-#6: an inner ScrollView must call stop_propagation so the wheel
        // event is not also applied to every ancestor ScrollView.
        with_scope(|cx| {
            let outer_node = ScrollView::new()
                .axis(ScrollAxis::Vertical)
                .child(ScrollView::new().axis(ScrollAxis::Vertical))
                .build(cx);

            // Extract outer & inner elements + their offset signals.
            let outer_el = extract_element(&outer_node);
            let outer_sig = *outer_el
                .custom_data()
                .downcast_ref::<Signal<(f64, f64)>>()
                .unwrap();
            // Outer element's first child is the inner Component(Element).
            let inner_node = &outer_el.children()[0];
            let inner_el = extract_element(inner_node);
            let inner_sig = *inner_el
                .custom_data()
                .downcast_ref::<Signal<(f64, f64)>>()
                .unwrap();

            // Dispatch a scroll event targeted at the inner ScrollView.
            // Path: Component(outer) → Element(outer) → Component(inner) → Element(inner).
            let event = dusty_core::event::ScrollEvent {
                delta_x: 0.0,
                delta_y: 30.0,
            };
            let result = dusty_core::event::dispatch_event(&outer_node, &[0, 0, 0], &event);
            assert!(result.is_ok(), "dispatch should succeed");

            assert_eq!(
                inner_sig.get().1,
                30.0,
                "inner ScrollView should consume the event"
            );
            assert_eq!(
                outer_sig.get().1,
                0.0,
                "outer ScrollView must not also receive the event (stop_propagation)"
            );
        });
    }
}
