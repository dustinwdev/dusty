use std::any::Any;
use std::cell::Cell;

use crate::node::Node;

/// Trait for typed events in the Dusty event system.
///
/// Each event type provides a static name used for handler matching
/// during dispatch.
///
/// # Example
///
/// ```
/// use dusty_core::event::Event;
/// use dusty_core::event::ClickEvent;
///
/// assert_eq!(ClickEvent::event_name(), "click");
/// ```
pub trait Event: 'static {
    /// Returns the string name of this event type.
    fn event_name() -> &'static str;
}

/// A mouse click event.
#[derive(Debug, Clone)]
pub struct ClickEvent {
    /// X coordinate of the click.
    pub x: f64,
    /// Y coordinate of the click.
    pub y: f64,
}

impl PartialEq for ClickEvent {
    fn eq(&self, other: &Self) -> bool {
        self.x.total_cmp(&other.x).is_eq() && self.y.total_cmp(&other.y).is_eq()
    }
}

impl Event for ClickEvent {
    fn event_name() -> &'static str {
        "click"
    }
}

/// A mouse hover event.
#[derive(Debug, Clone)]
pub struct HoverEvent {
    /// X coordinate of the pointer.
    pub x: f64,
    /// Y coordinate of the pointer.
    pub y: f64,
}

impl PartialEq for HoverEvent {
    fn eq(&self, other: &Self) -> bool {
        self.x.total_cmp(&other.x).is_eq() && self.y.total_cmp(&other.y).is_eq()
    }
}

impl Event for HoverEvent {
    fn event_name() -> &'static str {
        "hover"
    }
}

/// A simple key identifier.
///
/// Wraps a string name. Real key mapping comes in the platform layer (Phase 12).
///
/// Common keys are available as associated constructor functions:
///
/// ```
/// use dusty_core::event::Key;
///
/// assert_eq!(Key::enter(), Key("Enter".into()));
/// assert_eq!(Key::escape(), Key("Escape".into()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key(pub String);

impl Key {
    /// The Enter/Return key.
    #[must_use]
    pub fn enter() -> Self {
        Self("Enter".into())
    }
    /// The Escape key.
    #[must_use]
    pub fn escape() -> Self {
        Self("Escape".into())
    }
    /// The Tab key.
    #[must_use]
    pub fn tab() -> Self {
        Self("Tab".into())
    }
    /// The Backspace key.
    #[must_use]
    pub fn backspace() -> Self {
        Self("Backspace".into())
    }
    /// The Space key.
    #[must_use]
    pub fn space() -> Self {
        Self("Space".into())
    }
    /// The Delete key.
    #[must_use]
    pub fn delete() -> Self {
        Self("Delete".into())
    }
    /// The Up arrow key.
    #[must_use]
    pub fn arrow_up() -> Self {
        Self("ArrowUp".into())
    }
    /// The Down arrow key.
    #[must_use]
    pub fn arrow_down() -> Self {
        Self("ArrowDown".into())
    }
    /// The Left arrow key.
    #[must_use]
    pub fn arrow_left() -> Self {
        Self("ArrowLeft".into())
    }
    /// The Right arrow key.
    #[must_use]
    pub fn arrow_right() -> Self {
        Self("ArrowRight".into())
    }
}

/// Keyboard modifier state.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Modifiers {
    /// Whether Shift is held.
    pub shift: bool,
    /// Whether Ctrl is held.
    pub ctrl: bool,
    /// Whether Alt/Option is held.
    pub alt: bool,
    /// Whether Meta/Command is held.
    pub meta: bool,
}

/// A key-down event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyDownEvent {
    /// The key that was pressed.
    pub key: Key,
    /// Active modifier keys.
    pub modifiers: Modifiers,
}

impl Event for KeyDownEvent {
    fn event_name() -> &'static str {
        "key_down"
    }
}

/// A key-up event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyUpEvent {
    /// The key that was released.
    pub key: Key,
    /// Active modifier keys.
    pub modifiers: Modifiers,
}

impl Event for KeyUpEvent {
    fn event_name() -> &'static str {
        "key_up"
    }
}

/// An element gained focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusEvent;

impl Event for FocusEvent {
    fn event_name() -> &'static str {
        "focus"
    }
}

/// An element lost focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlurEvent;

impl Event for BlurEvent {
    fn event_name() -> &'static str {
        "blur"
    }
}

/// A scroll event.
#[derive(Debug, Clone)]
pub struct ScrollEvent {
    /// Horizontal scroll delta.
    pub delta_x: f64,
    /// Vertical scroll delta.
    pub delta_y: f64,
}

impl PartialEq for ScrollEvent {
    fn eq(&self, other: &Self) -> bool {
        self.delta_x.total_cmp(&other.delta_x).is_eq()
            && self.delta_y.total_cmp(&other.delta_y).is_eq()
    }
}

impl Event for ScrollEvent {
    fn event_name() -> &'static str {
        "scroll"
    }
}

/// A text input event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputEvent {
    /// The text that was entered.
    pub text: String,
}

impl Event for TextInputEvent {
    fn event_name() -> &'static str {
        "text_input"
    }
}

/// The phase of a drag gesture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragPhase {
    /// The drag gesture started.
    Start,
    /// The pointer moved during the drag.
    Move,
    /// The drag gesture ended.
    End,
}

/// A drag event with position and delta information.
#[derive(Debug, Clone)]
pub struct DragEvent {
    /// X coordinate of the pointer.
    pub x: f64,
    /// Y coordinate of the pointer.
    pub y: f64,
    /// Horizontal distance moved since last event.
    pub delta_x: f64,
    /// Vertical distance moved since last event.
    pub delta_y: f64,
    /// Phase of the drag gesture.
    pub phase: DragPhase,
}

impl PartialEq for DragEvent {
    fn eq(&self, other: &Self) -> bool {
        self.x.total_cmp(&other.x).is_eq()
            && self.y.total_cmp(&other.y).is_eq()
            && self.delta_x.total_cmp(&other.delta_x).is_eq()
            && self.delta_y.total_cmp(&other.delta_y).is_eq()
            && self.phase == other.phase
    }
}

impl Event for DragEvent {
    fn event_name() -> &'static str {
        "drag"
    }
}

/// Context passed to event handlers during dispatch.
///
/// Allows handlers to control event propagation. Two levels of stopping
/// are available:
///
/// - [`stop_propagation`](Self::stop_propagation) prevents the event from
///   bubbling to ancestor elements, but sibling handlers on the **same**
///   element still fire.
/// - [`stop_immediate_propagation`](Self::stop_immediate_propagation) prevents
///   both bubbling **and** any remaining sibling handlers on the current element.
pub struct EventContext {
    stopped: Cell<bool>,
    immediate_stopped: Cell<bool>,
    target_path: Vec<usize>,
}

impl EventContext {
    /// Creates a new context for the given target path.
    pub(crate) const fn new(target_path: Vec<usize>) -> Self {
        Self {
            stopped: Cell::new(false),
            immediate_stopped: Cell::new(false),
            target_path,
        }
    }

    /// Stops the event from bubbling to ancestor elements.
    ///
    /// Sibling handlers on the same element will still fire. Use
    /// [`stop_immediate_propagation`](Self::stop_immediate_propagation) to
    /// also cancel sibling handlers.
    pub fn stop_propagation(&self) {
        self.stopped.set(true);
    }

    /// Returns whether propagation has been stopped.
    #[must_use]
    pub fn is_propagation_stopped(&self) -> bool {
        self.stopped.get()
    }

    /// Stops the event from reaching any further handlers, including
    /// sibling handlers on the same element and all ancestor handlers.
    pub fn stop_immediate_propagation(&self) {
        self.immediate_stopped.set(true);
        self.stopped.set(true);
    }

    /// Returns whether immediate propagation has been stopped.
    #[must_use]
    pub fn is_immediate_propagation_stopped(&self) -> bool {
        self.immediate_stopped.get()
    }

    /// Returns the path of child indices from root to the target element.
    #[must_use]
    pub fn target_path(&self) -> &[usize] {
        &self.target_path
    }
}

/// Dispatches a typed event along a path through the node tree, bubbling from target to root.
///
/// Walks the `target_path` from the root, collecting element references.
/// Then iterates in reverse (target first, root last), invoking matching handlers.
/// Returns `Ok(true)` if any handler fired, `Ok(false)` if none matched.
///
/// Non-element nodes encountered along the path are skipped.
///
/// # Errors
///
/// Returns [`CoreError::InvalidTargetPath`](crate::CoreError::InvalidTargetPath)
/// if a path index is out of bounds.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_core::el;
/// use dusty_core::event::{dispatch_event, ClickEvent};
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let tree = el("Root", cx)
///         .on_click(|_ctx, _e| {})
///         .child(el("Child", cx))
///         .build_node();
///     let handled = dispatch_event(&tree, &[0], &ClickEvent { x: 10.0, y: 20.0 }).unwrap();
///     assert!(handled);
/// }).unwrap();
/// dispose_runtime();
/// ```
pub fn dispatch_event<E: Event>(
    root: &Node,
    target_path: &[usize],
    event: &E,
) -> crate::Result<bool> {
    // Phase 1: Walk the path, resolving Dynamic nodes. All resolved nodes are
    // stored in `resolved` so their element references remain valid. We track
    // which nodes along the path are elements using (source, index) pairs.
    let mut resolved: Vec<Node> = Vec::new();

    // Resolve all dynamics first, building up `resolved`. Then re-walk to
    // collect element references. This avoids borrowing `resolved` while
    // mutating it.

    // Step 1a: Resolve root
    let root_is_dynamic = matches!(root, Node::Dynamic(_));
    if let Node::Dynamic(dn) = root {
        resolved.push(resolve_dynamic(dn));
    }

    // Step 1b: Walk path, resolving as we go. Track the index in `resolved`
    // for each step that needed resolution.
    //
    // We need to know which steps in the path resolve to dynamic nodes.
    // Walk using indices: get a reference to current, check children[idx],
    // resolve if Dynamic.
    {
        let mut current: &Node = if root_is_dynamic { &resolved[0] } else { root };
        for &idx in target_path {
            let children = current.children();
            if idx >= children.len() {
                return Err(crate::CoreError::InvalidTargetPath);
            }
            let child = &children[idx];
            if let Node::Dynamic(dn) = child {
                resolved.push(resolve_dynamic(dn));
                // SAFETY rationale: we just pushed, so `last()` is always `Some`.
                let len = resolved.len();
                current = &resolved[len - 1];
            } else {
                current = child;
            }
        }
    }

    // Phase 2: Re-walk the path (now that `resolved` is frozen) to collect
    // element references for bubbling dispatch.
    let mut elements: Vec<&crate::element::Element> = Vec::new();
    let mut resolved_cursor = 0usize;

    let effective_root: &Node = if root_is_dynamic {
        let r = &resolved[resolved_cursor];
        resolved_cursor += 1;
        r
    } else {
        root
    };

    if let Node::Element(el) = effective_root {
        elements.push(el);
    }

    let mut current: &Node = effective_root;
    for &idx in target_path {
        let child = &current.children()[idx];
        if matches!(child, Node::Dynamic(_)) {
            let r = &resolved[resolved_cursor];
            resolved_cursor += 1;
            current = r;
        } else {
            current = child;
        }
        if let Node::Element(el) = current {
            elements.push(el);
        }
    }

    // Phase 3: Bubble dispatch — target (last) to root (first)
    let event_name = E::event_name();
    let ctx = EventContext::new(target_path.to_vec());
    let event_any: &dyn Any = event;
    let mut handled = false;

    for el in elements.iter().rev() {
        for handler in el.event_handlers() {
            if ctx.is_immediate_propagation_stopped() {
                break;
            }
            if handler.name() == event_name {
                handler.invoke(&ctx, event_any);
                handled = true;
            }
        }
        if ctx.is_propagation_stopped() {
            break;
        }
    }

    Ok(handled)
}

/// Resolves a Dynamic node, unwrapping nested dynamics.
fn resolve_dynamic(dn: &crate::node::DynamicNode) -> Node {
    let mut node = dn.current_node();
    while let Node::Dynamic(inner) = node {
        node = inner.current_node();
    }
    node
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Event trait + struct tests --

    #[test]
    fn click_event_name() {
        assert_eq!(ClickEvent::event_name(), "click");
    }

    #[test]
    fn hover_event_name() {
        assert_eq!(HoverEvent::event_name(), "hover");
    }

    #[test]
    fn key_down_event_name() {
        assert_eq!(KeyDownEvent::event_name(), "key_down");
    }

    #[test]
    fn key_up_event_name() {
        assert_eq!(KeyUpEvent::event_name(), "key_up");
    }

    #[test]
    fn focus_event_name() {
        assert_eq!(FocusEvent::event_name(), "focus");
    }

    #[test]
    fn blur_event_name() {
        assert_eq!(BlurEvent::event_name(), "blur");
    }

    #[test]
    fn scroll_event_name() {
        assert_eq!(ScrollEvent::event_name(), "scroll");
    }

    #[test]
    fn text_input_event_name() {
        assert_eq!(TextInputEvent::event_name(), "text_input");
    }

    #[test]
    fn click_event_data_accessible() {
        let e = ClickEvent { x: 10.0, y: 20.0 };
        assert_eq!(e.x, 10.0);
        assert_eq!(e.y, 20.0);
    }

    #[test]
    fn key_event_data_accessible() {
        let e = KeyDownEvent {
            key: Key("Enter".into()),
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
        };
        assert_eq!(e.key, Key("Enter".into()));
        assert!(e.modifiers.shift);
        assert!(!e.modifiers.ctrl);
    }

    #[test]
    fn key_enter() {
        assert_eq!(Key::enter(), Key("Enter".into()));
    }

    #[test]
    fn key_escape() {
        assert_eq!(Key::escape(), Key("Escape".into()));
    }

    #[test]
    fn key_tab() {
        assert_eq!(Key::tab(), Key("Tab".into()));
    }

    #[test]
    fn key_backspace() {
        assert_eq!(Key::backspace(), Key("Backspace".into()));
    }

    #[test]
    fn key_space() {
        assert_eq!(Key::space(), Key("Space".into()));
    }

    #[test]
    fn key_delete() {
        assert_eq!(Key::delete(), Key("Delete".into()));
    }

    #[test]
    fn key_arrow_up() {
        assert_eq!(Key::arrow_up(), Key("ArrowUp".into()));
    }

    #[test]
    fn key_arrow_down() {
        assert_eq!(Key::arrow_down(), Key("ArrowDown".into()));
    }

    #[test]
    fn key_arrow_left() {
        assert_eq!(Key::arrow_left(), Key("ArrowLeft".into()));
    }

    #[test]
    fn key_arrow_right() {
        assert_eq!(Key::arrow_right(), Key("ArrowRight".into()));
    }

    #[test]
    fn modifiers_default_all_false() {
        let m = Modifiers::default();
        assert!(!m.shift);
        assert!(!m.ctrl);
        assert!(!m.alt);
        assert!(!m.meta);
    }

    #[test]
    fn text_input_event_data() {
        let e = TextInputEvent {
            text: "hello".into(),
        };
        assert_eq!(e.text, "hello");
    }

    #[test]
    fn scroll_event_data() {
        let e = ScrollEvent {
            delta_x: 1.5,
            delta_y: -3.0,
        };
        assert_eq!(e.delta_x, 1.5);
        assert_eq!(e.delta_y, -3.0);
    }

    #[test]
    fn click_event_nan_equality() {
        let a = ClickEvent {
            x: f64::NAN,
            y: 0.0,
        };
        let b = ClickEvent {
            x: f64::NAN,
            y: 0.0,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn hover_event_nan_equality() {
        let a = HoverEvent {
            x: f64::NAN,
            y: 0.0,
        };
        let b = HoverEvent {
            x: f64::NAN,
            y: 0.0,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn drag_event_name() {
        assert_eq!(DragEvent::event_name(), "drag");
    }

    #[test]
    fn drag_event_phases() {
        let start = DragEvent {
            x: 0.0,
            y: 0.0,
            delta_x: 0.0,
            delta_y: 0.0,
            phase: DragPhase::Start,
        };
        let moving = DragEvent {
            x: 10.0,
            y: 5.0,
            delta_x: 10.0,
            delta_y: 5.0,
            phase: DragPhase::Move,
        };
        let end = DragEvent {
            x: 10.0,
            y: 5.0,
            delta_x: 0.0,
            delta_y: 0.0,
            phase: DragPhase::End,
        };
        assert_eq!(start.phase, DragPhase::Start);
        assert_eq!(moving.phase, DragPhase::Move);
        assert_eq!(end.phase, DragPhase::End);
    }

    #[test]
    fn scroll_event_nan_equality() {
        let a = ScrollEvent {
            delta_x: f64::NAN,
            delta_y: 0.0,
        };
        let b = ScrollEvent {
            delta_x: f64::NAN,
            delta_y: 0.0,
        };
        assert_eq!(a, b);
    }

    // -- EventContext tests --

    #[test]
    fn context_propagation_starts_unset() {
        let ctx = EventContext::new(vec![0, 1]);
        assert!(!ctx.is_propagation_stopped());
    }

    #[test]
    fn context_stop_propagation() {
        let ctx = EventContext::new(vec![]);
        ctx.stop_propagation();
        assert!(ctx.is_propagation_stopped());
    }

    #[test]
    fn context_exposes_target_path() {
        let ctx = EventContext::new(vec![2, 0, 1]);
        assert_eq!(ctx.target_path(), &[2, 0, 1]);
    }

    // -- Dispatch tests --

    use crate::el;
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};

    fn with_scope(f: impl FnOnce(dusty_reactive::Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx)).unwrap();
        dispose_runtime();
    }

    #[test]
    fn dispatch_to_leaf_invokes_handler() {
        with_scope(|cx| {
            let called = std::rc::Rc::new(std::cell::Cell::new(false));
            let called_clone = called.clone();
            let tree = el("Root", cx)
                .child(el("Child", cx).on_click(move |_ctx, _e| {
                    called_clone.set(true);
                }))
                .build_node();

            let handled = dispatch_event(&tree, &[0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            assert!(handled);
            assert!(called.get());
        });
    }

    #[test]
    fn dispatch_bubbles_target_to_root() {
        with_scope(|cx| {
            let order = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
            let order1 = order.clone();
            let order2 = order.clone();

            let tree = el("Root", cx)
                .on_click(move |_ctx, _e| {
                    order1.borrow_mut().push("root");
                })
                .child(el("Child", cx).on_click(move |_ctx, _e| {
                    order2.borrow_mut().push("child");
                }))
                .build_node();

            dispatch_event(&tree, &[0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            assert_eq!(*order.borrow(), vec!["child", "root"]);
        });
    }

    #[test]
    fn dispatch_stop_propagation_prevents_ancestor() {
        with_scope(|cx| {
            let root_called = std::rc::Rc::new(std::cell::Cell::new(false));
            let root_clone = root_called.clone();

            let tree = el("Root", cx)
                .on_click(move |_ctx, _e| {
                    root_clone.set(true);
                })
                .child(el("Child", cx).on_click(|ctx, _e| {
                    ctx.stop_propagation();
                }))
                .build_node();

            dispatch_event(&tree, &[0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            assert!(!root_called.get());
        });
    }

    #[test]
    fn dispatch_returns_false_when_unhandled() {
        with_scope(|cx| {
            let tree = el("Root", cx).child(el("Child", cx)).build_node();

            let handled = dispatch_event(&tree, &[0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            assert!(!handled);
        });
    }

    #[test]
    fn dispatch_invalid_path_returns_error() {
        with_scope(|cx| {
            let tree = el("Root", cx).build_node();
            let result = dispatch_event(&tree, &[5], &ClickEvent { x: 0.0, y: 0.0 });
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), crate::CoreError::InvalidTargetPath);
        });
    }

    #[test]
    fn dispatch_empty_path_targets_root() {
        with_scope(|cx| {
            let called = std::rc::Rc::new(std::cell::Cell::new(false));
            let called_clone = called.clone();
            let tree = el("Root", cx)
                .on_click(move |_ctx, _e| {
                    called_clone.set(true);
                })
                .build_node();

            let handled = dispatch_event(&tree, &[], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            assert!(handled);
            assert!(called.get());
        });
    }

    #[test]
    fn dispatch_skips_non_element_nodes() {
        with_scope(|cx| {
            // A fragment wrapping an element — dispatching through it shouldn't error
            let tree = el("Root", cx)
                .child(el("Child", cx).on_click(|_ctx, _e| {}))
                .build_node();

            // Path [0] targets the child element — works fine
            let handled = dispatch_event(&tree, &[0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            assert!(handled);
        });
    }

    #[test]
    fn dispatch_multiple_handlers_at_same_level() {
        with_scope(|cx| {
            let count = std::rc::Rc::new(std::cell::Cell::new(0u32));
            let c1 = count.clone();
            let c2 = count.clone();

            let tree = el("Root", cx)
                .on_click(move |_ctx, _e| {
                    c1.set(c1.get() + 1);
                })
                .on_click(move |_ctx, _e| {
                    c2.set(c2.get() + 1);
                })
                .build_node();

            dispatch_event(&tree, &[], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            assert_eq!(count.get(), 2);
        });
    }

    #[test]
    fn dispatch_stop_propagation_mid_chain() {
        with_scope(|cx| {
            let order = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
            let o1 = order.clone();
            let o2 = order.clone();
            let o3 = order.clone();

            let tree = el("Root", cx)
                .on_click(move |_ctx, _e| {
                    o1.borrow_mut().push("root");
                })
                .child(
                    el("Mid", cx)
                        .on_click(move |ctx, _e| {
                            o2.borrow_mut().push("mid");
                            ctx.stop_propagation();
                        })
                        .child(el("Leaf", cx).on_click(move |_ctx, _e| {
                            o3.borrow_mut().push("leaf");
                        })),
                )
                .build_node();

            dispatch_event(&tree, &[0, 0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            assert_eq!(*order.borrow(), vec!["leaf", "mid"]);
        });
    }

    #[test]
    fn dispatch_through_dynamic_node() {
        use crate::node::{dynamic_node, Node};

        with_scope(|cx| {
            // Root element has a Dynamic child that resolves to an element
            let tree = el("Root", cx)
                .child_node(Node::Dynamic(dynamic_node(|| {
                    Node::Element(crate::element::Element::new("Inner"))
                })))
                .build_node();

            // Path [0] hits the Dynamic node. Dispatch resolves it and
            // walks the resolved Element. No handler → not handled, no error.
            let handled = dispatch_event(&tree, &[0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            assert!(!handled);
        });
    }

    #[test]
    fn dispatch_dynamic_root_resolves() {
        use crate::node::{dynamic_node, Node};

        with_scope(|_cx| {
            // Dynamic at root, empty path — resolves to element
            let tree = Node::Dynamic(dynamic_node(|| {
                Node::Element(crate::element::Element::new("Resolved"))
            }));

            let result = dispatch_event(&tree, &[], &ClickEvent { x: 0.0, y: 0.0 });
            assert!(result.is_ok());
        });
    }

    // -- stop_immediate_propagation tests --

    #[test]
    fn context_immediate_propagation_starts_unset() {
        let ctx = EventContext::new(vec![]);
        assert!(!ctx.is_immediate_propagation_stopped());
    }

    #[test]
    fn context_stop_immediate_propagation_sets_both_flags() {
        let ctx = EventContext::new(vec![]);
        ctx.stop_immediate_propagation();
        assert!(ctx.is_immediate_propagation_stopped());
        assert!(ctx.is_propagation_stopped());
    }

    #[test]
    fn stop_propagation_does_not_set_immediate() {
        let ctx = EventContext::new(vec![]);
        ctx.stop_propagation();
        assert!(ctx.is_propagation_stopped());
        assert!(!ctx.is_immediate_propagation_stopped());
    }

    #[test]
    fn dispatch_stop_immediate_prevents_sibling_handlers() {
        with_scope(|cx| {
            let count = std::rc::Rc::new(std::cell::Cell::new(0u32));
            let c1 = count.clone();
            let c2 = count.clone();

            let tree = el("Root", cx)
                .on_click(move |ctx, _e| {
                    c1.set(c1.get() + 1);
                    ctx.stop_immediate_propagation();
                })
                .on_click(move |_ctx, _e| {
                    c2.set(c2.get() + 1);
                })
                .build_node();

            dispatch_event(&tree, &[], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            // Only the first handler should have fired
            assert_eq!(count.get(), 1);
        });
    }

    #[test]
    fn dispatch_stop_immediate_also_prevents_ancestor() {
        with_scope(|cx| {
            let root_called = std::rc::Rc::new(std::cell::Cell::new(false));
            let root_clone = root_called.clone();

            let tree = el("Root", cx)
                .on_click(move |_ctx, _e| {
                    root_clone.set(true);
                })
                .child(el("Child", cx).on_click(|ctx, _e| {
                    ctx.stop_immediate_propagation();
                }))
                .build_node();

            dispatch_event(&tree, &[0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            assert!(!root_called.get());
        });
    }

    #[test]
    fn dispatch_stop_propagation_still_fires_sibling_handlers() {
        with_scope(|cx| {
            let count = std::rc::Rc::new(std::cell::Cell::new(0u32));
            let c1 = count.clone();
            let c2 = count.clone();

            let tree = el("Root", cx)
                .on_click(move |ctx, _e| {
                    c1.set(c1.get() + 1);
                    ctx.stop_propagation();
                })
                .on_click(move |_ctx, _e| {
                    c2.set(c2.get() + 1);
                })
                .build_node();

            dispatch_event(&tree, &[], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
            // Both sibling handlers fire — stop_propagation only prevents ancestors
            assert_eq!(count.get(), 2);
        });
    }
}
