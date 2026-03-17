#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Theme showcase -- palette colors, light/dark theme, design tokens, button variants.

use dusty::prelude::*;
use dusty::style::tokens::{spacing, RadiusToken, ShadowToken};

fn main() {
    dusty::app("Theme Showcase")
        .width(800.0)
        .height(700.0)
        .root(|cx| {
            let dark_mode = create_signal(false);

            let theme = if dark_mode.get() {
                Theme::dark()
            } else {
                Theme::light()
            };
            provide_theme(theme);

            // Helper: color swatch
            let swatch = |color: Color| -> Node {
                el("Swatch", cx)
                    .style(Style {
                        background: Some(color),
                        width: Some(32.0),
                        height: Some(32.0),
                        border_radius: Corners::all(4.0),
                        ..Style::default()
                    })
                    .build_node()
            };

            // Helper: spacing demo box
            let spacing_box = |key: f32| -> Node {
                let size = spacing(key);
                el("SpacingBox", cx)
                    .style(Style {
                        background: Some(Palette::BLUE.get(200).unwrap()),
                        width: Some(size),
                        height: Some(24.0),
                        border_radius: Corners::all(2.0),
                        ..Style::default()
                    })
                    .build_node()
            };

            // Helper: radius demo box
            let radius_box = |token: RadiusToken| -> Node {
                el("RadiusBox", cx)
                    .style(Style {
                        background: Some(Palette::INDIGO.get(300).unwrap()),
                        width: Some(48.0),
                        height: Some(48.0),
                        border_radius: Corners::all(token.to_px()),
                        ..Style::default()
                    })
                    .build_node()
            };

            // Color palette: a few hues at stop 500
            let palette_row = row![cx;
                swatch(Palette::RED.get(500).unwrap()),
                swatch(Palette::ORANGE.get(500).unwrap()),
                swatch(Palette::AMBER.get(500).unwrap()),
                swatch(Palette::GREEN.get(500).unwrap()),
                swatch(Palette::BLUE.get(500).unwrap()),
                swatch(Palette::INDIGO.get(500).unwrap()),
                swatch(Palette::VIOLET.get(500).unwrap()),
                swatch(Palette::PINK.get(500).unwrap())
            ];

            // Button variants
            let button_row = row![cx;
                Button::new("Primary").build(cx),
                Button::new("Secondary").variant(ButtonVariant::Secondary).build(cx),
                Button::new("Outline").variant(ButtonVariant::Outline).build(cx),
                Button::new("Ghost").variant(ButtonVariant::Ghost).build(cx),
                Button::new("Danger").variant(ButtonVariant::Danger).build(cx),
                Button::new("Disabled").disabled(true).build(cx)
            ];

            // Spacing tokens
            let spacing_row = row![cx;
                spacing_box(1.0),
                spacing_box(2.0),
                spacing_box(4.0),
                spacing_box(8.0),
                spacing_box(16.0)
            ];

            // Radius tokens
            let radius_row = row![cx;
                radius_box(RadiusToken::None),
                radius_box(RadiusToken::Sm),
                radius_box(RadiusToken::Md),
                radius_box(RadiusToken::Lg),
                radius_box(RadiusToken::Xl),
                radius_box(RadiusToken::Full)
            ];

            // Shadow tokens -- show count of layers as text
            let shadow_info = row![cx;
                Text::new(format!("Sm: {} layer", ShadowToken::Sm.to_shadows().len())).build(cx),
                Text::new(format!("Md: {} layers", ShadowToken::Md.to_shadows().len())).build(cx),
                Text::new(format!("Lg: {} layers", ShadowToken::Lg.to_shadows().len())).build(cx),
                Text::new(format!("Xl: {} layers", ShadowToken::Xl.to_shadows().len())).build(cx)
            ];

            // Semantic colors from theme
            let theme = use_theme();
            let semantic_row = row![cx;
                swatch(theme.primary.get(500).unwrap()),
                swatch(theme.danger.get(500).unwrap()),
                swatch(theme.success.get(500).unwrap()),
                swatch(theme.warning.get(500).unwrap()),
                swatch(theme.info.get(500).unwrap())
            ];

            col![cx;
                row![cx;
                    Text::new("Theme Showcase").build(cx),
                    Toggle::new().controlled(dark_mode).label("Dark Mode").build(cx)
                ],
                Divider::horizontal().build(cx),
                Text::new("Color Palette").build(cx),
                palette_row,
                Text::new("Button Variants").build(cx),
                button_row,
                Text::new("Spacing Tokens").build(cx),
                spacing_row,
                Text::new("Border Radius").build(cx),
                radius_row,
                Text::new("Shadows & Semantic Colors").build(cx),
                row![cx; shadow_info, semantic_row]
            ]
        })
        .run()
        .unwrap();
}
