//! Verify that re-exports compile and types are accessible through the facade.

#[test]
fn subcrate_modules_accessible() {
    // Verify subcrate module aliases compile
    let _ = dusty::reactive::ReactiveError::NoRuntime;
    let _ = dusty::style::Color::WHITE;
    let _ = dusty::layout::LayoutError::EmptyTree;
    let _ = dusty::a11y::element_role;
    // render, platform, widgets are accessible but require runtime state to construct
}

#[test]
fn prelude_reactive_types_compile() {
    use dusty::prelude::*;

    // Verify reactive types are in scope
    fn _takes_scope(_cx: Scope) {}
    let _: fn(Scope) = _takes_scope;

    // Verify reactive functions exist
    fn _use_batch() {
        let _ = batch(|| 42);
    }
}

#[test]
fn prelude_core_types_compile() {
    use dusty::prelude::*;

    // Node construction via text (returns TextNode, wraps to Node)
    let _node: Node = Node::Text(text("hello"));
    let _node: Node = Node::Text(text_dynamic(|| "dynamic".to_string()));
}

#[test]
fn prelude_core_fragment_compile() {
    use dusty::prelude::*;

    dusty_reactive::initialize_runtime();
    let _ = create_scope(|cx| {
        let _node: Node = fragment(("a", "b"), cx);
    });
    dusty_reactive::dispose_runtime();
}

#[test]
fn prelude_style_types_compile() {
    use dusty::prelude::*;

    let _style = Style::default();
    let _color = Color::WHITE;
    let _dir = FlexDirection::Row;
    let _edges = Edges::all(8.0);
    let _corners = Corners::all(4.0);
}

#[test]
fn prelude_event_types_compile() {
    use dusty::prelude::*;

    // Just verify the types are in scope
    let _: fn() -> Modifiers = Modifiers::default;
    let _key = Key("Enter".into());
}

#[test]
fn prelude_widget_types_compile() {
    use dusty::prelude::*;

    // Display widgets
    let _ = Orientation::Horizontal;
    let _ = SizingMode::Cover;

    // Interactive widgets
    let _ = ButtonVariant::Primary;

    // Container widgets
    let _ = ScrollAxis::Vertical;
}

#[test]
fn prelude_theme_compile() {
    use dusty::prelude::*;

    let _light = Theme::light();
    let _dark = Theme::dark();
}

#[test]
fn root_level_exports_compile() {
    // app function
    let _app = dusty::app("test");

    // Error type
    let _err = dusty::DustyError::NoRoot;

    // Result alias
    let _ok: dusty::Result<i32> = Ok(42);
}

#[test]
fn view_core_alias_accessible() {
    // Verify the alias resolves to dusty_core types
    let _node = dusty::view_core::node::text("hello");
    let _key = dusty::view_core::event::Key("Enter".into());
}

#[test]
fn text_engine_alias_accessible() {
    // Verify the alias resolves to dusty_text types
    let _system = dusty::text_engine::TextSystem::new();
}

#[test]
fn existing_aliases_still_work() {
    // Smoke test that existing aliases didn't break
    let _ = dusty::reactive::initialize_runtime;
    let _ = dusty::style::Style::default();
}

#[test]
fn macro_reexports_compile() {
    use dusty::prelude::*;

    dusty_reactive::initialize_runtime();
    let _ = create_scope(|cx| {
        // col! and row! macros require scope
        let _node: Node = dusty::col![cx;];
        let _node: Node = dusty::row![cx;];
        let _node: Node = dusty::col![cx; Node::Text(text("a")), Node::Text(text("b"))];
        let _node: Node = dusty::row![cx; Node::Text(text("a")), Node::Text(text("b"))];
    });
    dusty_reactive::dispose_runtime();
}
