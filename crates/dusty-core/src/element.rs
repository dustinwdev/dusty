use std::any::Any;
use std::fmt;

use dusty_reactive::Scope;

use crate::event::{
    BlurEvent, ClickEvent, DragEvent, Event, EventContext, FocusEvent, HoverEvent, KeyDownEvent,
    KeyUpEvent, ScrollEvent, TextInputEvent,
};
use crate::node::{Node, TextNode};
use crate::view::IntoView;
use crate::view_seq::ViewSeq;

/// An element node in the view tree.
///
/// Elements have a name (e.g. `"Button"`, `"Row"`), attributes, styles,
/// event handlers, and children.
pub struct Element {
    name: &'static str,
    attributes: Vec<(&'static str, AttributeValue)>,
    style: Box<dyn Any>,
    custom_data: Box<dyn Any>,
    event_handlers: Vec<EventHandler>,
    children: Vec<Node>,
}

impl Element {
    /// Creates a new element with the given name and no children/attributes.
    pub(crate) fn new(name: &'static str) -> Self {
        Self {
            name,
            attributes: Vec::new(),
            style: Box::new(()),
            custom_data: Box::new(()),
            event_handlers: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Returns the element's name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Looks up an attribute by name.
    #[must_use]
    pub fn attr(&self, name: &str) -> Option<&AttributeValue> {
        self.attributes
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| v)
    }

    /// Returns all attributes.
    #[must_use]
    pub fn attributes(&self) -> &[(&'static str, AttributeValue)] {
        &self.attributes
    }

    /// Returns a reference to the style data.
    #[must_use]
    pub fn style(&self) -> &dyn Any {
        &*self.style
    }

    /// Downcasts the style data to a concrete type, returning `None` if the
    /// type does not match.
    ///
    /// The concrete type is typically `dusty_style::Style`, but elements may
    /// store any `'static` type via [`ElementBuilder::style`].
    ///
    /// # Example
    ///
    /// ```
    /// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
    /// use dusty_core::el;
    ///
    /// initialize_runtime();
    /// create_scope(|cx| {
    ///     let elem = el("Box", cx).style(42u32).build();
    ///     assert_eq!(elem.style_as::<u32>(), Some(&42));
    ///     assert_eq!(elem.style_as::<String>(), None);
    /// }).unwrap();
    /// dispose_runtime();
    /// ```
    #[must_use]
    pub fn style_as<T: Any>(&self) -> Option<&T> {
        self.style.downcast_ref::<T>()
    }

    /// Returns a reference to the custom data slot.
    #[must_use]
    pub fn custom_data(&self) -> &dyn Any {
        &*self.custom_data
    }

    /// Returns the event handlers.
    #[must_use]
    pub fn event_handlers(&self) -> &[EventHandler] {
        &self.event_handlers
    }

    /// Returns the children.
    #[must_use]
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// Returns a mutable reference to the children as a slice.
    pub fn children_mut(&mut self) -> &mut [Node] {
        &mut self.children
    }

    /// Adds a child node.
    pub fn push_child(&mut self, child: Node) {
        self.children.push(child);
    }
}

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Element")
            .field("name", &self.name)
            .field("attributes", &self.attributes)
            .field("event_handlers", &self.event_handlers)
            .field("children", &self.children)
            .finish_non_exhaustive()
    }
}

/// The value of an element attribute.
#[derive(Debug, Clone)]
pub enum AttributeValue {
    /// A string value.
    String(String),
    /// An integer value.
    Int(i64),
    /// A floating-point value.
    Float(f64),
    /// A boolean value.
    Bool(bool),
}

impl PartialEq for AttributeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a.total_cmp(b).is_eq(),
            (Self::Bool(a), Self::Bool(b)) => a == b,
            _ => false,
        }
    }
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<i64> for AttributeValue {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<f64> for AttributeValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<i32> for AttributeValue {
    fn from(v: i32) -> Self {
        Self::Int(i64::from(v))
    }
}

impl From<u32> for AttributeValue {
    fn from(v: u32) -> Self {
        Self::Int(i64::from(v))
    }
}

impl From<f32> for AttributeValue {
    fn from(v: f32) -> Self {
        Self::Float(f64::from(v))
    }
}

impl From<usize> for AttributeValue {
    fn from(v: usize) -> Self {
        #[allow(clippy::cast_possible_wrap)]
        Self::Int(v as i64)
    }
}

impl From<bool> for AttributeValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

/// Type alias for event handler callback functions.
type EventCallback = Box<dyn Fn(&EventContext, &dyn Any)>;

/// An event handler attached to an element.
pub struct EventHandler {
    name: &'static str,
    callback: EventCallback,
}

impl EventHandler {
    /// Returns the event name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Invokes the handler with the given context and event data.
    pub fn invoke(&self, ctx: &EventContext, event: &dyn Any) {
        (self.callback)(ctx, event);
    }
}

impl fmt::Debug for EventHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventHandler")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

/// Builder for constructing elements with a fluent API.
///
/// Created via [`el()`]. Captures the [`Scope`] at creation so child
/// views can be built without passing the scope explicitly.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_scope, dispose_runtime};
/// use dusty_core::{el, text};
///
/// initialize_runtime();
/// let scope = create_scope(|cx| {
///     let node = el("Row", cx)
///         .child(text("hello"))
///         .attr("gap", 8i64)
///         .build_node();
///     assert!(node.is_element());
/// }).unwrap();
/// dispose_runtime();
/// ```
#[must_use]
pub struct ElementBuilder {
    element: Element,
    cx: Scope,
}

impl ElementBuilder {
    /// Adds a child view. The child is built using the builder's scope.
    pub fn child(mut self, child: impl IntoView) -> Self {
        let node = child.into_view(self.cx);
        self.element.children.push(node);
        self
    }

    /// Adds multiple children from a [`ViewSeq`].
    pub fn children(mut self, seq: impl ViewSeq) -> Self {
        let mut nodes = seq.build_seq(self.cx);
        self.element.children.append(&mut nodes);
        self
    }

    /// Adds a pre-built node as a child without requiring a scope.
    pub fn child_node(mut self, node: Node) -> Self {
        self.element.children.push(node);
        self
    }

    /// Adds a pre-built text node as a child.
    pub fn child_text(mut self, text: TextNode) -> Self {
        self.element.children.push(Node::Text(text));
        self
    }

    /// Sets an attribute on the element.
    ///
    /// If an attribute with the same name already exists, its value is updated.
    pub fn attr(mut self, name: &'static str, value: impl Into<AttributeValue>) -> Self {
        let value = value.into();
        if let Some(existing) = self.element.attributes.iter_mut().find(|(n, _)| *n == name) {
            existing.1 = value;
        } else {
            self.element.attributes.push((name, value));
        }
        self
    }

    /// Registers an untyped event handler on the element.
    pub fn on(
        mut self,
        event: &'static str,
        handler: impl Fn(&EventContext, &dyn Any) + 'static,
    ) -> Self {
        self.element.event_handlers.push(EventHandler {
            name: event,
            callback: Box::new(handler),
        });
        self
    }

    /// Registers a typed event handler. The handler receives the concrete event type.
    fn on_event<E: Event>(mut self, handler: impl Fn(&EventContext, &E) + 'static) -> Self {
        self.element.event_handlers.push(EventHandler {
            name: E::event_name(),
            callback: Box::new(move |ctx, any| {
                if let Some(event) = any.downcast_ref::<E>() {
                    handler(ctx, event);
                } else {
                    debug_assert!(
                        false,
                        "event type mismatch: handler for '{}' received wrong type",
                        E::event_name()
                    );
                }
            }),
        });
        self
    }

    /// Registers a click event handler.
    pub fn on_click(self, handler: impl Fn(&EventContext, &ClickEvent) + 'static) -> Self {
        self.on_event(handler)
    }

    /// Registers a hover event handler.
    pub fn on_hover(self, handler: impl Fn(&EventContext, &HoverEvent) + 'static) -> Self {
        self.on_event(handler)
    }

    /// Registers a key-down event handler.
    pub fn on_key_down(self, handler: impl Fn(&EventContext, &KeyDownEvent) + 'static) -> Self {
        self.on_event(handler)
    }

    /// Registers a key-up event handler.
    pub fn on_key_up(self, handler: impl Fn(&EventContext, &KeyUpEvent) + 'static) -> Self {
        self.on_event(handler)
    }

    /// Registers a focus event handler.
    pub fn on_focus(self, handler: impl Fn(&EventContext, &FocusEvent) + 'static) -> Self {
        self.on_event(handler)
    }

    /// Registers a blur event handler.
    pub fn on_blur(self, handler: impl Fn(&EventContext, &BlurEvent) + 'static) -> Self {
        self.on_event(handler)
    }

    /// Registers a scroll event handler.
    pub fn on_scroll(self, handler: impl Fn(&EventContext, &ScrollEvent) + 'static) -> Self {
        self.on_event(handler)
    }

    /// Registers a text input event handler.
    pub fn on_text_input(self, handler: impl Fn(&EventContext, &TextInputEvent) + 'static) -> Self {
        self.on_event(handler)
    }

    /// Registers a drag event handler.
    pub fn on_drag(self, handler: impl Fn(&EventContext, &DragEvent) + 'static) -> Self {
        self.on_event(handler)
    }

    /// Sets the style data on the element.
    pub fn style(mut self, style: impl Any) -> Self {
        self.element.style = Box::new(style);
        self
    }

    /// Sets custom data on the element.
    pub fn data(mut self, data: impl Any) -> Self {
        self.element.custom_data = Box::new(data);
        self
    }

    /// Consumes the builder and returns the [`Element`].
    #[must_use]
    pub fn build(self) -> Element {
        self.element
    }

    /// Consumes the builder and returns a [`Node::Element`].
    #[must_use]
    pub fn build_node(self) -> Node {
        Node::Element(self.element)
    }
}

/// Creates an [`ElementBuilder`] for the given element name.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_core::el;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = el("Button", cx).attr("label", "OK").build_node();
///     assert!(node.is_element());
/// }).unwrap();
/// dispose_runtime();
/// ```
pub fn el(name: &'static str, cx: Scope) -> ElementBuilder {
    ElementBuilder {
        element: Element::new(name),
        cx,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::text;
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};

    /// Drop guard that ensures `dispose_runtime()` runs even if a test panics.
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

    #[test]
    fn element_name() {
        let elem = Element::new("Button");
        assert_eq!(elem.name(), "Button");
    }

    #[test]
    fn element_attributes() {
        with_scope(|cx| {
            let elem = el("Input", cx)
                .attr("placeholder", "type here")
                .attr("max_length", 100i64)
                .attr("disabled", true)
                .build();

            assert_eq!(
                elem.attr("placeholder"),
                Some(&AttributeValue::String("type here".into()))
            );
            assert_eq!(elem.attr("max_length"), Some(&AttributeValue::Int(100)));
            assert_eq!(elem.attr("disabled"), Some(&AttributeValue::Bool(true)));
            assert_eq!(elem.attr("nonexistent"), None);
        });
    }

    #[test]
    fn element_children() {
        with_scope(|cx| {
            let elem = el("Row", cx).child(text("a")).child(text("b")).build();
            assert_eq!(elem.children().len(), 2);
        });
    }

    #[test]
    fn element_children_mut() {
        with_scope(|cx| {
            let mut elem = el("Col", cx).child(text("a")).build();
            elem.push_child(Node::Text(text("b")));
            assert_eq!(elem.children().len(), 2);
        });
    }

    #[test]
    fn element_event_handler() {
        with_scope(|cx| {
            let elem = el("Button", cx).on("click", |_ctx, _| {}).build();
            assert_eq!(elem.event_handlers().len(), 1);
            assert_eq!(elem.event_handlers()[0].name(), "click");
        });
    }

    #[test]
    fn event_handler_invocation() {
        with_scope(|cx| {
            let called = std::rc::Rc::new(std::cell::Cell::new(false));
            let called_clone = called.clone();
            let elem = el("Button", cx)
                .on("click", move |_ctx, _| {
                    called_clone.set(true);
                })
                .build();
            let ctx = EventContext::new(vec![]);
            elem.event_handlers()[0].invoke(&ctx, &());
            assert!(called.get());
        });
    }

    #[test]
    fn element_debug_skips_callbacks() {
        with_scope(|cx| {
            let elem = el("Btn", cx).on("click", |_ctx, _| {}).build();
            let debug = format!("{elem:?}");
            assert!(debug.contains("Btn"));
            assert!(debug.contains("click"));
        });
    }

    #[test]
    fn element_builder_build_node() {
        with_scope(|cx| {
            let node = el("Spacer", cx).build_node();
            assert!(node.is_element());
        });
    }

    #[test]
    fn element_builder_child_node() {
        with_scope(|cx| {
            let child = Node::Text(text("raw"));
            let elem = el("Wrapper", cx).child_node(child).build();
            assert_eq!(elem.children().len(), 1);
        });
    }

    #[test]
    fn attribute_value_from_impls() {
        let s: AttributeValue = "hello".into();
        assert_eq!(s, AttributeValue::String("hello".into()));

        let s2: AttributeValue = String::from("world").into();
        assert_eq!(s2, AttributeValue::String("world".into()));

        let i: AttributeValue = 42i64.into();
        assert_eq!(i, AttributeValue::Int(42));

        let f: AttributeValue = 3.14f64.into();
        assert_eq!(f, AttributeValue::Float(3.14));

        let b: AttributeValue = true.into();
        assert_eq!(b, AttributeValue::Bool(true));
    }

    #[test]
    fn attribute_value_from_i32() {
        let v: AttributeValue = 42i32.into();
        assert_eq!(v, AttributeValue::Int(42));

        let neg: AttributeValue = (-10i32).into();
        assert_eq!(neg, AttributeValue::Int(-10));
    }

    #[test]
    fn attribute_value_from_u32() {
        let v: AttributeValue = 100u32.into();
        assert_eq!(v, AttributeValue::Int(100));

        let zero: AttributeValue = 0u32.into();
        assert_eq!(zero, AttributeValue::Int(0));
    }

    #[test]
    fn attribute_value_from_f32() {
        let v: AttributeValue = 1.5f32.into();
        assert_eq!(v, AttributeValue::Float(f64::from(1.5f32)));
    }

    #[test]
    fn attribute_value_from_usize() {
        let v: AttributeValue = 99usize.into();
        assert_eq!(v, AttributeValue::Int(99));

        let zero: AttributeValue = 0usize.into();
        assert_eq!(zero, AttributeValue::Int(0));
    }

    #[test]
    fn on_click_registers_handler_with_click_name() {
        with_scope(|cx| {
            let elem = el("Button", cx).on_click(|_ctx, _e| {}).build();
            assert_eq!(elem.event_handlers().len(), 1);
            assert_eq!(elem.event_handlers()[0].name(), "click");
        });
    }

    #[test]
    fn typed_handler_receives_correct_event_data() {
        with_scope(|cx| {
            let received = std::rc::Rc::new(std::cell::Cell::new((0.0f64, 0.0f64)));
            let received_clone = received.clone();
            let elem = el("Button", cx)
                .on_click(move |_ctx, e| {
                    received_clone.set((e.x, e.y));
                })
                .build();

            let ctx = EventContext::new(vec![]);
            let event = ClickEvent { x: 42.0, y: 99.0 };
            elem.event_handlers()[0].invoke(&ctx, &event);
            assert_eq!(received.get(), (42.0, 99.0));
        });
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn typed_handler_ignores_wrong_event_type() {
        with_scope(|cx| {
            let called = std::rc::Rc::new(std::cell::Cell::new(false));
            let called_clone = called.clone();
            let elem = el("Button", cx)
                .on_click(move |_ctx, _e| {
                    called_clone.set(true);
                })
                .build();

            // Invoke with a non-ClickEvent type
            let ctx = EventContext::new(vec![]);
            elem.event_handlers()[0].invoke(&ctx, &42u32);
            assert!(!called.get());
        });
    }

    #[test]
    fn multiple_handlers_on_same_event() {
        with_scope(|cx| {
            let count = std::rc::Rc::new(std::cell::Cell::new(0u32));
            let c1 = count.clone();
            let c2 = count.clone();
            let elem = el("Button", cx)
                .on_click(move |_ctx, _e| c1.set(c1.get() + 1))
                .on_click(move |_ctx, _e| c2.set(c2.get() + 1))
                .build();

            let ctx = EventContext::new(vec![]);
            let event = ClickEvent { x: 0.0, y: 0.0 };
            for h in elem.event_handlers() {
                h.invoke(&ctx, &event);
            }
            assert_eq!(count.get(), 2);
        });
    }

    #[test]
    fn handler_receives_context_and_can_stop_propagation() {
        with_scope(|cx| {
            let elem = el("Button", cx)
                .on_click(|ctx, _e| {
                    ctx.stop_propagation();
                })
                .build();

            let ctx = EventContext::new(vec![]);
            let event = ClickEvent { x: 0.0, y: 0.0 };
            elem.event_handlers()[0].invoke(&ctx, &event);
            assert!(ctx.is_propagation_stopped());
        });
    }

    #[test]
    fn typed_builder_methods_chain() {
        with_scope(|cx| {
            let elem = el("Widget", cx)
                .on_click(|_ctx, _e| {})
                .on_hover(|_ctx, _e| {})
                .on_key_down(|_ctx, _e| {})
                .on_key_up(|_ctx, _e| {})
                .on_focus(|_ctx, _e| {})
                .on_blur(|_ctx, _e| {})
                .on_scroll(|_ctx, _e| {})
                .on_text_input(|_ctx, _e| {})
                .on_drag(|_ctx, _e| {})
                .build();
            assert_eq!(elem.event_handlers().len(), 9);
        });
    }

    #[test]
    fn duplicate_attr_upserts() {
        with_scope(|cx| {
            let elem = el("Input", cx)
                .attr("value", "first")
                .attr("value", "second")
                .build();
            // Should have exactly 1 "value" attribute
            let count = elem
                .attributes()
                .iter()
                .filter(|(n, _)| *n == "value")
                .count();
            assert_eq!(count, 1);
            assert_eq!(
                elem.attr("value"),
                Some(&AttributeValue::String("second".into()))
            );
        });
    }

    #[test]
    fn attribute_value_nan_equality() {
        assert_eq!(
            AttributeValue::Float(f64::NAN),
            AttributeValue::Float(f64::NAN)
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "event type mismatch")]
    fn typed_handler_panics_on_wrong_event_type_in_debug() {
        with_scope(|cx| {
            let elem = el("Button", cx).on_click(|_ctx, _e| {}).build();
            let ctx = EventContext::new(vec![]);
            elem.event_handlers()[0].invoke(&ctx, &42u32);
        });
    }

    #[test]
    fn on_drag_registers_handler() {
        with_scope(|cx| {
            let elem = el("Canvas", cx).on_drag(|_ctx, _e| {}).build();
            assert_eq!(elem.event_handlers().len(), 1);
            assert_eq!(elem.event_handlers()[0].name(), "drag");
        });
    }

    #[test]
    fn element_custom_data_default_is_unit() {
        let elem = Element::new("Box");
        assert!(elem.custom_data().is::<()>());
    }

    #[test]
    fn element_builder_data_method() {
        with_scope(|cx| {
            let elem = el("Box", cx).data(42u32).build();
            assert!(elem.custom_data().is::<u32>());
        });
    }

    #[test]
    fn element_custom_data_downcast() {
        with_scope(|cx| {
            let elem = el("Box", cx).data(vec![1, 2, 3]).build();
            let data = elem.custom_data().downcast_ref::<Vec<i32>>();
            assert!(data.is_some());
            assert_eq!(data.map(Vec::len), Some(3));
        });
    }

    #[test]
    fn style_as_returns_correct_type() {
        with_scope(|cx| {
            let elem = el("Box", cx).style(42u32).build();
            assert_eq!(elem.style_as::<u32>(), Some(&42));
        });
    }

    #[test]
    fn style_as_returns_none_for_wrong_type() {
        with_scope(|cx| {
            let elem = el("Box", cx).style(42u32).build();
            assert_eq!(elem.style_as::<String>(), None);
        });
    }

    #[test]
    fn style_as_with_default_unit_type() {
        let elem = Element::new("Box");
        assert_eq!(elem.style_as::<()>(), Some(&()));
        assert_eq!(elem.style_as::<u32>(), None);
    }

    #[test]
    fn not_send_not_sync() {
        use static_assertions::{assert_not_impl_all, assert_not_impl_any};
        assert_not_impl_any!(Element: Send, Sync);
        assert_not_impl_any!(ElementBuilder: Send, Sync);
        assert_not_impl_all!(EventHandler: Send, Sync);
    }
}
