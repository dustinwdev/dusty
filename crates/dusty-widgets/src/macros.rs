/// Creates a column layout container (flex-direction: column).
///
/// # Syntax
///
/// ```ignore
/// col![cx; child1, child2, child3]
/// col![cx;]  // empty column
/// ```
#[macro_export]
macro_rules! col {
    ($cx:expr;) => {{
        $crate::__macro_internals::el("Col", $cx)
            .style($crate::__macro_internals::Style {
                flex_direction: Some($crate::__macro_internals::FlexDirection::Column),
                gap: Some($crate::__macro_internals::LengthPercent::Px(8.0)),
                ..$crate::__macro_internals::Style::default()
            })
            .build_node()
    }};
    ($cx:expr; $($child:expr),+ $(,)?) => {{
        $crate::__macro_internals::el("Col", $cx)
            .style($crate::__macro_internals::Style {
                flex_direction: Some($crate::__macro_internals::FlexDirection::Column),
                gap: Some($crate::__macro_internals::LengthPercent::Px(8.0)),
                ..$crate::__macro_internals::Style::default()
            })
            .children(($($child,)+))
            .build_node()
    }};
}

/// Creates a row layout container (flex-direction: row).
///
/// # Syntax
///
/// ```ignore
/// row![cx; child1, child2, child3]
/// row![cx;]  // empty row
/// ```
#[macro_export]
macro_rules! row {
    ($cx:expr;) => {{
        $crate::__macro_internals::el("Row", $cx)
            .style($crate::__macro_internals::Style {
                flex_direction: Some($crate::__macro_internals::FlexDirection::Row),
                gap: Some($crate::__macro_internals::LengthPercent::Px(8.0)),
                align_items: Some($crate::__macro_internals::AlignItems::Center),
                ..$crate::__macro_internals::Style::default()
            })
            .build_node()
    }};
    ($cx:expr; $($child:expr),+ $(,)?) => {{
        $crate::__macro_internals::el("Row", $cx)
            .style($crate::__macro_internals::Style {
                flex_direction: Some($crate::__macro_internals::FlexDirection::Row),
                gap: Some($crate::__macro_internals::LengthPercent::Px(8.0)),
                align_items: Some($crate::__macro_internals::AlignItems::Center),
                ..$crate::__macro_internals::Style::default()
            })
            .children(($($child,)+))
            .build_node()
    }};
}

/// Creates a [`Text`](crate::Text) widget.
///
/// # Syntax
///
/// ```ignore
/// text!("hello")                          // static
/// text!("Count: {}", count)              // format
/// text!(=> format!("x: {}", s.get()))    // dynamic (closure)
/// ```
#[macro_export]
macro_rules! text {
    (=> $expr:expr) => {
        $crate::Text::dynamic(move || $expr)
    };
    ($lit:literal) => {
        $crate::Text::new($lit)
    };
    ($fmt:literal, $($arg:expr),+ $(,)?) => {
        $crate::Text::new(format!($fmt, $($arg),+))
    };
}

/// Creates a [`Button`](crate::Button) widget.
///
/// # Syntax
///
/// ```ignore
/// button!("Click")                        // static
/// button!("Save {}", name)              // format
/// button!(=> format!("x: {}", s.get())) // dynamic (closure)
/// ```
#[macro_export]
macro_rules! button {
    (=> $expr:expr) => {
        $crate::Button::dynamic(move || $expr)
    };
    ($lit:literal) => {
        $crate::Button::new($lit)
    };
    ($fmt:literal, $($arg:expr),+ $(,)?) => {
        $crate::Button::new(format!($fmt, $($arg),+))
    };
}

#[cfg(test)]
mod tests {
    use dusty_core::node::Node;
    use dusty_core::view::View;
    use dusty_core::Element;
    use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime, Scope};
    use dusty_style::{FlexDirection, Length, Style};

    fn with_scope(f: impl FnOnce(Scope)) {
        initialize_runtime();
        create_scope(|cx| f(cx));
        dispose_runtime();
    }

    fn extract_element(node: &Node) -> &Element {
        match node {
            Node::Element(el) => el,
            _ => panic!("expected Element node, got {node:?}"),
        }
    }

    fn extract_component_element(node: &Node) -> &Element {
        match node {
            Node::Component(comp) => match &*comp.child {
                Node::Element(el) => el,
                _ => panic!("expected Element inside Component"),
            },
            _ => panic!("expected Component node"),
        }
    }

    // ---- col! / row! ----

    #[test]
    fn col_empty() {
        with_scope(|cx| {
            let node = col![cx;];
            assert!(node.is_element());
            let el = extract_element(&node);
            assert_eq!(el.name(), "Col");
            assert_eq!(el.children().len(), 0);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.flex_direction, Some(FlexDirection::Column));
        });
    }

    #[test]
    fn col_with_children() {
        with_scope(|cx| {
            let node = col![cx; "a", "b", "c"];
            let el = extract_element(&node);
            assert_eq!(el.name(), "Col");
            assert_eq!(el.children().len(), 3);
        });
    }

    #[test]
    fn col_trailing_comma() {
        with_scope(|cx| {
            let node = col![cx; "a", "b",];
            let el = extract_element(&node);
            assert_eq!(el.children().len(), 2);
        });
    }

    #[test]
    fn row_empty() {
        with_scope(|cx| {
            let node = row![cx;];
            assert!(node.is_element());
            let el = extract_element(&node);
            assert_eq!(el.name(), "Row");
            assert_eq!(el.children().len(), 0);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.flex_direction, Some(FlexDirection::Row));
        });
    }

    #[test]
    fn row_with_children() {
        with_scope(|cx| {
            let node = row![cx; "x", "y"];
            let el = extract_element(&node);
            assert_eq!(el.name(), "Row");
            assert_eq!(el.children().len(), 2);
        });
    }

    // ---- text! ----

    #[test]
    fn text_static() {
        with_scope(|cx| {
            let widget = text!("hello");
            let node = widget.build(cx);
            assert!(node.is_component());
            let el = extract_component_element(&node);
            if let Node::Text(t) = &el.children()[0] {
                assert_eq!(t.current_text(), "hello");
            } else {
                panic!("expected Text child");
            }
        });
    }

    #[test]
    fn text_format() {
        with_scope(|cx| {
            let count = 42;
            let widget = text!("Count: {}", count);
            let node = widget.build(cx);
            let el = extract_component_element(&node);
            if let Node::Text(t) = &el.children()[0] {
                assert_eq!(t.current_text(), "Count: 42");
            } else {
                panic!("expected Text child");
            }
        });
    }

    #[test]
    fn text_dynamic() {
        with_scope(|cx| {
            let sig = create_signal(10i32);
            let widget = text!(=> format!("val: {}", sig.get()));
            let node = widget.build(cx);
            let el = extract_component_element(&node);
            if let Node::Text(t) = &el.children()[0] {
                assert_eq!(t.current_text(), "val: 10");
            } else {
                panic!("expected Text child");
            }
        });
    }

    // ---- button! ----

    #[test]
    fn button_static() {
        with_scope(|cx| {
            let widget = button!("Click");
            let node = widget.build(cx);
            assert!(node.is_component());
            let el = extract_component_element(&node);
            if let Node::Text(t) = &el.children()[0] {
                assert_eq!(t.current_text(), "Click");
            } else {
                panic!("expected Text child");
            }
        });
    }

    #[test]
    fn button_format() {
        with_scope(|cx| {
            let name = "World";
            let widget = button!("Save {}", name);
            let node = widget.build(cx);
            let el = extract_component_element(&node);
            if let Node::Text(t) = &el.children()[0] {
                assert_eq!(t.current_text(), "Save World");
            } else {
                panic!("expected Text child");
            }
        });
    }

    #[test]
    fn button_dynamic() {
        with_scope(|cx| {
            let sig = create_signal(7i32);
            let widget = button!(=> format!("n={}", sig.get()));
            let node = widget.build(cx);
            let el = extract_component_element(&node);
            if let Node::Text(t) = &el.children()[0] {
                assert_eq!(t.current_text(), "n=7");
            } else {
                panic!("expected Text child");
            }
        });
    }

    #[test]
    fn button_macro_returns_widget_for_chaining() {
        with_scope(|cx| {
            let widget = button!("Delete").variant(crate::ButtonVariant::Danger);
            let node = widget.build(cx);
            let el = extract_component_element(&node);
            assert_eq!(
                el.attr("variant"),
                Some(&dusty_core::AttributeValue::String("danger".into()))
            );
        });
    }

    #[test]
    fn text_macro_returns_widget_for_chaining() {
        with_scope(|cx| {
            let widget = text!("styled").style(Style {
                width: Some(Length::Px(100.0)),
                ..Style::default()
            });
            let node = widget.build(cx);
            let el = extract_component_element(&node);
            let style = el.style().downcast_ref::<Style>().unwrap();
            assert_eq!(style.width, Some(Length::Px(100.0)));
        });
    }
}
