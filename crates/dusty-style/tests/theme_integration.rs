//! Integration tests for Theme and context-based theme propagation.

use dusty_reactive::{
    create_child_scope, create_scope, dispose_runtime, dispose_scope, initialize_runtime,
};
use dusty_style::theme::{provide_theme, use_theme, Theme};
use dusty_style::{Color, Style};

fn with_runtime(f: impl FnOnce()) {
    initialize_runtime();
    f();
    dispose_runtime();
}

#[test]
fn theme_provides_and_retrieves() {
    with_runtime(|| {
        let scope = create_scope(|_s| {
            provide_theme(Theme::dark());
            let theme = use_theme();
            assert_eq!(theme, Theme::dark());
        });
        dispose_scope(scope);
    });
}

#[test]
fn theme_inherited_by_child() {
    with_runtime(|| {
        let scope = create_scope(|p| {
            provide_theme(Theme::dark());
            let _child = create_child_scope(p, |_c| {
                let theme = use_theme();
                assert_eq!(theme, Theme::dark());
            });
        });
        dispose_scope(scope);
    });
}

#[test]
fn theme_child_override_does_not_affect_parent() {
    with_runtime(|| {
        let scope = create_scope(|p| {
            provide_theme(Theme::dark());
            let _child = create_child_scope(p, |_c| {
                provide_theme(Theme::light());
                assert_eq!(use_theme(), Theme::light());
            });
            // Parent still sees dark
            assert_eq!(use_theme(), Theme::dark());
        });
        dispose_scope(scope);
    });
}

#[test]
fn theme_defaults_to_light() {
    with_runtime(|| {
        let scope = create_scope(|_s| {
            let theme = use_theme();
            assert_eq!(theme, Theme::light());
        });
        dispose_scope(scope);
    });
}

#[test]
fn theme_with_style_builder() {
    with_runtime(|| {
        let scope = create_scope(|_s| {
            provide_theme(Theme::dark());
            let theme = use_theme();

            let card = Style::default()
                .p(4.0)
                .bg(theme.surface)
                .text_color(theme.foreground)
                .rounded_lg()
                .border(1.0, theme.border);

            assert_eq!(card.background, Some(theme.surface));
            assert_eq!(card.foreground, Some(theme.foreground));
            assert_eq!(card.border_color, Some(theme.border));
        });
        dispose_scope(scope);
    });
}

#[test]
fn theme_semantic_colors_accessible() {
    let theme = Theme::light();
    // Primary 500 should be Blue 500
    let primary_500 = theme.primary.get(500);
    assert!(primary_500.is_some());
    assert_eq!(primary_500, dusty_style::palette::Palette::BLUE.get(500));

    // Danger 500 should be Red 500
    assert_eq!(
        theme.danger.get(500),
        dusty_style::palette::Palette::RED.get(500)
    );
}

#[test]
fn dark_theme_has_dark_background() {
    let dark = Theme::dark();
    // Background should be very dark (low luminance)
    let bg = dark.background;
    let luminance = bg.r + bg.g + bg.b;
    assert!(
        luminance < 0.3,
        "dark background should have low luminance, got {luminance}"
    );

    // Foreground should be very light
    let fg = dark.foreground;
    let fg_luminance = fg.r + fg.g + fg.b;
    assert!(
        fg_luminance > 2.5,
        "dark foreground should have high luminance, got {fg_luminance}"
    );
}

#[test]
fn light_theme_has_white_background() {
    let light = Theme::light();
    assert_eq!(light.background, Color::WHITE);
}
