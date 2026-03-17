use dusty_core::el;
use dusty_core::event::{
    BlurEvent, ClickEvent, EventContext, FocusEvent, KeyDownEvent, TextInputEvent,
};
use dusty_core::node::{ComponentNode, Node};
use dusty_core::view::View;
use dusty_reactive::{create_signal, Scope, Signal};
use dusty_style::Style;

/// Source of truth for the text value.
pub enum InputSource {
    /// Widget manages its own signal internally.
    Uncontrolled(String),
    /// Caller provides the signal.
    Controlled(Signal<String>),
}

/// Internal state bundle stored in `custom_data` for renderer access.
pub struct TextInputState {
    /// The text value signal.
    pub value: Signal<String>,
    /// Cursor byte offset (always on a char boundary).
    pub cursor: Signal<usize>,
    /// Selection start byte offset, if a selection is active.
    pub selection_start: Signal<Option<usize>>,
    /// Whether the input is focused.
    pub focused: Signal<bool>,
}

/// Shared callback type for `on_change` handlers.
type ChangeCallback = std::rc::Rc<Option<Box<dyn Fn(&str)>>>;
type SubmitHandler = Box<dyn Fn(&str)>;

/// A single-line text input widget.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::TextInput;
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = TextInput::new().build(cx);
///     assert!(node.is_component());
/// });
/// dispose_runtime();
/// ```
pub struct TextInput {
    source: InputSource,
    placeholder: Option<PlaceholderContent>,
    max_length: Option<usize>,
    disabled: bool,
    read_only: bool,
    user_style: Option<Style>,
    on_change: Option<SubmitHandler>,
    on_submit: Option<SubmitHandler>,
}

enum PlaceholderContent {
    Static(String),
    Dynamic(Box<dyn Fn() -> String>),
}

impl TextInput {
    /// Creates an empty text input.
    #[must_use]
    pub fn new() -> Self {
        Self {
            source: InputSource::Uncontrolled(String::new()),
            placeholder: None,
            max_length: None,
            disabled: false,
            read_only: false,
            user_style: None,
            on_change: None,
            on_submit: None,
        }
    }

    /// Sets the initial text value (uncontrolled mode).
    #[must_use]
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.source = InputSource::Uncontrolled(value.into());
        self
    }

    /// Uses an external signal as the source of truth (controlled mode).
    #[must_use]
    pub fn controlled(mut self, signal: Signal<String>) -> Self {
        self.source = InputSource::Controlled(signal);
        self
    }

    /// Sets a static placeholder.
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(PlaceholderContent::Static(text.into()));
        self
    }

    /// Sets a reactive placeholder.
    #[must_use]
    pub fn placeholder_dynamic(mut self, f: impl Fn() -> String + 'static) -> Self {
        self.placeholder = Some(PlaceholderContent::Dynamic(Box::new(f)));
        self
    }

    /// Sets the maximum character length.
    #[must_use]
    pub const fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Disables the input, suppressing all interaction.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Makes the input read-only — focusable but not editable.
    #[must_use]
    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Merges user styles on top of text input defaults.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.user_style = Some(style);
        self
    }

    /// Registers a change handler called with the new text value.
    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Registers a submit handler called when Enter is pressed.
    #[must_use]
    pub fn on_submit(mut self, handler: impl Fn(&str) + 'static) -> Self {
        self.on_submit = Some(Box::new(handler));
        self
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Move cursor left by one char, respecting char boundaries.
fn prev_char_boundary(s: &str, pos: usize) -> usize {
    if pos == 0 {
        return 0;
    }
    let mut idx = pos - 1;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// Move cursor right by one char, respecting char boundaries.
fn next_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    let mut idx = pos + 1;
    while idx < s.len() && !s.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}

/// Delete the selection range and return the new cursor position.
fn delete_selection(s: &mut String, cursor: usize, sel_start: usize) -> usize {
    let start = cursor.min(sel_start);
    let end = cursor.max(sel_start);
    s.drain(start..end);
    start
}

impl View for TextInput {
    #[allow(clippy::too_many_lines)]
    fn build(self, cx: Scope) -> Node {
        let base = Style::default();

        let merged = if let Some(user) = &self.user_style {
            base.merge(user)
        } else {
            base
        };

        let value_signal = match self.source {
            InputSource::Uncontrolled(initial) => create_signal(initial),
            InputSource::Controlled(sig) => sig,
        };

        let initial_len = value_signal.with(String::len);
        let cursor_signal = create_signal(initial_len);
        let selection_signal: Signal<Option<usize>> = create_signal(None);
        let focused_signal = create_signal(false);

        let mut builder = el("TextInput", cx)
            .attr("value", value_signal.with(Clone::clone))
            .attr("cursor", i64::try_from(cursor_signal.get()).unwrap_or(0))
            .attr("disabled", self.disabled)
            .attr("read_only", self.read_only)
            .attr("focused", false)
            .data(TextInputState {
                value: value_signal,
                cursor: cursor_signal,
                selection_start: selection_signal,
                focused: focused_signal,
            });

        if let Some(placeholder_content) = self.placeholder {
            match placeholder_content {
                PlaceholderContent::Static(s) => {
                    builder = builder.attr("placeholder", s);
                }
                PlaceholderContent::Dynamic(f) => {
                    builder = builder.attr("placeholder", f());
                }
            }
        }

        if let Some(max) = self.max_length {
            builder = builder.attr("max_length", i64::try_from(max).unwrap_or(i64::MAX));
        }

        builder = builder.style(merged);

        if self.disabled {
            let element = builder.build_node();
            return Node::Component(ComponentNode {
                name: "TextInput",
                child: Box::new(element),
            });
        }

        // Focus / Blur
        let focused_for_focus = focused_signal;
        builder = builder.on_focus(move |_ctx: &EventContext, _e: &FocusEvent| {
            focused_for_focus.set(true);
        });

        let focused_for_blur = focused_signal;
        let sel_for_blur = selection_signal;
        builder = builder.on_blur(move |_ctx: &EventContext, _e: &BlurEvent| {
            focused_for_blur.set(false);
            sel_for_blur.set(None);
        });

        // Click — position cursor
        let cursor_for_click = cursor_signal;
        let val_for_click = value_signal;
        let sel_for_click = selection_signal;
        builder = builder.on_click(move |_ctx: &EventContext, e: &ClickEvent| {
            let len = val_for_click.with(String::len);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let pos = (e.x.max(0.0) as usize).min(len);
            let text = val_for_click.with(Clone::clone);
            let mut safe_pos = pos;
            while safe_pos > 0 && !text.is_char_boundary(safe_pos) {
                safe_pos -= 1;
            }
            cursor_for_click.set(safe_pos);
            sel_for_click.set(None);
        });

        let read_only = self.read_only;

        // Text input event — insert text at cursor
        let val_for_input = value_signal;
        let cursor_for_input = cursor_signal;
        let sel_for_input = selection_signal;
        let max_length = self.max_length;
        let on_change_for_input: ChangeCallback = std::rc::Rc::new(self.on_change);
        let on_change_for_key = on_change_for_input.clone();

        builder = builder.on_text_input(move |_ctx: &EventContext, e: &TextInputEvent| {
            if read_only {
                return;
            }

            // Read cursor and selection BEFORE mutating value
            let mut cursor = cursor_for_input.get();
            let sel = sel_for_input.get();

            let mut new_text = val_for_input.with(Clone::clone);

            // If there's a selection, delete it first
            if let Some(sel_start) = sel {
                cursor = delete_selection(&mut new_text, cursor, sel_start);
                sel_for_input.set(None);
            }

            // Enforce max_length
            if let Some(max) = max_length {
                let current_chars = new_text.chars().count();
                let insert_chars = e.text.chars().count();
                if current_chars + insert_chars > max {
                    return;
                }
            }

            new_text.insert_str(cursor, &e.text);
            cursor += e.text.len();

            val_for_input.set(new_text.clone());
            cursor_for_input.set(cursor);

            if let Some(ref cb) = *on_change_for_input {
                cb(&new_text);
            }
        });

        // Key down — navigation and editing
        let val_for_key = value_signal;
        let cursor_for_key = cursor_signal;
        let sel_for_key = selection_signal;
        let on_submit = self.on_submit;

        builder = builder.on_key_down(move |_ctx: &EventContext, e: &KeyDownEvent| {
            let key_name = e.key.0.as_str();

            // Read all state upfront (before any signal writes)
            let text_val = val_for_key.with(Clone::clone);
            let cursor = cursor_for_key.get();
            let sel = sel_for_key.get();

            match key_name {
                "Enter" => {
                    if let Some(ref cb) = on_submit {
                        cb(&text_val);
                    }
                }
                "ArrowLeft" => {
                    let new_cursor = prev_char_boundary(&text_val, cursor);
                    if e.modifiers.shift {
                        if sel.is_none() {
                            sel_for_key.set(Some(cursor));
                        }
                    } else {
                        sel_for_key.set(None);
                    }
                    cursor_for_key.set(new_cursor);
                }
                "ArrowRight" => {
                    let new_cursor = next_char_boundary(&text_val, cursor);
                    if e.modifiers.shift {
                        if sel.is_none() {
                            sel_for_key.set(Some(cursor));
                        }
                    } else {
                        sel_for_key.set(None);
                    }
                    cursor_for_key.set(new_cursor);
                }
                "Home" => {
                    if e.modifiers.shift {
                        if sel.is_none() {
                            sel_for_key.set(Some(cursor));
                        }
                    } else {
                        sel_for_key.set(None);
                    }
                    cursor_for_key.set(0);
                }
                "End" => {
                    if e.modifiers.shift {
                        if sel.is_none() {
                            sel_for_key.set(Some(cursor));
                        }
                    } else {
                        sel_for_key.set(None);
                    }
                    cursor_for_key.set(text_val.len());
                }
                "Backspace" => {
                    if read_only {
                        return;
                    }
                    let mut new_text = text_val;

                    if let Some(sel_start) = sel {
                        let new_cursor = delete_selection(&mut new_text, cursor, sel_start);
                        val_for_key.set(new_text.clone());
                        cursor_for_key.set(new_cursor);
                        sel_for_key.set(None);
                        if let Some(ref cb) = *on_change_for_key {
                            cb(&new_text);
                        }
                        return;
                    }

                    if cursor == 0 {
                        return;
                    }
                    let prev = prev_char_boundary(&new_text, cursor);
                    new_text.drain(prev..cursor);
                    val_for_key.set(new_text.clone());
                    cursor_for_key.set(prev);
                    if let Some(ref cb) = *on_change_for_key {
                        cb(&new_text);
                    }
                }
                "Delete" => {
                    if read_only {
                        return;
                    }
                    let mut new_text = text_val;

                    if let Some(sel_start) = sel {
                        let new_cursor = delete_selection(&mut new_text, cursor, sel_start);
                        val_for_key.set(new_text.clone());
                        cursor_for_key.set(new_cursor);
                        sel_for_key.set(None);
                        if let Some(ref cb) = *on_change_for_key {
                            cb(&new_text);
                        }
                        return;
                    }

                    if cursor >= new_text.len() {
                        return;
                    }
                    let next = next_char_boundary(&new_text, cursor);
                    new_text.drain(cursor..next);
                    val_for_key.set(new_text.clone());
                    if let Some(ref cb) = *on_change_for_key {
                        cb(&new_text);
                    }
                }
                "a" if e.modifiers.ctrl || e.modifiers.meta => {
                    sel_for_key.set(Some(0));
                    cursor_for_key.set(text_val.len());
                }
                _ => {}
            }
        });

        let element = builder.build_node();

        Node::Component(ComponentNode {
            name: "TextInput",
            child: Box::new(element),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_core::{AttributeValue, Element};
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
    fn builds_component() {
        with_scope(|cx| {
            let node = TextInput::new().build(cx);
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "TextInput");
            }
        });
    }

    #[test]
    fn default_empty() {
        with_scope(|cx| {
            let node = TextInput::new().build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("value"),
                Some(&AttributeValue::String(String::new()))
            );
        });
    }

    #[test]
    fn initial_value() {
        with_scope(|cx| {
            let node = TextInput::new().value("hello").build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("value"),
                Some(&AttributeValue::String("hello".into()))
            );
        });
    }

    #[test]
    fn placeholder_attr() {
        with_scope(|cx| {
            let node = TextInput::new().placeholder("Type here...").build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("placeholder"),
                Some(&AttributeValue::String("Type here...".into()))
            );
        });
    }

    #[test]
    fn disabled_suppresses_all() {
        with_scope(|cx| {
            let node = TextInput::new().disabled(true).build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().is_empty());
            assert_eq!(el.attr("disabled"), Some(&AttributeValue::Bool(true)));
        });
    }

    #[test]
    fn read_only_attr() {
        with_scope(|cx| {
            let node = TextInput::new().read_only(true).build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("read_only"), Some(&AttributeValue::Bool(true)));
            // Should still have event handlers (focus, key_down, etc.)
            assert!(!el.event_handlers().is_empty());
        });
    }

    #[test]
    fn controlled_reads_signal() {
        with_scope(|cx| {
            let sig = create_signal("initial".to_string());
            let node = TextInput::new().controlled(sig).build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("value"),
                Some(&AttributeValue::String("initial".into()))
            );
        });
    }

    #[test]
    fn focus_and_blur_handlers_registered() {
        with_scope(|cx| {
            let node = TextInput::new().build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "focus"));
            assert!(el.event_handlers().iter().any(|h| h.name() == "blur"));
        });
    }

    #[test]
    fn text_input_handler_registered() {
        with_scope(|cx| {
            let node = TextInput::new().build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "text_input"));
        });
    }

    #[test]
    fn key_down_handler_registered() {
        with_scope(|cx| {
            let node = TextInput::new().build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "key_down"));
        });
    }

    #[test]
    fn click_handler_registered() {
        with_scope(|cx| {
            let node = TextInput::new().build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "click"));
        });
    }

    #[test]
    fn stores_state_in_custom_data() {
        with_scope(|cx| {
            let node = TextInput::new().build(cx);
            let el = extract_element(&node);
            assert!(el.custom_data().downcast_ref::<TextInputState>().is_some());
        });
    }

    #[test]
    fn max_length_attr() {
        with_scope(|cx| {
            let node = TextInput::new().max_length(10).build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("max_length"), Some(&AttributeValue::Int(10)));
        });
    }

    #[test]
    fn style_merges() {
        with_scope(|cx| {
            let node = TextInput::new()
                .style(Style {
                    width: Some(300.0),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.width, Some(300.0));
        });
    }

    #[test]
    fn prev_char_boundary_ascii() {
        let s = "hello";
        assert_eq!(prev_char_boundary(s, 3), 2);
        assert_eq!(prev_char_boundary(s, 0), 0);
        assert_eq!(prev_char_boundary(s, 5), 4);
    }

    #[test]
    fn next_char_boundary_ascii() {
        let s = "hello";
        assert_eq!(next_char_boundary(s, 0), 1);
        assert_eq!(next_char_boundary(s, 4), 5);
        assert_eq!(next_char_boundary(s, 5), 5);
    }

    #[test]
    fn char_boundary_multibyte() {
        let s = "héllo"; // é is 2 bytes
        assert_eq!(next_char_boundary(s, 0), 1); // h -> é
        assert_eq!(next_char_boundary(s, 1), 3); // é (2 bytes) -> l
        assert_eq!(prev_char_boundary(s, 3), 1); // l -> é
        assert_eq!(prev_char_boundary(s, 1), 0); // é -> h
    }
}
