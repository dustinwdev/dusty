//! Common imports for Dusty component authors.
//!
//! ```
//! use dusty::prelude::*;
//! ```

// Reactive types
pub use dusty_reactive::{
    Effect, Memo, ReadSignal, Resource, ResourceResolver, ResourceState, Scope, Signal, WriteSignal,
};

// Reactive functions
pub use dusty_reactive::{
    batch, create_child_scope, create_effect, create_memo, create_resource, create_scope,
    create_signal, create_signal_split, on_cleanup, provide_context, untrack, use_context,
};

// Fallible reactive functions (try_* variants)
pub use dusty_reactive::{
    try_batch, try_create_child_scope, try_create_effect, try_create_memo, try_create_resource,
    try_create_scope, try_create_signal, try_create_signal_split, try_dispose_scope,
    try_provide_context, try_use_context,
};

// Core types
pub use dusty_core::{
    AttributeValue, ComponentNode, DynamicNode, Element, ElementBuilder, EventHandler,
    IntoEventHandler, Node, TextContent, TextNode,
};

// Core functions
pub use dusty_core::{dynamic_node, el, fragment, text, text_dynamic};

// Core traits
pub use dusty_core::{IntoView, View, ViewSeq};

// Events
pub use dusty_core::{
    BlurEvent, ClickEvent, DragEvent, DragPhase, Event, EventContext, FocusEvent, HoverEvent, Key,
    KeyDownEvent, KeyUpEvent, Modifiers, ScrollEvent, TextInputEvent,
};

// Style types
pub use dusty_style::{
    AlignItems, AlignSelf, BoxShadow, Color, ColorScale, ColorStop, Corners, Edges, FlexDirection,
    FlexWrap, FontSlant, FontStyle, FontWeight, GradientDirection, InteractionState,
    JustifyContent, Length, LengthPercent, LinearGradient, Overflow, Palette, Style,
};

// Theme
pub use dusty_style::theme::{provide_theme, use_theme, Theme};

// Widgets — display
pub use dusty_widgets::{Canvas, Divider, Image, Orientation, SizingMode, Spacer, Text};

// Widgets — interactive
pub use dusty_widgets::{
    Button, ButtonVariant, Checkbox, CheckedSource, InputSource, Radio, Slider, SliderSource,
    TextInput, TextInputState, Toggle, ToggleSource,
};

// Widgets — containers
pub use dusty_widgets::{ErrorBoundary, For, MatchView, ScrollAxis, ScrollView, Show, Suspense};

// Macros
pub use dusty_macros::component;
pub use dusty_widgets::{col, row};
