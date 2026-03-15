//! Tests for the App builder API and error handling.

use dusty::{app, DustyError};

#[test]
fn app_creates_builder_with_title() {
    let a = app("My App");
    // Can't directly inspect config, but we can verify it doesn't panic
    drop(a);
}

#[test]
fn builder_method_chaining() {
    let _a = app("Test")
        .width(1024.0)
        .height(768.0)
        .min_size(320.0, 240.0)
        .max_size(1920.0, 1080.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .theme(dusty::style::theme::Theme::dark())
        .root(|_cx| dusty_core::Node::Text(dusty_core::text("hello")));
}

#[test]
fn run_without_root_returns_no_root() {
    let result = app("Test").run();
    assert!(result.is_err());
    match result.unwrap_err() {
        DustyError::NoRoot => {}
        other => panic!("expected NoRoot, got: {other}"),
    }
}

#[test]
fn no_root_error_display() {
    let err = DustyError::NoRoot;
    assert_eq!(err.to_string(), "no root component provided");
}

#[test]
#[ignore] // Requires display server
fn run_with_root_compiles() {
    let result = app("Integration")
        .width(400.0)
        .height(300.0)
        .root(|_cx| dusty_core::Node::Text(dusty_core::text("hello")))
        .run();
    let _ = result;
}
