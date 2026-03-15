use std::cell::Cell;
use std::rc::Rc;

use dusty_core::event::{dispatch_event, ClickEvent, HoverEvent, KeyDownEvent};
use dusty_core::node::ComponentNode;
use dusty_core::{el, text, Key, Modifiers, Node};
use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime, Scope};

fn with_scope(f: impl FnOnce(Scope)) {
    initialize_runtime();
    create_scope(|cx| f(cx)).unwrap();
    dispose_runtime();
}

#[test]
fn click_dispatch_end_to_end() {
    with_scope(|cx| {
        let coords = Rc::new(Cell::new((0.0f64, 0.0f64)));
        let coords_clone = coords.clone();

        let tree = el("App", cx)
            .child(
                el("Panel", cx).child(el("Button", cx).on_click(move |_ctx, e| {
                    coords_clone.set((e.x, e.y));
                })),
            )
            .build_node();

        let handled = dispatch_event(&tree, &[0, 0], &ClickEvent { x: 100.0, y: 200.0 }).unwrap();
        assert!(handled);
        assert_eq!(coords.get(), (100.0, 200.0));
    });
}

#[test]
fn bubbling_order_three_levels() {
    with_scope(|cx| {
        let order = Rc::new(std::cell::RefCell::new(Vec::new()));
        let o1 = order.clone();
        let o2 = order.clone();
        let o3 = order.clone();

        let tree = el("App", cx)
            .on_click(move |_ctx, _e| o1.borrow_mut().push("app"))
            .child(
                el("Panel", cx)
                    .on_click(move |_ctx, _e| o2.borrow_mut().push("panel"))
                    .child(el("Button", cx).on_click(move |_ctx, _e| {
                        o3.borrow_mut().push("button");
                    })),
            )
            .build_node();

        dispatch_event(&tree, &[0, 0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
        assert_eq!(*order.borrow(), vec!["button", "panel", "app"]);
    });
}

#[test]
fn stop_propagation_at_middle_level() {
    with_scope(|cx| {
        let order = Rc::new(std::cell::RefCell::new(Vec::new()));
        let o1 = order.clone();
        let o2 = order.clone();
        let o3 = order.clone();

        let tree = el("App", cx)
            .on_click(move |_ctx, _e| o1.borrow_mut().push("app"))
            .child(
                el("Panel", cx)
                    .on_click(move |ctx, _e| {
                        o2.borrow_mut().push("panel");
                        ctx.stop_propagation();
                    })
                    .child(el("Button", cx).on_click(move |_ctx, _e| {
                        o3.borrow_mut().push("button");
                    })),
            )
            .build_node();

        dispatch_event(&tree, &[0, 0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
        assert_eq!(*order.borrow(), vec!["button", "panel"]);
    });
}

#[test]
fn different_event_types_dont_cross() {
    with_scope(|cx| {
        let click_count = Rc::new(Cell::new(0u32));
        let hover_count = Rc::new(Cell::new(0u32));
        let cc = click_count.clone();
        let hc = hover_count.clone();

        let tree = el("Button", cx)
            .on_click(move |_ctx, _e| cc.set(cc.get() + 1))
            .on_hover(move |_ctx, _e| hc.set(hc.get() + 1))
            .build_node();

        dispatch_event(&tree, &[], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
        assert_eq!(click_count.get(), 1);
        assert_eq!(hover_count.get(), 0);

        dispatch_event(&tree, &[], &HoverEvent { x: 0.0, y: 0.0 }).unwrap();
        assert_eq!(click_count.get(), 1);
        assert_eq!(hover_count.get(), 1);
    });
}

#[test]
fn keyboard_event_dispatch() {
    with_scope(|cx| {
        let received_key = Rc::new(std::cell::RefCell::new(String::new()));
        let rk = received_key.clone();

        let tree = el("Input", cx)
            .on_key_down(move |_ctx, e| {
                *rk.borrow_mut() = e.key.0.clone();
            })
            .build_node();

        let event = KeyDownEvent {
            key: Key("Enter".into()),
            modifiers: Modifiers::default(),
        };
        dispatch_event(&tree, &[], &event).unwrap();
        assert_eq!(*received_key.borrow(), "Enter");
    });
}

#[test]
fn dispatch_through_text_node_sibling() {
    with_scope(|cx| {
        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();

        // Tree: App > [Text("label"), Button]
        // Dispatch to child index 1 (the Button)
        let tree = el("App", cx)
            .child(text("label"))
            .child(el("Button", cx).on_click(move |_ctx, _e| {
                called_clone.set(true);
            }))
            .build_node();

        let handled = dispatch_event(&tree, &[1], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
        assert!(handled);
        assert!(called.get());
    });
}

#[test]
fn dispatch_to_text_node_returns_false() {
    with_scope(|cx| {
        let tree = el("App", cx).child(text("label")).build_node();

        // Path [0] targets the text node — no handlers, not handled
        let handled = dispatch_event(&tree, &[0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
        assert!(!handled);
    });
}

#[test]
fn event_context_path_is_correct_in_handler() {
    with_scope(|cx| {
        let path = Rc::new(std::cell::RefCell::new(Vec::new()));
        let path_clone = path.clone();

        let tree = el("Root", cx)
            .child(el("Child", cx).on_click(move |ctx, _e| {
                *path_clone.borrow_mut() = ctx.target_path().to_vec();
            }))
            .build_node();

        dispatch_event(&tree, &[0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
        assert_eq!(*path.borrow(), vec![0]);
    });
}

#[test]
fn invalid_deep_path() {
    with_scope(|cx| {
        let tree = el("Root", cx).child(el("Child", cx)).build_node();

        // Path [0, 5] — child at index 0 has no children, index 5 is out of bounds
        let result = dispatch_event(&tree, &[0, 5], &ClickEvent { x: 0.0, y: 0.0 });
        assert!(result.is_err());
    });
}

#[test]
fn fragment_child_dispatch() {
    with_scope(|cx| {
        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();

        // Root element with a fragment child containing a button
        let frag = Node::Fragment(vec![el("Button", cx)
            .on_click(move |_ctx, _e| called_clone.set(true))
            .build_node()]);
        let tree = el("Root", cx).child_node(frag).build_node();

        // Path [0] → Fragment, Fragment.children()[0] → Button
        // But dispatch walks Node::children(), which for Fragment returns items.
        // So path [0, 0] should reach the Button inside the fragment.
        let handled = dispatch_event(&tree, &[0, 0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
        assert!(handled);
        assert!(called.get());
    });
}

#[test]
fn component_node_dispatch_traverses_child() {
    with_scope(|cx| {
        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();

        // Root element wrapping a ComponentNode whose single child is a Button
        let button = el("Button", cx)
            .on_click(move |_ctx, _e| called_clone.set(true))
            .build_node();
        let comp = Node::Component(ComponentNode {
            name: "MyComponent",
            child: Box::new(button),
        });
        let tree = el("Root", cx).child_node(comp).build_node();

        // Path [0] → ComponentNode, ComponentNode.children()[0] → Button (index 0)
        let handled = dispatch_event(&tree, &[0, 0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
        assert!(handled);
        assert!(called.get());
    });
}

#[test]
fn component_node_bubbles_through_parent() {
    with_scope(|cx| {
        let order = Rc::new(std::cell::RefCell::new(Vec::new()));
        let o1 = order.clone();
        let o2 = order.clone();

        let button = el("Button", cx)
            .on_click(move |_ctx, _e| o2.borrow_mut().push("button"))
            .build_node();
        let comp = Node::Component(ComponentNode {
            name: "Wrapper",
            child: Box::new(button),
        });
        let tree = el("Root", cx)
            .on_click(move |_ctx, _e| o1.borrow_mut().push("root"))
            .child_node(comp)
            .build_node();

        dispatch_event(&tree, &[0, 0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
        // ComponentNode is transparent — button fires, then root
        assert_eq!(*order.borrow(), vec!["button", "root"]);
    });
}

#[test]
fn stop_immediate_propagation_integration() {
    with_scope(|cx| {
        let count = Rc::new(Cell::new(0u32));
        let c1 = count.clone();
        let c2 = count.clone();
        let root_called = Rc::new(Cell::new(false));
        let rc = root_called.clone();

        let tree = el("Root", cx)
            .on_click(move |_ctx, _e| rc.set(true))
            .child(
                el("Button", cx)
                    .on_click(move |ctx, _e| {
                        c1.set(c1.get() + 1);
                        ctx.stop_immediate_propagation();
                    })
                    .on_click(move |_ctx, _e| {
                        c2.set(c2.get() + 1);
                    }),
            )
            .build_node();

        dispatch_event(&tree, &[0], &ClickEvent { x: 0.0, y: 0.0 }).unwrap();
        // Only the first handler fires; sibling and ancestor are both blocked
        assert_eq!(count.get(), 1);
        assert!(!root_called.get());
    });
}
