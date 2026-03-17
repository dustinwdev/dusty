use dusty_core::{
    el, fragment, text, text_dynamic, ComponentNode, IntoView, Node, TextContent, View, ViewSeq,
};
use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime, Scope};

fn with_scope(f: impl FnOnce(Scope)) {
    initialize_runtime();
    create_scope(|cx| f(cx));
    dispose_runtime();
}

#[test]
fn nested_element_tree() {
    with_scope(|cx| {
        let tree = el("App", cx)
            .child(el("Header", cx).child(text("Title")).attr("level", 1i64))
            .child(
                el("Body", cx)
                    .child(el("Row", cx).child(text("Left")).child(text("Right")))
                    .child(el("Footer", cx).child(text("copyright"))),
            )
            .build_node();

        assert!(tree.is_element());
        assert_eq!(tree.children().len(), 2);

        // Header
        let header = &tree.children()[0];
        assert!(header.is_element());
        assert_eq!(header.children().len(), 1);
        assert!(header.children()[0].is_text());

        // Body > Row
        let body = &tree.children()[1];
        assert_eq!(body.children().len(), 2);
        let row = &body.children()[0];
        assert_eq!(row.children().len(), 2);
    });
}

#[test]
fn fragment_with_mixed_types() {
    with_scope(|cx| {
        let frag = fragment(
            (
                "plain text",
                String::from("owned string"),
                el("Divider", cx),
                text("text node"),
            ),
            cx,
        );

        assert!(frag.is_fragment());
        assert_eq!(frag.children().len(), 4);
        assert!(frag.children()[0].is_text());
        assert!(frag.children()[1].is_text());
        assert!(frag.children()[2].is_element());
        assert!(frag.children()[3].is_text());
    });
}

#[test]
fn view_seq_heterogeneous_children() {
    with_scope(|cx| {
        let node = el("Container", cx)
            .children((
                "label",
                el("Input", cx).attr("type", "text"),
                el("Button", cx).attr("label", "Submit"),
            ))
            .build_node();

        assert_eq!(node.children().len(), 3);
    });
}

#[test]
fn component_node_wraps_child() {
    with_scope(|cx| {
        let inner = el("Row", cx).child(text("content")).build_node();
        let comp = Node::Component(ComponentNode {
            name: "MyWidget",
            child: Box::new(inner),
        });

        assert!(comp.is_component());
        assert_eq!(comp.children().len(), 1);
        assert!(comp.children()[0].is_element());

        let debug = format!("{comp:?}");
        assert!(debug.contains("MyWidget"));
    });
}

#[test]
fn reactive_text_with_signal() {
    with_scope(|cx| {
        cx.run(|| {
            let count = create_signal(0i32);

            let text_node = text_dynamic(move || format!("Count: {}", count.get()));

            assert_eq!(text_node.current_text(), "Count: 0");

            count.set(42);
            assert_eq!(text_node.current_text(), "Count: 42");

            count.set(100);
            assert_eq!(text_node.current_text(), "Count: 100");
        });
    });
}

#[test]
fn dynamic_text_content_variant() {
    let node = text_dynamic(|| "dynamic".to_string());
    assert!(matches!(node.content, TextContent::Dynamic(_)));

    let node = text("static");
    assert!(matches!(node.content, TextContent::Static(_)));
}

#[test]
fn option_view_renders_or_skips() {
    with_scope(|cx| {
        let some_node: Option<&str> = Some("visible");
        let none_node: Option<&str> = None;

        let visible = some_node.into_view(cx);
        let empty = none_node.into_view(cx);

        assert!(visible.is_text());
        assert!(empty.is_fragment());
        assert_eq!(empty.children().len(), 0);
    });
}

#[test]
fn custom_view_builds_correctly() {
    struct Counter {
        initial: i32,
    }

    impl View for Counter {
        fn build(self, cx: Scope) -> Node {
            el("Counter", cx)
                .child(text(format!("Value: {}", self.initial)))
                .build_node()
        }
    }

    with_scope(|cx| {
        let node = Counter { initial: 5 }.build(cx);
        assert!(node.is_element());
        assert_eq!(node.children().len(), 1);
    });
}

#[test]
fn vec_view_seq_dynamic() {
    with_scope(|cx| {
        let items: Vec<String> = (0..5).map(|i| format!("Item {i}")).collect();
        let nodes = items.build_seq(cx);
        assert_eq!(nodes.len(), 5);
        for node in &nodes {
            assert!(node.is_text());
        }
    });
}

#[test]
fn element_style_placeholder() {
    with_scope(|cx| {
        let elem = el("Box", cx).style(42u32).build();
        assert!(elem.style().downcast_ref::<u32>().is_some());
        assert_eq!(*elem.style().downcast_ref::<u32>().unwrap(), 42);
    });
}
