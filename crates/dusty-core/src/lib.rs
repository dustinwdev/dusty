//! View trait, node types, and events for Dusty.

pub mod element;
pub mod error;
pub mod event;
pub mod node;
pub mod view;
pub mod view_seq;

pub use element::{el, AttributeValue, Element, ElementBuilder, EventHandler};
pub use error::{CoreError, Result};
pub use event::{
    BlurEvent, ClickEvent, DragEvent, DragPhase, Event, EventContext, FocusEvent, HoverEvent, Key,
    KeyDownEvent, KeyUpEvent, Modifiers, ScrollEvent, TextInputEvent,
};
pub use node::{
    dynamic_node, text, text_dynamic, ComponentNode, DynamicNode, Node, TextContent, TextNode,
};
pub use view::{fragment, IntoView, View};
pub use view_seq::ViewSeq;
