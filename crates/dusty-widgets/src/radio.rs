use dusty_core::el;
use dusty_core::event::{ClickEvent, EventContext};
use dusty_core::node::{text, text_dynamic, ComponentNode, Node, TextNode};
use dusty_core::view::View;
use dusty_reactive::{Scope, Signal};
use dusty_style::Style;

use crate::common::LabelContent;

/// A radio button widget.
///
/// Multiple `Radio` widgets share a `Signal<V>` to form a group. Selecting
/// one automatically deselects the others since they all read the same signal.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, create_signal, dispose_runtime};
/// use dusty_widgets::Radio;
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let choice = create_signal("a".to_string());
///     let node = Radio::new("a".to_string(), choice).build(cx);
///     assert!(node.is_component());
/// });
/// dispose_runtime();
/// ```
#[allow(clippy::type_complexity)]
pub struct Radio<V: PartialEq + Clone + 'static> {
    value: V,
    group: Signal<V>,
    label: Option<LabelContent>,
    disabled: bool,
    user_style: Option<Style>,
    on_select: Option<Box<dyn Fn(&V)>>,
}

impl<V: PartialEq + Clone + 'static> Radio<V> {
    /// Creates a radio button for the given value within a group signal.
    #[must_use]
    pub fn new(value: V, group: Signal<V>) -> Self {
        Self {
            value,
            group,
            label: None,
            disabled: false,
            user_style: None,
            on_select: None,
        }
    }

    /// Sets a static label.
    #[must_use]
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(LabelContent::Static(text.into()));
        self
    }

    /// Sets a reactive label.
    #[must_use]
    pub fn label_dynamic(mut self, f: impl Fn() -> String + 'static) -> Self {
        self.label = Some(LabelContent::Dynamic(Box::new(f)));
        self
    }

    /// Disables the radio button, suppressing click events.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Merges user styles on top of radio defaults.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.user_style = Some(style);
        self
    }

    /// Registers a selection handler called with the selected value.
    #[must_use]
    pub fn on_select(mut self, handler: impl Fn(&V) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl<V: PartialEq + Clone + 'static> View for Radio<V> {
    fn build(self, cx: Scope) -> Node {
        let base = Style::default();

        let merged = if let Some(user) = &self.user_style {
            base.merge(user)
        } else {
            base
        };

        let is_checked = self.group.with(|g| *g == self.value);

        let mut builder = el("Radio", cx)
            .attr("checked", is_checked)
            .attr("disabled", self.disabled)
            .style(merged)
            .data(self.group);

        if let Some(label_content) = self.label {
            let label_child: TextNode = match label_content {
                LabelContent::Static(s) => text(s),
                LabelContent::Dynamic(f) => text_dynamic(f),
            };
            let label_str = label_child.current_text().into_owned();
            builder = builder.attr("label", label_str).child_text(label_child);
        }

        if !self.disabled {
            let value = self.value;
            let group = self.group;
            let on_select = self.on_select;
            builder = builder.on_click(move |_ctx: &EventContext, _e: &ClickEvent| {
                group.set(value.clone());
                if let Some(ref cb) = on_select {
                    cb(&value);
                }
            });
        }

        let element = builder.build_node();

        Node::Component(ComponentNode {
            name: "Radio",
            child: Box::new(element),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{extract_element, with_scope};
    use dusty_core::AttributeValue;
    use dusty_reactive::create_signal;

    #[test]
    fn builds_component() {
        with_scope(|cx| {
            let group = create_signal("a".to_string());
            let node = Radio::new("a".to_string(), group).build(cx);
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "Radio");
            }
        });
    }

    #[test]
    fn unchecked_when_group_differs() {
        with_scope(|cx| {
            let group = create_signal("b".to_string());
            let node = Radio::new("a".to_string(), group).build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("checked"), Some(&AttributeValue::Bool(false)));
        });
    }

    #[test]
    fn checked_when_group_matches() {
        with_scope(|cx| {
            let group = create_signal("a".to_string());
            let node = Radio::new("a".to_string(), group).build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("checked"), Some(&AttributeValue::Bool(true)));
        });
    }

    #[test]
    fn disabled_suppresses_click() {
        with_scope(|cx| {
            let group = create_signal("a".to_string());
            let node = Radio::new("b".to_string(), group).disabled(true).build(cx);
            let el = extract_element(&node);
            assert!(!el.event_handlers().iter().any(|h| h.name() == "click"));
        });
    }

    #[test]
    fn label_text() {
        with_scope(|cx| {
            let group = create_signal(0i32);
            let node = Radio::new(1, group).label("Option A").build(cx);
            let el = extract_element(&node);
            assert_eq!(el.children().len(), 1);
            if let Node::Text(text_node) = &el.children()[0] {
                assert_eq!(text_node.current_text(), "Option A");
            } else {
                panic!("expected Text child");
            }
        });
    }

    #[test]
    fn style_merges() {
        with_scope(|cx| {
            let group = create_signal(0i32);
            let node = Radio::new(1, group)
                .style(Style {
                    width: Some(20.0),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.width, Some(20.0));
        });
    }

    #[test]
    fn label_sets_label_attr() {
        with_scope(|cx| {
            let group = create_signal(0i32);
            let node = Radio::new(1, group).label("Option A").build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("label"),
                Some(&AttributeValue::String("Option A".into()))
            );
        });
    }

    #[test]
    fn click_handler_registered_when_enabled() {
        with_scope(|cx| {
            let group = create_signal(0i32);
            let node = Radio::new(1, group).build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "click"));
        });
    }
}
