//! Integration tests for dusty-style.

use dusty_style::{
    palette::Palette, tokens, BoxShadow, Color, Corners, Edges, FlexDirection, FontStyle,
    FontWeight, JustifyContent, Style,
};

#[test]
fn end_to_end_cascade() {
    // Base theme style
    let base = Style {
        padding: Edges::all(tokens::spacing(2.0)), // 8px
        background: Some(Color::WHITE),
        foreground: Some(Palette::SLATE.get(900).unwrap_or(Color::BLACK)),
        font: FontStyle {
            family: Some("Inter".into()),
            size: Some(14.0),
            weight: Some(FontWeight::NORMAL),
            ..FontStyle::default()
        },
        ..Style::default()
    };

    // Component-level style
    let card = Style {
        padding: Edges::all(tokens::spacing(4.0)), // 16px
        background: Some(Color::WHITE),
        border_radius: Corners::all(tokens::RadiusToken::Lg.to_px()),
        shadow: Some(tokens::ShadowToken::Md.to_shadows().into_owned()),
        flex_direction: Some(FlexDirection::Column),
        ..Style::default()
    };

    // Explicit overrides
    let explicit = Style {
        background: Some(Palette::BLUE.get(50).unwrap_or(Color::WHITE)),
        justify_content: Some(JustifyContent::Center),
        ..Style::default()
    };

    // Hover state
    let hover = Style {
        shadow: Some(tokens::ShadowToken::Lg.to_shadows().into_owned()),
        ..Style::default()
    };

    let result = base.merge(&card).merge(&explicit).merge(&hover);

    // Padding: card wins (16px all sides)
    assert_eq!(result.padding, Edges::all(16.0));

    // Background: explicit wins (blue-50)
    assert_eq!(
        result.background,
        Some(Palette::BLUE.get(50).unwrap_or(Color::WHITE))
    );

    // Foreground: from base (slate-900)
    assert_eq!(
        result.foreground,
        Some(Palette::SLATE.get(900).unwrap_or(Color::BLACK))
    );

    // Border radius: from card
    assert_eq!(result.border_radius, Corners::all(8.0));

    // Shadow: hover replaced card's shadow entirely
    let shadows = result.shadow.as_ref();
    assert!(shadows.is_some());
    assert_eq!(shadows.map(Vec::len), Some(2)); // Lg has 2 layers

    // Flex direction: from card
    assert_eq!(result.flex_direction, Some(FlexDirection::Column));

    // Justify content: from explicit
    assert_eq!(result.justify_content, Some(JustifyContent::Center));

    // Font: family + size + weight from base
    assert_eq!(result.font.family, Some("Inter".into()));
    assert_eq!(result.font.size, Some(14.0));
    assert_eq!(result.font.weight, Some(FontWeight::NORMAL));
}

#[test]
fn style_as_dyn_any_roundtrip() {
    use std::any::Any;

    let style = Style {
        width: Some(200.0),
        height: Some(100.0),
        background: Some(Palette::RED.get(500).unwrap_or(Color::BLACK)),
        shadow: Some(tokens::ShadowToken::Sm.to_shadows().into_owned()),
        ..Style::default()
    };

    // Box as dyn Any (matches dusty-core's Element storage)
    let boxed: Box<dyn Any> = Box::new(style.clone());

    // Downcast back
    let recovered = boxed.downcast_ref::<Style>();
    assert!(recovered.is_some());
    let default = Style::default();
    let recovered = recovered.unwrap_or(&default);
    assert_eq!(recovered.width, Some(200.0));
    assert_eq!(recovered.height, Some(100.0));
    assert_eq!(
        recovered.background,
        Some(Palette::RED.get(500).unwrap_or(Color::BLACK))
    );
}

#[test]
fn palette_to_style_pipeline() {
    // Build a style entirely from palette + tokens
    let style = Style {
        padding: Edges::xy(tokens::spacing(4.0), tokens::spacing(2.0)),
        background: Some(Palette::INDIGO.get(600).unwrap_or(Color::BLACK)),
        foreground: Some(Color::WHITE),
        border_radius: Corners::all(tokens::RadiusToken::Md.to_px()),
        font: FontStyle {
            size: Some(16.0),
            weight: Some(FontWeight::SEMI_BOLD),
            ..FontStyle::default()
        },
        shadow: Some(tokens::ShadowToken::Sm.to_shadows().into_owned()),
        ..Style::default()
    };

    assert_eq!(style.padding.left, Some(16.0)); // spacing(4) = 16
    assert_eq!(style.padding.top, Some(8.0)); // spacing(2) = 8
    assert_eq!(
        style.background,
        Some(Palette::INDIGO.get(600).unwrap_or(Color::BLACK))
    );
    assert_eq!(style.border_radius, Corners::all(6.0)); // RadiusToken::Md = 6
}

#[test]
fn merge_with_empty_shadow_clears_inherited() {
    let base = Style {
        shadow: Some(vec![BoxShadow {
            offset_x: 0.0,
            offset_y: 4.0,
            blur_radius: 8.0,
            spread_radius: 0.0,
            color: Color::BLACK,
            inset: false,
        }]),
        ..Style::default()
    };

    // Explicitly clear shadows
    let clear = Style {
        shadow: Some(vec![]),
        ..Style::default()
    };

    let result = base.merge(&clear);
    assert_eq!(result.shadow, Some(vec![]));
}
