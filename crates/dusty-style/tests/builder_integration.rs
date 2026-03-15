//! Integration tests for Style builder methods.

use dusty_style::{
    palette::Palette, tokens, Color, Corners, Edges, FlexDirection, FontWeight, InteractionState,
    JustifyContent, Overflow, Style,
};

#[test]
fn full_builder_chain_produces_correct_style() {
    let style = Style::default()
        .flex_col()
        .p(4.0)
        .gap(2.0)
        .bg_white()
        .text_slate(900)
        .rounded_lg()
        .shadow_md()
        .font_size(14.0)
        .font_bold()
        .font_family("Inter")
        .hover(|s| s.bg_blue(50).shadow_lg())
        .disabled(|s| s.opacity(0.5));

    assert_eq!(style.flex_direction, Some(FlexDirection::Column));
    assert_eq!(style.padding, Edges::all(16.0));
    assert_eq!(style.gap, Some(8.0));
    assert_eq!(style.background, Some(Color::WHITE));
    assert_eq!(style.foreground, Palette::SLATE.get(900));
    assert_eq!(style.border_radius, Corners::all(8.0));
    assert!(style.shadow.is_some());
    assert_eq!(style.font.size, Some(14.0));
    assert_eq!(style.font.weight, Some(FontWeight::BOLD));
    assert_eq!(style.font.family, Some("Inter".into()));
    assert!(style.hover.is_some());
    assert!(style.disabled.is_some());
}

#[test]
fn builder_plus_cascade_merge() {
    let base = Style::default()
        .p(2.0)
        .bg_slate(100)
        .font_size(14.0)
        .font_normal();

    let card = Style::default()
        .p(4.0)
        .bg_white()
        .rounded_lg()
        .shadow_md()
        .flex_col();

    let result = base.merge(&card);

    // Card padding overrides base
    assert_eq!(result.padding, Edges::all(16.0));
    // Card bg overrides base
    assert_eq!(result.background, Some(Color::WHITE));
    // Font comes from base (card didn't set it)
    assert_eq!(result.font.size, Some(14.0));
    // Layout from card
    assert_eq!(result.flex_direction, Some(FlexDirection::Column));
}

#[test]
fn state_resolve_with_builder() {
    let button = Style::default()
        .px(4.0)
        .py(2.0)
        .bg_blue(500)
        .text_white()
        .rounded_md()
        .font_semibold()
        .hover(|s| s.bg_blue(600).shadow_md())
        .active(|s| s.bg_blue(700))
        .disabled(|s| s.opacity(0.5).bg_slate(300));

    // Normal state
    let normal = button.resolve(&InteractionState::default());
    assert_eq!(normal.background, Palette::BLUE.get(500));
    assert!(normal.hover.is_none());

    // Hovered
    let hovered = button.resolve(&InteractionState {
        hovered: true,
        ..InteractionState::default()
    });
    assert_eq!(hovered.background, Palette::BLUE.get(600));
    assert!(hovered.shadow.is_some());

    // Active overrides hover bg
    let active = button.resolve(&InteractionState {
        hovered: true,
        active: true,
        ..InteractionState::default()
    });
    assert_eq!(active.background, Palette::BLUE.get(700));

    // Disabled overrides everything
    let disabled = button.resolve(&InteractionState {
        hovered: true,
        active: true,
        disabled: true,
        ..InteractionState::default()
    });
    assert_eq!(disabled.opacity, Some(0.5));
    assert_eq!(disabled.background, Palette::SLATE.get(300));
}

#[test]
fn state_styles_merge_across_cascade() {
    let base = Style::default().hover(|s| s.bg_blue(600));
    let over = Style::default().hover(|s| s.shadow_md());
    let merged = base.merge(&over);

    // Both bg and shadow should be present in hover
    let hover = merged.hover.as_ref().unwrap();
    assert_eq!(hover.background, Palette::BLUE.get(600));
    assert!(hover.shadow.is_some());
}

#[test]
fn conditional_styling() {
    let is_primary = true;
    let is_large = false;

    let style = Style::default()
        .p(2.0)
        .when(is_primary, |s| s.bg_blue(500).text_white())
        .when(is_large, |s| s.font_size(24.0))
        .apply(|s| s.rounded_md());

    assert_eq!(style.background, Palette::BLUE.get(500));
    assert_eq!(style.foreground, Some(Color::WHITE));
    assert_eq!(style.font.size, None); // is_large was false
    assert_eq!(style.border_radius, Corners::all(6.0));
}

#[test]
fn builder_with_token_equivalence() {
    // Builder methods should produce the same result as manual token usage
    let builder_style = Style::default().p(4.0).rounded_md().shadow_sm();

    let manual_style = Style {
        padding: Edges::all(tokens::spacing(4.0)),
        border_radius: Corners::all(tokens::RadiusToken::Md.to_px()),
        shadow: Some(tokens::ShadowToken::Sm.to_shadows().into_owned()),
        ..Style::default()
    };

    assert_eq!(builder_style.padding, manual_style.padding);
    assert_eq!(builder_style.border_radius, manual_style.border_radius);
    assert_eq!(builder_style.shadow, manual_style.shadow);
}

#[test]
fn overflow_and_opacity_integration() {
    let modal = Style::default()
        .size(400.0)
        .bg_white()
        .rounded_xl()
        .shadow_2xl()
        .overflow_hidden()
        .opacity(0.95);

    assert_eq!(modal.width, Some(400.0));
    assert_eq!(modal.height, Some(400.0));
    assert_eq!(modal.overflow, Some(Overflow::Hidden));
    assert_eq!(modal.opacity, Some(0.95));
}

#[test]
fn flex_layout_integration() {
    let row = Style::default()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(4.0)
        .flex_wrap();

    assert_eq!(row.flex_direction, Some(FlexDirection::Row));
    assert_eq!(row.align_items, Some(dusty_style::AlignItems::Center));
    assert_eq!(row.justify_content, Some(JustifyContent::SpaceBetween));
    assert_eq!(row.gap, Some(16.0));
    assert_eq!(row.flex_wrap, Some(dusty_style::FlexWrap::Wrap));
}
