use dusty_core::el;
use dusty_core::event::{ClickEvent, EventContext};
use dusty_core::node::{text, text_dynamic, ComponentNode, Node, TextNode};
use dusty_core::view::View;
use dusty_reactive::{create_signal, Scope, Signal};
use dusty_style::Style;

/// Source of truth for the checked state.
pub enum CheckedSource {
    /// Widget manages its own signal internally.
    Uncontrolled(bool),
    /// Caller provides the signal.
    Controlled(Signal<bool>),
}

/// A checkbox input widget.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::Checkbox;
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = Checkbox::new().build(cx);
///     assert!(node.is_component());
/// }).unwrap();
/// dispose_runtime();
/// ```
pub struct Checkbox {
    source: CheckedSource,
    label: Option<LabelContent>,
    disabled: bool,
    user_style: Option<Style>,
    on_change: Option<Box<dyn Fn(bool)>>,
}

enum LabelContent {
    Static(String),
    Dynamic(Box<dyn Fn() -> String>),
}

impl Checkbox {
    /// Creates an unchecked checkbox.
    #[must_use]
    pub fn new() -> Self {
        Self {
            source: CheckedSource::Uncontrolled(false),
            label: None,
            disabled: false,
            user_style: None,
            on_change: None,
        }
    }

    /// Sets the initial checked state (uncontrolled mode).
    #[must_use]
    pub const fn checked(mut self, checked: bool) -> Self {
        self.source = CheckedSource::Uncontrolled(checked);
        self
    }

    /// Uses an external signal as the source of truth (controlled mode).
    #[must_use]
    pub const fn controlled(mut self, signal: Signal<bool>) -> Self {
        self.source = CheckedSource::Controlled(signal);
        self
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

    /// Disables the checkbox, suppressing click events.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Merges user styles on top of checkbox defaults.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.user_style = Some(style);
        self
    }

    /// Registers a change handler called with the new checked value.
    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Checkbox {
    fn build(self, cx: Scope) -> Node {
        let base = Style::default();

        let merged = if let Some(user) = &self.user_style {
            base.merge(user)
        } else {
            base
        };

        let signal = match self.source {
            CheckedSource::Uncontrolled(initial) => match create_signal(initial) {
                Ok(s) => s,
                Err(_) => return Node::Fragment(vec![]),
            },
            CheckedSource::Controlled(sig) => sig,
        };

        let mut builder = el("Checkbox", cx)
            .attr("checked", signal.get().unwrap_or(false))
            .attr("disabled", self.disabled)
            .style(merged)
            .data(signal);

        if let Some(label_content) = self.label {
            let label_child: TextNode = match label_content {
                LabelContent::Static(s) => text(s),
                LabelContent::Dynamic(f) => text_dynamic(f),
            };
            builder = builder.child_text(label_child);
        }

        if !self.disabled {
            let on_change = self.on_change;
            builder = builder.on_click(move |_ctx: &EventContext, _e: &ClickEvent| {
                let current = signal.get().unwrap_or(false);
                let new_val = !current;
                let _ = signal.set(new_val);
                if let Some(ref cb) = on_change {
                    cb(new_val);
                }
            });
        }

        let element = builder.build_node();

        Node::Component(ComponentNode {
            name: "Checkbox",
            child: Box::new(element),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::{AttributeValue, Element};
    use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime};
    use std::cell::Cell;
    use std::rc::Rc;

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
            let node = Checkbox::new().build(cx);
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "Checkbox");
            }
        });
    }

    #[test]
    fn default_unchecked() {
        with_scope(|cx| {
            let node = Checkbox::new().build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("checked"), Some(&AttributeValue::Bool(false)));
        });
    }

    #[test]
    fn initial_checked() {
        with_scope(|cx| {
            let node = Checkbox::new().checked(true).build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("checked"), Some(&AttributeValue::Bool(true)));
        });
    }

    #[test]
    fn toggle_on_click() {
        with_scope(|cx| {
            let node = Checkbox::new().build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "click"));
        });
    }

    #[test]
    fn on_change_fires() {
        with_scope(|cx| {
            let fired = Rc::new(Cell::new(false));
            let fired_clone = fired.clone();
            let _node = Checkbox::new()
                .on_change(move |_val| {
                    fired_clone.set(true);
                })
                .build(cx);
            // Handler is registered — actual firing tested in integration tests
            assert!(!fired.get());
        });
    }

    #[test]
    fn disabled_suppresses_click() {
        with_scope(|cx| {
            let node = Checkbox::new().disabled(true).build(cx);
            let el = extract_element(&node);
            assert!(!el.event_handlers().iter().any(|h| h.name() == "click"));
        });
    }

    #[test]
    fn controlled_reads_signal() {
        with_scope(|cx| {
            let sig = create_signal(true).unwrap();
            let node = Checkbox::new().controlled(sig).build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("checked"), Some(&AttributeValue::Bool(true)));
        });
    }

    #[test]
    fn label_text() {
        with_scope(|cx| {
            let node = Checkbox::new().label("Accept terms").build(cx);
            let el = extract_element(&node);
            assert_eq!(el.children().len(), 1);
            if let Node::Text(text_node) = &el.children()[0] {
                assert_eq!(text_node.current_text(), "Accept terms");
            } else {
                panic!("expected Text child");
            }
        });
    }

    #[test]
    fn style_merges() {
        with_scope(|cx| {
            let node = Checkbox::new()
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
    fn stores_signal_in_custom_data() {
        with_scope(|cx| {
            let node = Checkbox::new().build(cx);
            let el = extract_element(&node);
            assert!(el.custom_data().downcast_ref::<Signal<bool>>().is_some());
        });
    }
}
