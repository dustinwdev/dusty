use dusty_core::event::{
    dispatch_event, BlurEvent, FocusEvent, KeyDownEvent, Modifiers, TextInputEvent,
};
use dusty_core::node::Node;
use dusty_core::view::View;
use dusty_core::Element;
use dusty_reactive::{create_scope, create_signal, dispose_runtime, initialize_runtime};
use dusty_widgets::text_input::TextInputState;
use dusty_widgets::TextInput;

fn extract_element(node: &Node) -> &Element {
    match node {
        Node::Component(comp) => match &*comp.child {
            Node::Element(el) => el,
            _ => panic!("expected Element inside Component"),
        },
        _ => panic!("expected Component node"),
    }
}

fn inner_node(node: &Node) -> &Node {
    match node {
        Node::Component(comp) => &*comp.child,
        _ => panic!("expected Component node"),
    }
}

fn get_state(node: &Node) -> &TextInputState {
    let el = extract_element(node);
    el.custom_data()
        .downcast_ref::<TextInputState>()
        .expect("TextInputState in custom_data")
}

fn key(name: &str) -> dusty_core::Key {
    dusty_core::Key(name.to_string())
}

#[test]
fn type_sequence_accumulates() {
    initialize_runtime();
    create_scope(|cx| {
        let node = TextInput::new().build(cx);
        let inner = inner_node(&node);
        let state = get_state(&node);

        dispatch_event(
            inner,
            &[],
            &TextInputEvent {
                text: "h".to_string(),
            },
        )
        .unwrap();
        dispatch_event(
            inner,
            &[],
            &TextInputEvent {
                text: "i".to_string(),
            },
        )
        .unwrap();

        let val = state.value.get();
        assert_eq!(val, "hi");
        assert_eq!(state.cursor.get(), 2);
    });
    dispose_runtime();
}

#[test]
fn controlled_round_trip() {
    initialize_runtime();
    create_scope(|cx| {
        let sig = create_signal("init".to_string());
        let node = TextInput::new().controlled(sig).build(cx);
        let inner = inner_node(&node);

        // Type at end (cursor starts at len)
        dispatch_event(
            inner,
            &[],
            &TextInputEvent {
                text: "!".to_string(),
            },
        )
        .unwrap();

        assert_eq!(sig.get(), "init!");
    });
    dispose_runtime();
}

#[test]
fn backspace_at_position_zero_is_noop() {
    initialize_runtime();
    create_scope(|cx| {
        let node = TextInput::new().value("abc").build(cx);
        let inner = inner_node(&node);
        let state = get_state(&node);

        // Move cursor to 0
        state.cursor.set(0);

        dispatch_event(
            inner,
            &[],
            &KeyDownEvent {
                key: key("Backspace"),
                modifiers: Modifiers::default(),
            },
        )
        .unwrap();

        assert_eq!(state.value.get(), "abc");
        assert_eq!(state.cursor.get(), 0);
    });
    dispose_runtime();
}

#[test]
fn backspace_deletes_before_cursor() {
    initialize_runtime();
    create_scope(|cx| {
        let node = TextInput::new().value("abc").build(cx);
        let inner = inner_node(&node);
        let state = get_state(&node);

        // Cursor at end (3)
        dispatch_event(
            inner,
            &[],
            &KeyDownEvent {
                key: key("Backspace"),
                modifiers: Modifiers::default(),
            },
        )
        .unwrap();

        assert_eq!(state.value.get(), "ab");
        assert_eq!(state.cursor.get(), 2);
    });
    dispose_runtime();
}

#[test]
fn delete_key_deletes_after_cursor() {
    initialize_runtime();
    create_scope(|cx| {
        let node = TextInput::new().value("abc").build(cx);
        let inner = inner_node(&node);
        let state = get_state(&node);

        state.cursor.set(0);
        dispatch_event(
            inner,
            &[],
            &KeyDownEvent {
                key: key("Delete"),
                modifiers: Modifiers::default(),
            },
        )
        .unwrap();

        assert_eq!(state.value.get(), "bc");
        assert_eq!(state.cursor.get(), 0);
    });
    dispose_runtime();
}

#[test]
fn arrow_keys_move_cursor() {
    initialize_runtime();
    create_scope(|cx| {
        let node = TextInput::new().value("abc").build(cx);
        let inner = inner_node(&node);
        let state = get_state(&node);

        // Start at end (3)
        dispatch_event(
            inner,
            &[],
            &KeyDownEvent {
                key: key("ArrowLeft"),
                modifiers: Modifiers::default(),
            },
        )
        .unwrap();
        assert_eq!(state.cursor.get(), 2);

        dispatch_event(
            inner,
            &[],
            &KeyDownEvent {
                key: key("ArrowRight"),
                modifiers: Modifiers::default(),
            },
        )
        .unwrap();
        assert_eq!(state.cursor.get(), 3);
    });
    dispose_runtime();
}

#[test]
fn home_and_end() {
    initialize_runtime();
    create_scope(|cx| {
        let node = TextInput::new().value("hello").build(cx);
        let inner = inner_node(&node);
        let state = get_state(&node);

        dispatch_event(
            inner,
            &[],
            &KeyDownEvent {
                key: key("Home"),
                modifiers: Modifiers::default(),
            },
        )
        .unwrap();
        assert_eq!(state.cursor.get(), 0);

        dispatch_event(
            inner,
            &[],
            &KeyDownEvent {
                key: key("End"),
                modifiers: Modifiers::default(),
            },
        )
        .unwrap();
        assert_eq!(state.cursor.get(), 5);
    });
    dispose_runtime();
}

#[test]
fn enter_fires_on_submit() {
    initialize_runtime();
    create_scope(|cx| {
        let submitted = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
        let submitted_clone = submitted.clone();

        let node = TextInput::new()
            .value("done")
            .on_submit(move |text| {
                *submitted_clone.borrow_mut() = text.to_string();
            })
            .build(cx);
        let inner = inner_node(&node);

        dispatch_event(
            inner,
            &[],
            &KeyDownEvent {
                key: key("Enter"),
                modifiers: Modifiers::default(),
            },
        )
        .unwrap();

        assert_eq!(*submitted.borrow(), "done");
    });
    dispose_runtime();
}

#[test]
fn max_length_enforced() {
    initialize_runtime();
    create_scope(|cx| {
        let node = TextInput::new().max_length(3).build(cx);
        let inner = inner_node(&node);
        let state = get_state(&node);

        // Type "abcd" -- only first 3 chars should be accepted
        for ch in &["a", "b", "c", "d"] {
            dispatch_event(
                inner,
                &[],
                &TextInputEvent {
                    text: ch.to_string(),
                },
            )
            .unwrap();
        }

        assert_eq!(state.value.get(), "abc");
    });
    dispose_runtime();
}

#[test]
fn read_only_allows_focus_but_not_mutation() {
    initialize_runtime();
    create_scope(|cx| {
        let node = TextInput::new().value("fixed").read_only(true).build(cx);
        let inner = inner_node(&node);
        let state = get_state(&node);

        // Focus should work
        dispatch_event(inner, &[], &FocusEvent).unwrap();
        assert_eq!(state.focused.get(), true);

        // Text input should be suppressed
        dispatch_event(
            inner,
            &[],
            &TextInputEvent {
                text: "x".to_string(),
            },
        )
        .unwrap();
        assert_eq!(state.value.get(), "fixed");

        // Backspace should be suppressed
        dispatch_event(
            inner,
            &[],
            &KeyDownEvent {
                key: key("Backspace"),
                modifiers: Modifiers::default(),
            },
        )
        .unwrap();
        assert_eq!(state.value.get(), "fixed");
    });
    dispose_runtime();
}

#[test]
fn focus_blur_tracking() {
    initialize_runtime();
    create_scope(|cx| {
        let node = TextInput::new().build(cx);
        let inner = inner_node(&node);
        let state = get_state(&node);

        assert_eq!(state.focused.get(), false);

        dispatch_event(inner, &[], &FocusEvent).unwrap();
        assert_eq!(state.focused.get(), true);

        dispatch_event(inner, &[], &BlurEvent).unwrap();
        assert_eq!(state.focused.get(), false);
    });
    dispose_runtime();
}

#[test]
fn on_change_fires() {
    initialize_runtime();
    create_scope(|cx| {
        let changed = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
        let changed_clone = changed.clone();

        let node = TextInput::new()
            .on_change(move |text| {
                *changed_clone.borrow_mut() = text.to_string();
            })
            .build(cx);
        let inner = inner_node(&node);

        dispatch_event(
            inner,
            &[],
            &TextInputEvent {
                text: "x".to_string(),
            },
        )
        .unwrap();

        assert_eq!(*changed.borrow(), "x");
    });
    dispose_runtime();
}
