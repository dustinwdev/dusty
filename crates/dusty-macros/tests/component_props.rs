use dusty_core::node::Node;
use dusty_core::view::View;
use dusty_macros::component;
use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime, Scope};

fn with_scope(f: impl FnOnce(Scope)) {
    initialize_runtime();
    create_scope(|cx| f(cx));
    dispose_runtime();
}

// -- default (trait) --

#[component]
fn WithDefault(cx: Scope, #[prop(default)] count: i32) -> Node {
    dusty_core::el("WithDefault", cx)
        .child(dusty_core::text(format!("{}", count)))
        .build_node()
}

#[test]
fn default_trait_uses_default_value() {
    with_scope(|cx| {
        let node = WithDefault::new().build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "0"); // i32::default() == 0
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

#[test]
fn default_can_be_overridden_via_builder() {
    with_scope(|cx| {
        let node = WithDefault::new().count(42).build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "42");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

// -- default = expr --

#[component]
fn WithDefaultExpr(cx: Scope, #[prop(default = 5)] step: i32) -> Node {
    dusty_core::el("WithDefaultExpr", cx)
        .child(dusty_core::text(format!("{}", step)))
        .build_node()
}

#[test]
fn default_expr_uses_expression() {
    with_scope(|cx| {
        let node = WithDefaultExpr::new().build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "5");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

#[test]
fn default_expr_overridden() {
    with_scope(|cx| {
        let node = WithDefaultExpr::new().step(99).build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "99");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

// -- optional --

#[component]
fn WithOptional(cx: Scope, #[prop(optional)] label: String) -> Node {
    let text = label.unwrap_or_else(|| "none".to_string());
    dusty_core::el("WithOptional", cx)
        .child(dusty_core::text(text))
        .build_node()
}

#[test]
fn optional_defaults_to_none() {
    with_scope(|cx| {
        let node = WithOptional::new().build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "none");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

#[test]
fn optional_builder_takes_bare_type() {
    with_scope(|cx| {
        let node = WithOptional::new().label("hello").build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "hello");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

// -- into --

#[component]
fn WithInto(cx: Scope, #[prop(into)] title: String) -> Node {
    dusty_core::el("WithInto", cx)
        .child(dusty_core::text(title))
        .build_node()
}

#[test]
fn into_accepts_impl_into() {
    with_scope(|cx| {
        // Pass &str where String is expected — impl Into<String>
        let node = WithInto::new("hello").build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "hello");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

// -- into + default --

#[component]
fn WithIntoDefault(
    cx: Scope,
    required: i32,
    #[prop(into, default = "untitled".to_string())] title: String,
) -> Node {
    dusty_core::el("WithIntoDefault", cx)
        .child(dusty_core::text(format!("{}:{}", required, title)))
        .build_node()
}

#[test]
fn into_default_uses_default_when_omitted() {
    with_scope(|cx| {
        let node = WithIntoDefault::new(1).build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "1:untitled");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

#[test]
fn into_default_builder_accepts_into() {
    with_scope(|cx| {
        let node = WithIntoDefault::new(2).title("custom").build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "2:custom");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

// -- into + optional --

#[component]
fn WithIntoOptional(cx: Scope, #[prop(into, optional)] subtitle: String) -> Node {
    let text = subtitle.unwrap_or_else(|| "none".to_string());
    dusty_core::el("WithIntoOptional", cx)
        .child(dusty_core::text(text))
        .build_node()
}

#[test]
fn into_optional_defaults_to_none() {
    with_scope(|cx| {
        let node = WithIntoOptional::new().build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "none");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

#[test]
fn into_optional_builder_accepts_into() {
    with_scope(|cx| {
        let node = WithIntoOptional::new().subtitle("sub").build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "sub");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

// -- mixed: required + default + optional + into --

#[component]
fn Kitchen(
    cx: Scope,
    name: String,
    #[prop(default = 1)] step: i32,
    #[prop(optional)] label: String,
    #[prop(into)] title: String,
) -> Node {
    let label_str = label.unwrap_or_else(|| "none".to_string());
    dusty_core::el("Kitchen", cx)
        .child(dusty_core::text(format!(
            "{}:{}:{}:{}",
            name, step, label_str, title
        )))
        .build_node()
}

#[test]
fn kitchen_sink_required_props_in_new() {
    with_scope(|cx| {
        let node = Kitchen::new("a".to_string(), "t").build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "a:1:none:t");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}

#[test]
fn kitchen_sink_all_set() {
    with_scope(|cx| {
        let node = Kitchen::new("a".to_string(), "t")
            .step(10)
            .label("lbl")
            .build(cx);
        if let Node::Component(comp) = &node {
            if let Node::Element(el) = &*comp.child {
                if let Node::Text(t) = &el.children()[0] {
                    assert_eq!(t.current_text(), "a:10:lbl:t");
                } else {
                    panic!("expected Text child");
                }
            } else {
                panic!("expected Element child");
            }
        }
    });
}
