use dusty_core::el;
use dusty_core::element::IntoEventHandler;
use dusty_core::event::{ClickEvent, EventContext};
use dusty_core::node::{text, text_dynamic, ComponentNode, Node, TextNode};
use dusty_core::view::View;
use dusty_reactive::Scope;
use dusty_style::theme::use_theme;
use dusty_style::{Color, Corners, Edges, LengthPercent, Style};

use crate::common::LabelContent;

type ClickHandler = Box<dyn Fn(&EventContext, &ClickEvent)>;

/// Visual variant for a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Default filled button.
    #[default]
    Primary,
    /// Less prominent filled button.
    Secondary,
    /// Border-only button.
    Outline,
    /// No background or border.
    Ghost,
    /// Destructive action button.
    Danger,
}

/// An interactive button widget.
///
/// # Example
///
/// ```
/// use dusty_reactive::{initialize_runtime, create_scope, dispose_runtime};
/// use dusty_widgets::Button;
/// use dusty_core::view::View;
///
/// initialize_runtime();
/// create_scope(|cx| {
///     let node = Button::new("Click me").build(cx);
///     assert!(node.is_component());
/// });
/// dispose_runtime();
/// ```
pub struct Button {
    label: LabelContent,
    variant: ButtonVariant,
    disabled: bool,
    user_style: Option<Style>,
    on_click: Option<ClickHandler>,
}

impl Button {
    /// Creates a button with a static label.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: LabelContent::Static(label.into()),
            variant: ButtonVariant::default(),
            disabled: false,
            user_style: None,
            on_click: None,
        }
    }

    /// Creates a button with a reactive (dynamic) label.
    #[must_use]
    pub fn dynamic(f: impl Fn() -> String + 'static) -> Self {
        Self {
            label: LabelContent::Dynamic(Box::new(f)),
            variant: ButtonVariant::default(),
            disabled: false,
            user_style: None,
            on_click: None,
        }
    }

    /// Sets the visual variant.
    #[must_use]
    pub const fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Disables the button, suppressing click events.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Merges user styles on top of button defaults.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.user_style = Some(style);
        self
    }

    /// Registers a click event handler.
    ///
    /// Accepts either `|e| { ... }` or `|ctx, e| { ... }`.
    #[must_use]
    pub fn on_click<M>(mut self, handler: impl IntoEventHandler<ClickEvent, M>) -> Self {
        self.on_click = Some(handler.into_handler());
        self
    }
}

impl View for Button {
    fn build(self, cx: Scope) -> Node {
        let theme = use_theme();
        let (bg, fg, border) = variant_colors(self.variant, &theme);

        let base = Style {
            padding: Edges::xy(LengthPercent::Px(16.0), LengthPercent::Px(8.0)),
            border_radius: Corners::all(6.0),
            background: Some(bg),
            foreground: Some(fg),
            border_color: border,
            border_width: if border.is_some() {
                Edges::all(1.0)
            } else {
                Edges::default()
            },
            ..Style::default()
        };

        let merged = if let Some(user) = &self.user_style {
            base.merge(user)
        } else {
            base
        };

        let styled = if self.disabled {
            merged.merge(&Style {
                opacity: Some(0.5),
                ..Style::default()
            })
        } else {
            merged
        };

        let variant_str = match self.variant {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Outline => "outline",
            ButtonVariant::Ghost => "ghost",
            ButtonVariant::Danger => "danger",
        };

        let text_child: TextNode = match self.label {
            LabelContent::Static(s) => text(s),
            LabelContent::Dynamic(f) => text_dynamic(f),
        };

        let label_str = text_child.current_text().into_owned();

        let mut builder = el("Button", cx)
            .attr("variant", variant_str)
            .attr("disabled", self.disabled)
            .attr("label", label_str)
            .style(styled)
            .child_text(text_child);

        if !self.disabled {
            if let Some(handler) = self.on_click {
                builder = builder.on_click(handler);
            }
        }

        let element = builder.build_node();

        Node::Component(ComponentNode {
            name: "Button",
            child: Box::new(element),
        })
    }
}

/// Returns `(background, foreground, border)` colors for a button variant.
#[allow(clippy::unreadable_literal)]
fn variant_colors(
    variant: ButtonVariant,
    theme: &dusty_style::theme::Theme,
) -> (Color, Color, Option<Color>) {
    match variant {
        ButtonVariant::Primary => {
            let bg = theme
                .primary
                .get(600)
                .unwrap_or_else(|| Color::hex(0x2563eb));
            (bg, Color::WHITE, None)
        }
        ButtonVariant::Secondary => {
            let bg = theme
                .secondary
                .get(200)
                .unwrap_or_else(|| Color::hex(0xe2e8f0));
            (bg, theme.foreground, None)
        }
        ButtonVariant::Outline => (Color::TRANSPARENT, theme.foreground, Some(theme.border)),
        ButtonVariant::Ghost => (Color::TRANSPARENT, theme.foreground, None),
        ButtonVariant::Danger => {
            let bg = theme
                .danger
                .get(600)
                .unwrap_or_else(|| Color::hex(0xdc2626));
            (bg, Color::WHITE, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{extract_element, with_scope};
    use dusty_core::node::TextContent as NodeTextContent;
    use dusty_core::AttributeValue;
    use dusty_reactive::create_signal;
    use dusty_style::{Edges, Length, LengthPercent};

    #[test]
    fn builds_component() {
        with_scope(|cx| {
            let node = Button::new("OK").build(cx);
            assert!(node.is_component());
            if let Node::Component(comp) = &node {
                assert_eq!(comp.name, "Button");
            }
        });
    }

    #[test]
    fn has_label_text() {
        with_scope(|cx| {
            let node = Button::new("Submit").build(cx);
            let el = extract_element(&node);
            assert_eq!(el.children().len(), 1);
            if let Node::Text(text_node) = &el.children()[0] {
                assert_eq!(text_node.current_text(), "Submit");
            } else {
                panic!("expected Text child");
            }
        });
    }

    #[test]
    fn dynamic_label() {
        with_scope(|cx| {
            let count = create_signal(0i32);
            let node = Button::dynamic(move || format!("Count: {}", count.get())).build(cx);
            let el = extract_element(&node);
            if let Node::Text(text_node) = &el.children()[0] {
                assert_eq!(text_node.current_text(), "Count: 0");
                assert!(matches!(text_node.content, NodeTextContent::Dynamic(_)));
            } else {
                panic!("expected Text child");
            }
        });
    }

    #[test]
    fn default_variant_is_primary() {
        with_scope(|cx| {
            let node = Button::new("OK").build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("variant"),
                Some(&AttributeValue::String("primary".into()))
            );
        });
    }

    #[test]
    fn variant_attr() {
        with_scope(|cx| {
            let node = Button::new("Delete")
                .variant(ButtonVariant::Danger)
                .build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("variant"),
                Some(&AttributeValue::String("danger".into()))
            );
        });
    }

    #[test]
    fn click_registers_handler() {
        with_scope(|cx| {
            let node = Button::new("Go").on_click(|_e: &ClickEvent| {}).build(cx);
            let el = extract_element(&node);
            assert!(el.event_handlers().iter().any(|h| h.name() == "click"));
        });
    }

    #[test]
    fn disabled_suppresses_click() {
        with_scope(|cx| {
            let node = Button::new("No")
                .disabled(true)
                .on_click(|_e: &ClickEvent| {})
                .build(cx);
            let el = extract_element(&node);
            assert!(!el.event_handlers().iter().any(|h| h.name() == "click"));
        });
    }

    #[test]
    fn disabled_attr() {
        with_scope(|cx| {
            let node = Button::new("No").disabled(true).build(cx);
            let el = extract_element(&node);
            assert_eq!(el.attr("disabled"), Some(&AttributeValue::Bool(true)));
        });
    }

    #[test]
    fn style_merges() {
        with_scope(|cx| {
            let node = Button::new("Styled")
                .style(Style {
                    width: Some(Length::Px(200.0)),
                    ..Style::default()
                })
                .build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.width, Some(Length::Px(200.0)));
            // Base padding should still be present
            assert_eq!(
                style.padding,
                Edges::xy(LengthPercent::Px(16.0), LengthPercent::Px(8.0))
            );
        });
    }

    #[test]
    fn button_sets_label_attr() {
        with_scope(|cx| {
            let node = Button::new("Submit").build(cx);
            let el = extract_element(&node);
            assert_eq!(
                el.attr("label"),
                Some(&AttributeValue::String("Submit".into()))
            );
        });
    }

    #[test]
    fn disabled_dims_opacity() {
        with_scope(|cx| {
            let node = Button::new("Dim").disabled(true).build(cx);
            let el = extract_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.opacity, Some(0.5));
        });
    }
}
