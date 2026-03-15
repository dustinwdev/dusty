//! Tailwind-inspired builder methods for [`Style`].
//!
//! Many methods call non-const functions (spacing scale, token lookups), so
//! making a subset const would be inconsistent across the builder API.
#![allow(clippy::missing_const_for_fn)]

use crate::gradient::{ColorStop, GradientDirection, LinearGradient};
use crate::palette::Palette;
use crate::style::{AlignItems, AlignSelf, FlexDirection, FlexWrap, JustifyContent, Overflow};
use crate::tokens::{self, RadiusToken, ShadowToken};
use crate::{Color, Corners, Edges, FontSlant, FontWeight, Style};

// ---------------------------------------------------------------------------
// Spacing (14 methods)
// ---------------------------------------------------------------------------

impl Style {
    /// Set all padding from a spacing scale key (`key * 4.0` px).
    #[must_use]
    pub fn p(mut self, key: f32) -> Self {
        self.padding = Edges::all(tokens::spacing(key));
        self
    }

    /// Set horizontal padding (left + right) from a spacing scale key.
    #[must_use]
    pub fn px(mut self, key: f32) -> Self {
        let val = tokens::spacing(key);
        self.padding.left = Some(val);
        self.padding.right = Some(val);
        self
    }

    /// Set vertical padding (top + bottom) from a spacing scale key.
    #[must_use]
    pub fn py(mut self, key: f32) -> Self {
        let val = tokens::spacing(key);
        self.padding.top = Some(val);
        self.padding.bottom = Some(val);
        self
    }

    /// Set top padding from a spacing scale key.
    #[must_use]
    pub fn pt(mut self, key: f32) -> Self {
        self.padding.top = Some(tokens::spacing(key));
        self
    }

    /// Set right padding from a spacing scale key.
    #[must_use]
    pub fn pr(mut self, key: f32) -> Self {
        self.padding.right = Some(tokens::spacing(key));
        self
    }

    /// Set bottom padding from a spacing scale key.
    #[must_use]
    pub fn pb(mut self, key: f32) -> Self {
        self.padding.bottom = Some(tokens::spacing(key));
        self
    }

    /// Set left padding from a spacing scale key.
    #[must_use]
    pub fn pl(mut self, key: f32) -> Self {
        self.padding.left = Some(tokens::spacing(key));
        self
    }

    /// Set all margin from a spacing scale key (`key * 4.0` px).
    #[must_use]
    pub fn m(mut self, key: f32) -> Self {
        self.margin = Edges::all(tokens::spacing(key));
        self
    }

    /// Set horizontal margin (left + right) from a spacing scale key.
    #[must_use]
    pub fn mx(mut self, key: f32) -> Self {
        let val = tokens::spacing(key);
        self.margin.left = Some(val);
        self.margin.right = Some(val);
        self
    }

    /// Set vertical margin (top + bottom) from a spacing scale key.
    #[must_use]
    pub fn my(mut self, key: f32) -> Self {
        let val = tokens::spacing(key);
        self.margin.top = Some(val);
        self.margin.bottom = Some(val);
        self
    }

    /// Set top margin from a spacing scale key.
    #[must_use]
    pub fn mt(mut self, key: f32) -> Self {
        self.margin.top = Some(tokens::spacing(key));
        self
    }

    /// Set right margin from a spacing scale key.
    #[must_use]
    pub fn mr(mut self, key: f32) -> Self {
        self.margin.right = Some(tokens::spacing(key));
        self
    }

    /// Set bottom margin from a spacing scale key.
    #[must_use]
    pub fn mb(mut self, key: f32) -> Self {
        self.margin.bottom = Some(tokens::spacing(key));
        self
    }

    /// Set left margin from a spacing scale key.
    #[must_use]
    pub fn ml(mut self, key: f32) -> Self {
        self.margin.left = Some(tokens::spacing(key));
        self
    }
}

// ---------------------------------------------------------------------------
// Sizing (7 methods)
// ---------------------------------------------------------------------------

impl Style {
    /// Set explicit width in pixels.
    #[must_use]
    pub fn w(mut self, px: f32) -> Self {
        self.width = Some(px);
        self
    }

    /// Set explicit height in pixels.
    #[must_use]
    pub fn h(mut self, px: f32) -> Self {
        self.height = Some(px);
        self
    }

    /// Set both width and height in pixels.
    #[must_use]
    pub fn size(mut self, px: f32) -> Self {
        self.width = Some(px);
        self.height = Some(px);
        self
    }

    /// Set minimum width in pixels.
    #[must_use]
    pub fn min_w(mut self, px: f32) -> Self {
        self.min_width = Some(px);
        self
    }

    /// Set minimum height in pixels.
    #[must_use]
    pub fn min_h(mut self, px: f32) -> Self {
        self.min_height = Some(px);
        self
    }

    /// Set maximum width in pixels.
    #[must_use]
    pub fn max_w(mut self, px: f32) -> Self {
        self.max_width = Some(px);
        self
    }

    /// Set maximum height in pixels.
    #[must_use]
    pub fn max_h(mut self, px: f32) -> Self {
        self.max_height = Some(px);
        self
    }
}

// ---------------------------------------------------------------------------
// Flex layout (27 methods)
// ---------------------------------------------------------------------------

impl Style {
    /// Set flex direction to row (left-to-right).
    #[must_use]
    pub fn flex_row(mut self) -> Self {
        self.flex_direction = Some(FlexDirection::Row);
        self
    }

    /// Set flex direction to column (top-to-bottom).
    #[must_use]
    pub fn flex_col(mut self) -> Self {
        self.flex_direction = Some(FlexDirection::Column);
        self
    }

    /// Set flex direction to row-reverse (right-to-left).
    #[must_use]
    pub fn flex_row_reverse(mut self) -> Self {
        self.flex_direction = Some(FlexDirection::RowReverse);
        self
    }

    /// Set flex direction to column-reverse (bottom-to-top).
    #[must_use]
    pub fn flex_col_reverse(mut self) -> Self {
        self.flex_direction = Some(FlexDirection::ColumnReverse);
        self
    }

    /// Enable flex wrapping.
    #[must_use]
    pub fn flex_wrap(mut self) -> Self {
        self.flex_wrap = Some(FlexWrap::Wrap);
        self
    }

    /// Disable flex wrapping.
    #[must_use]
    pub fn flex_nowrap(mut self) -> Self {
        self.flex_wrap = Some(FlexWrap::NoWrap);
        self
    }

    /// Enable reverse flex wrapping.
    #[must_use]
    pub fn flex_wrap_reverse(mut self) -> Self {
        self.flex_wrap = Some(FlexWrap::WrapReverse);
        self
    }

    /// Align items to the start of the cross axis.
    #[must_use]
    pub fn items_start(mut self) -> Self {
        self.align_items = Some(AlignItems::FlexStart);
        self
    }

    /// Align items to the end of the cross axis.
    #[must_use]
    pub fn items_end(mut self) -> Self {
        self.align_items = Some(AlignItems::FlexEnd);
        self
    }

    /// Center items along the cross axis.
    #[must_use]
    pub fn items_center(mut self) -> Self {
        self.align_items = Some(AlignItems::Center);
        self
    }

    /// Align items to the text baseline.
    #[must_use]
    pub fn items_baseline(mut self) -> Self {
        self.align_items = Some(AlignItems::Baseline);
        self
    }

    /// Stretch items to fill the container cross axis.
    #[must_use]
    pub fn items_stretch(mut self) -> Self {
        self.align_items = Some(AlignItems::Stretch);
        self
    }

    /// Pack items toward the start of the main axis.
    #[must_use]
    pub fn justify_start(mut self) -> Self {
        self.justify_content = Some(JustifyContent::FlexStart);
        self
    }

    /// Pack items toward the end of the main axis.
    #[must_use]
    pub fn justify_end(mut self) -> Self {
        self.justify_content = Some(JustifyContent::FlexEnd);
        self
    }

    /// Center items along the main axis.
    #[must_use]
    pub fn justify_center(mut self) -> Self {
        self.justify_content = Some(JustifyContent::Center);
        self
    }

    /// Distribute items with equal space between.
    #[must_use]
    pub fn justify_between(mut self) -> Self {
        self.justify_content = Some(JustifyContent::SpaceBetween);
        self
    }

    /// Distribute items with equal space around.
    #[must_use]
    pub fn justify_around(mut self) -> Self {
        self.justify_content = Some(JustifyContent::SpaceAround);
        self
    }

    /// Distribute items with equal space between and around.
    #[must_use]
    pub fn justify_evenly(mut self) -> Self {
        self.justify_content = Some(JustifyContent::SpaceEvenly);
        self
    }

    /// Set align-self to auto (inherit from parent).
    #[must_use]
    pub fn self_auto(mut self) -> Self {
        self.align_self = Some(AlignSelf::Auto);
        self
    }

    /// Set align-self to start.
    #[must_use]
    pub fn self_start(mut self) -> Self {
        self.align_self = Some(AlignSelf::FlexStart);
        self
    }

    /// Set align-self to end.
    #[must_use]
    pub fn self_end(mut self) -> Self {
        self.align_self = Some(AlignSelf::FlexEnd);
        self
    }

    /// Set align-self to center.
    #[must_use]
    pub fn self_center(mut self) -> Self {
        self.align_self = Some(AlignSelf::Center);
        self
    }

    /// Set align-self to baseline.
    #[must_use]
    pub fn self_baseline(mut self) -> Self {
        self.align_self = Some(AlignSelf::Baseline);
        self
    }

    /// Set align-self to stretch.
    #[must_use]
    pub fn self_stretch(mut self) -> Self {
        self.align_self = Some(AlignSelf::Stretch);
        self
    }

    /// Set gap (both axes) from a spacing scale key.
    #[must_use]
    pub fn gap(mut self, key: f32) -> Self {
        self.gap = Some(tokens::spacing(key));
        self
    }

    /// Set row gap from a spacing scale key.
    #[must_use]
    pub fn row_gap(mut self, key: f32) -> Self {
        self.row_gap = Some(tokens::spacing(key));
        self
    }

    /// Set column gap from a spacing scale key.
    #[must_use]
    pub fn col_gap(mut self, key: f32) -> Self {
        self.column_gap = Some(tokens::spacing(key));
        self
    }

    /// Set flex grow factor.
    #[must_use]
    pub fn grow(mut self, factor: f32) -> Self {
        self.flex_grow = Some(factor);
        self
    }

    /// Set flex shrink factor.
    #[must_use]
    pub fn shrink(mut self, factor: f32) -> Self {
        self.flex_shrink = Some(factor);
        self
    }

    /// Set flex basis in pixels.
    #[must_use]
    pub fn basis(mut self, px: f32) -> Self {
        self.flex_basis = Some(px);
        self
    }
}

// ---------------------------------------------------------------------------
// Color — base (7 methods)
// ---------------------------------------------------------------------------

impl Style {
    /// Set background color.
    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Set foreground (text) color.
    #[must_use]
    pub fn text_color(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    /// Set background to white.
    #[must_use]
    pub fn bg_white(self) -> Self {
        self.bg(Color::WHITE)
    }

    /// Set background to black.
    #[must_use]
    pub fn bg_black(self) -> Self {
        self.bg(Color::BLACK)
    }

    /// Set background to transparent.
    #[must_use]
    pub fn bg_transparent(self) -> Self {
        self.bg(Color::TRANSPARENT)
    }

    /// Set text color to white.
    #[must_use]
    pub fn text_white(self) -> Self {
        self.text_color(Color::WHITE)
    }

    /// Set text color to black.
    #[must_use]
    pub fn text_black(self) -> Self {
        self.text_color(Color::BLACK)
    }
}

// ---------------------------------------------------------------------------
// Color — palette (44 methods via macro)
// ---------------------------------------------------------------------------

macro_rules! impl_hue_methods {
    ($($bg_name:ident, $text_name:ident, $hue:expr);* $(;)?) => {
        impl Style {
            $(
                /// Sets background to the given stop of this hue's color scale.
                /// Invalid stops panic in debug, no-op in release.
                #[must_use]
                pub fn $bg_name(self, stop: u16) -> Self {
                    match $hue.get(stop) {
                        Some(color) => self.bg(color),
                        None => {
                            debug_assert!(false, "invalid palette stop: {stop}");
                            self
                        }
                    }
                }

                /// Sets foreground (text) color to the given stop of this hue's color scale.
                /// Invalid stops panic in debug, no-op in release.
                #[must_use]
                pub fn $text_name(self, stop: u16) -> Self {
                    match $hue.get(stop) {
                        Some(color) => self.text_color(color),
                        None => {
                            debug_assert!(false, "invalid palette stop: {stop}");
                            self
                        }
                    }
                }
            )*
        }
    };
}

impl_hue_methods! {
    bg_slate, text_slate, Palette::SLATE;
    bg_gray, text_gray, Palette::GRAY;
    bg_zinc, text_zinc, Palette::ZINC;
    bg_neutral, text_neutral, Palette::NEUTRAL;
    bg_stone, text_stone, Palette::STONE;
    bg_red, text_red, Palette::RED;
    bg_orange, text_orange, Palette::ORANGE;
    bg_amber, text_amber, Palette::AMBER;
    bg_yellow, text_yellow, Palette::YELLOW;
    bg_lime, text_lime, Palette::LIME;
    bg_green, text_green, Palette::GREEN;
    bg_emerald, text_emerald, Palette::EMERALD;
    bg_teal, text_teal, Palette::TEAL;
    bg_cyan, text_cyan, Palette::CYAN;
    bg_sky, text_sky, Palette::SKY;
    bg_blue, text_blue, Palette::BLUE;
    bg_indigo, text_indigo, Palette::INDIGO;
    bg_violet, text_violet, Palette::VIOLET;
    bg_purple, text_purple, Palette::PURPLE;
    bg_fuchsia, text_fuchsia, Palette::FUCHSIA;
    bg_pink, text_pink, Palette::PINK;
    bg_rose, text_rose, Palette::ROSE;
}

// ---------------------------------------------------------------------------
// Border (7 methods)
// ---------------------------------------------------------------------------

impl Style {
    /// Set border width and color.
    #[must_use]
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.border_width = Edges::all(width);
        self.border_color = Some(color);
        self
    }

    /// Set border color.
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Set uniform border width in pixels.
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub fn border_width(mut self, px: f32) -> Self {
        self.border_width = Edges::all(px);
        self
    }

    /// Set top border width in pixels.
    #[must_use]
    pub fn border_t(mut self, px: f32) -> Self {
        self.border_width.top = Some(px);
        self
    }

    /// Set right border width in pixels.
    #[must_use]
    pub fn border_r(mut self, px: f32) -> Self {
        self.border_width.right = Some(px);
        self
    }

    /// Set bottom border width in pixels.
    #[must_use]
    pub fn border_b(mut self, px: f32) -> Self {
        self.border_width.bottom = Some(px);
        self
    }

    /// Set left border width in pixels.
    #[must_use]
    pub fn border_l(mut self, px: f32) -> Self {
        self.border_width.left = Some(px);
        self
    }
}

// ---------------------------------------------------------------------------
// Radius (9 methods)
// ---------------------------------------------------------------------------

impl Style {
    /// Set uniform border radius in pixels.
    #[must_use]
    pub fn rounded(mut self, px: f32) -> Self {
        self.border_radius = Corners::all(px);
        self
    }

    /// Remove border radius.
    #[must_use]
    pub fn rounded_none(self) -> Self {
        self.rounded(RadiusToken::None.to_px())
    }

    /// Set border radius to small (2px).
    #[must_use]
    pub fn rounded_sm(self) -> Self {
        self.rounded(RadiusToken::Sm.to_px())
    }

    /// Set border radius to medium (6px).
    #[must_use]
    pub fn rounded_md(self) -> Self {
        self.rounded(RadiusToken::Md.to_px())
    }

    /// Set border radius to large (8px).
    #[must_use]
    pub fn rounded_lg(self) -> Self {
        self.rounded(RadiusToken::Lg.to_px())
    }

    /// Set border radius to extra-large (12px).
    #[must_use]
    pub fn rounded_xl(self) -> Self {
        self.rounded(RadiusToken::Xl.to_px())
    }

    /// Set border radius to 2xl (16px).
    #[must_use]
    pub fn rounded_2xl(self) -> Self {
        self.rounded(RadiusToken::Xl2.to_px())
    }

    /// Set border radius to 3xl (24px).
    #[must_use]
    pub fn rounded_3xl(self) -> Self {
        self.rounded(RadiusToken::Xl3.to_px())
    }

    /// Set border radius to full/pill (9999px).
    #[must_use]
    pub fn rounded_full(self) -> Self {
        self.rounded(RadiusToken::Full.to_px())
    }
}

// ---------------------------------------------------------------------------
// Shadow (7 methods)
// ---------------------------------------------------------------------------

impl Style {
    /// Remove shadows.
    #[must_use]
    pub fn shadow_none(mut self) -> Self {
        self.shadow = Some(ShadowToken::None.to_shadows().into_owned());
        self
    }

    /// Set small shadow.
    #[must_use]
    pub fn shadow_sm(mut self) -> Self {
        self.shadow = Some(ShadowToken::Sm.to_shadows().into_owned());
        self
    }

    /// Set medium shadow.
    #[must_use]
    pub fn shadow_md(mut self) -> Self {
        self.shadow = Some(ShadowToken::Md.to_shadows().into_owned());
        self
    }

    /// Set large shadow.
    #[must_use]
    pub fn shadow_lg(mut self) -> Self {
        self.shadow = Some(ShadowToken::Lg.to_shadows().into_owned());
        self
    }

    /// Set extra-large shadow.
    #[must_use]
    pub fn shadow_xl(mut self) -> Self {
        self.shadow = Some(ShadowToken::Xl.to_shadows().into_owned());
        self
    }

    /// Set 2xl shadow.
    #[must_use]
    pub fn shadow_2xl(mut self) -> Self {
        self.shadow = Some(ShadowToken::Xl2.to_shadows().into_owned());
        self
    }

    /// Set inner (inset) shadow.
    #[must_use]
    pub fn shadow_inner(mut self) -> Self {
        self.shadow = Some(ShadowToken::Inner.to_shadows().into_owned());
        self
    }
}

// ---------------------------------------------------------------------------
// Typography (13 methods)
// ---------------------------------------------------------------------------

impl Style {
    /// Set font size in pixels.
    #[must_use]
    pub fn font_size(mut self, px: f32) -> Self {
        self.font.size = Some(px);
        self
    }

    /// Set font family.
    #[must_use]
    pub fn font_family(mut self, family: impl Into<std::sync::Arc<str>>) -> Self {
        self.font.family = Some(family.into());
        self
    }

    /// Set font weight to thin (100).
    #[must_use]
    pub fn font_thin(mut self) -> Self {
        self.font.weight = Some(FontWeight::THIN);
        self
    }

    /// Set font weight to light (300).
    #[must_use]
    pub fn font_light(mut self) -> Self {
        self.font.weight = Some(FontWeight::LIGHT);
        self
    }

    /// Set font weight to normal (400).
    #[must_use]
    pub fn font_normal(mut self) -> Self {
        self.font.weight = Some(FontWeight::NORMAL);
        self
    }

    /// Set font weight to medium (500).
    #[must_use]
    pub fn font_medium(mut self) -> Self {
        self.font.weight = Some(FontWeight::MEDIUM);
        self
    }

    /// Set font weight to semibold (600).
    #[must_use]
    pub fn font_semibold(mut self) -> Self {
        self.font.weight = Some(FontWeight::SEMI_BOLD);
        self
    }

    /// Set font weight to bold (700).
    #[must_use]
    pub fn font_bold(mut self) -> Self {
        self.font.weight = Some(FontWeight::BOLD);
        self
    }

    /// Set font weight to extrabold (800).
    #[must_use]
    pub fn font_extrabold(mut self) -> Self {
        self.font.weight = Some(FontWeight::EXTRA_BOLD);
        self
    }

    /// Set font weight to black (900).
    #[must_use]
    pub fn font_black(mut self) -> Self {
        self.font.weight = Some(FontWeight::BLACK);
        self
    }

    /// Set font slant to italic.
    #[must_use]
    pub fn italic(mut self) -> Self {
        self.font.slant = Some(FontSlant::Italic);
        self
    }

    /// Set line height multiplier.
    #[must_use]
    pub fn leading(mut self, multiplier: f32) -> Self {
        self.font.line_height = Some(multiplier);
        self
    }

    /// Set letter spacing in pixels.
    #[must_use]
    pub fn tracking(mut self, px: f32) -> Self {
        self.font.letter_spacing = Some(px);
        self
    }
}

// ---------------------------------------------------------------------------
// Overflow + Opacity (4 methods)
// ---------------------------------------------------------------------------

impl Style {
    /// Set opacity (0.0–1.0).
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub fn opacity(mut self, value: f32) -> Self {
        debug_assert!(
            (0.0..=1.0).contains(&value),
            "opacity out of range: {value}"
        );
        self.opacity = Some(value);
        self
    }

    /// Set overflow to hidden (clipped, no scrollbars).
    #[must_use]
    pub fn overflow_hidden(mut self) -> Self {
        self.overflow = Some(Overflow::Hidden);
        self
    }

    /// Set overflow to scroll (always show scrollbars).
    #[must_use]
    pub fn overflow_scroll(mut self) -> Self {
        self.overflow = Some(Overflow::Scroll);
        self
    }

    /// Set overflow to auto (scrollbars when needed).
    #[must_use]
    pub fn overflow_auto(mut self) -> Self {
        self.overflow = Some(Overflow::Auto);
        self
    }

    /// Set overflow to visible (content renders outside bounds).
    #[must_use]
    pub fn overflow_visible(mut self) -> Self {
        self.overflow = Some(Overflow::Visible);
        self
    }
}

// ---------------------------------------------------------------------------
// State modifiers (4 methods) + Conditional helpers (2 methods)
// ---------------------------------------------------------------------------

impl Style {
    /// Apply style overrides for the hover state.
    #[must_use]
    pub fn hover(mut self, f: impl FnOnce(Self) -> Self) -> Self {
        self.hover = Some(Box::new(f(Self::default())));
        self
    }

    /// Apply style overrides for the focus state.
    #[must_use]
    pub fn focus(mut self, f: impl FnOnce(Self) -> Self) -> Self {
        self.focus = Some(Box::new(f(Self::default())));
        self
    }

    /// Apply style overrides for the active (pressed) state.
    #[must_use]
    pub fn active(mut self, f: impl FnOnce(Self) -> Self) -> Self {
        self.active = Some(Box::new(f(Self::default())));
        self
    }

    /// Apply style overrides for the disabled state.
    #[must_use]
    pub fn disabled(mut self, f: impl FnOnce(Self) -> Self) -> Self {
        self.disabled = Some(Box::new(f(Self::default())));
        self
    }

    /// Conditionally apply a transformation. No-op when `condition` is false.
    #[must_use]
    pub fn when(self, condition: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if condition {
            f(self)
        } else {
            self
        }
    }

    /// Always apply a transformation. Useful for grouping builder chains.
    #[must_use]
    pub fn apply(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
    }
}

// ---------------------------------------------------------------------------
// Gradient (6 methods)
// ---------------------------------------------------------------------------

impl Style {
    /// Set a linear gradient background flowing left-to-right.
    #[must_use]
    pub fn bg_gradient_to_r(mut self, stops: Vec<ColorStop>) -> Self {
        self.background_gradient = Some(LinearGradient {
            direction: GradientDirection::ToRight,
            stops,
        });
        self
    }

    /// Set a linear gradient background flowing right-to-left.
    #[must_use]
    pub fn bg_gradient_to_l(mut self, stops: Vec<ColorStop>) -> Self {
        self.background_gradient = Some(LinearGradient {
            direction: GradientDirection::ToLeft,
            stops,
        });
        self
    }

    /// Set a linear gradient background flowing bottom-to-top.
    #[must_use]
    pub fn bg_gradient_to_t(mut self, stops: Vec<ColorStop>) -> Self {
        self.background_gradient = Some(LinearGradient {
            direction: GradientDirection::ToTop,
            stops,
        });
        self
    }

    /// Set a linear gradient background flowing top-to-bottom.
    #[must_use]
    pub fn bg_gradient_to_b(mut self, stops: Vec<ColorStop>) -> Self {
        self.background_gradient = Some(LinearGradient {
            direction: GradientDirection::ToBottom,
            stops,
        });
        self
    }

    /// Set a linear gradient background at an arbitrary angle.
    #[must_use]
    pub fn bg_gradient_angle(mut self, degrees: f32, stops: Vec<ColorStop>) -> Self {
        self.background_gradient = Some(LinearGradient {
            direction: GradientDirection::Angle(degrees),
            stops,
        });
        self
    }

    /// Set a linear gradient background with full control.
    #[must_use]
    pub fn bg_gradient(mut self, gradient: LinearGradient) -> Self {
        self.background_gradient = Some(gradient);
        self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Spacing --

    #[test]
    fn p_sets_all_padding() {
        let s = Style::default().p(4.0);
        assert_eq!(s.padding, Edges::all(16.0));
    }

    #[test]
    fn px_sets_horizontal_padding() {
        let s = Style::default().px(2.0);
        assert_eq!(s.padding.left, Some(8.0));
        assert_eq!(s.padding.right, Some(8.0));
        assert_eq!(s.padding.top, None);
    }

    #[test]
    fn py_sets_vertical_padding() {
        let s = Style::default().py(3.0);
        assert_eq!(s.padding.top, Some(12.0));
        assert_eq!(s.padding.bottom, Some(12.0));
        assert_eq!(s.padding.left, None);
    }

    #[test]
    fn individual_padding_methods() {
        let s = Style::default().pt(1.0).pr(2.0).pb(3.0).pl(4.0);
        assert_eq!(s.padding.top, Some(4.0));
        assert_eq!(s.padding.right, Some(8.0));
        assert_eq!(s.padding.bottom, Some(12.0));
        assert_eq!(s.padding.left, Some(16.0));
    }

    #[test]
    fn m_sets_all_margin() {
        let s = Style::default().m(2.0);
        assert_eq!(s.margin, Edges::all(8.0));
    }

    #[test]
    fn mx_my_set_axes() {
        let s = Style::default().mx(4.0).my(2.0);
        assert_eq!(s.margin.left, Some(16.0));
        assert_eq!(s.margin.right, Some(16.0));
        assert_eq!(s.margin.top, Some(8.0));
        assert_eq!(s.margin.bottom, Some(8.0));
    }

    // -- Sizing --

    #[test]
    fn w_h_set_dimensions() {
        let s = Style::default().w(100.0).h(200.0);
        assert_eq!(s.width, Some(100.0));
        assert_eq!(s.height, Some(200.0));
    }

    #[test]
    fn size_sets_both() {
        let s = Style::default().size(50.0);
        assert_eq!(s.width, Some(50.0));
        assert_eq!(s.height, Some(50.0));
    }

    #[test]
    fn min_max_dimensions() {
        let s = Style::default()
            .min_w(10.0)
            .min_h(20.0)
            .max_w(300.0)
            .max_h(400.0);
        assert_eq!(s.min_width, Some(10.0));
        assert_eq!(s.min_height, Some(20.0));
        assert_eq!(s.max_width, Some(300.0));
        assert_eq!(s.max_height, Some(400.0));
    }

    // -- Flex layout --

    #[test]
    fn flex_direction_methods() {
        assert_eq!(
            Style::default().flex_row().flex_direction,
            Some(FlexDirection::Row)
        );
        assert_eq!(
            Style::default().flex_col().flex_direction,
            Some(FlexDirection::Column)
        );
        assert_eq!(
            Style::default().flex_row_reverse().flex_direction,
            Some(FlexDirection::RowReverse)
        );
        assert_eq!(
            Style::default().flex_col_reverse().flex_direction,
            Some(FlexDirection::ColumnReverse)
        );
    }

    #[test]
    fn flex_wrap_methods() {
        assert_eq!(Style::default().flex_wrap().flex_wrap, Some(FlexWrap::Wrap));
        assert_eq!(
            Style::default().flex_nowrap().flex_wrap,
            Some(FlexWrap::NoWrap)
        );
        assert_eq!(
            Style::default().flex_wrap_reverse().flex_wrap,
            Some(FlexWrap::WrapReverse)
        );
    }

    #[test]
    fn align_items_methods() {
        assert_eq!(
            Style::default().items_center().align_items,
            Some(AlignItems::Center)
        );
        assert_eq!(
            Style::default().items_start().align_items,
            Some(AlignItems::FlexStart)
        );
        assert_eq!(
            Style::default().items_stretch().align_items,
            Some(AlignItems::Stretch)
        );
    }

    #[test]
    fn justify_content_methods() {
        assert_eq!(
            Style::default().justify_center().justify_content,
            Some(JustifyContent::Center)
        );
        assert_eq!(
            Style::default().justify_between().justify_content,
            Some(JustifyContent::SpaceBetween)
        );
        assert_eq!(
            Style::default().justify_evenly().justify_content,
            Some(JustifyContent::SpaceEvenly)
        );
    }

    #[test]
    fn align_self_methods() {
        assert_eq!(
            Style::default().self_center().align_self,
            Some(AlignSelf::Center)
        );
        assert_eq!(
            Style::default().self_auto().align_self,
            Some(AlignSelf::Auto)
        );
    }

    #[test]
    fn gap_uses_spacing_scale() {
        let s = Style::default().gap(4.0);
        assert_eq!(s.gap, Some(16.0));
    }

    #[test]
    fn row_col_gap() {
        let s = Style::default().row_gap(2.0).col_gap(3.0);
        assert_eq!(s.row_gap, Some(8.0));
        assert_eq!(s.column_gap, Some(12.0));
    }

    #[test]
    fn grow_shrink_basis() {
        let s = Style::default().grow(1.0).shrink(0.0).basis(200.0);
        assert_eq!(s.flex_grow, Some(1.0));
        assert_eq!(s.flex_shrink, Some(0.0));
        assert_eq!(s.flex_basis, Some(200.0));
    }

    // -- Color --

    #[test]
    fn bg_and_text_color() {
        let s = Style::default().bg(Color::WHITE).text_color(Color::BLACK);
        assert_eq!(s.background, Some(Color::WHITE));
        assert_eq!(s.foreground, Some(Color::BLACK));
    }

    #[test]
    fn convenience_colors() {
        assert_eq!(Style::default().bg_white().background, Some(Color::WHITE));
        assert_eq!(Style::default().bg_black().background, Some(Color::BLACK));
        assert_eq!(
            Style::default().bg_transparent().background,
            Some(Color::TRANSPARENT)
        );
        assert_eq!(Style::default().text_white().foreground, Some(Color::WHITE));
        assert_eq!(Style::default().text_black().foreground, Some(Color::BLACK));
    }

    #[test]
    fn bg_blue_500_matches_palette() {
        let s = Style::default().bg_blue(500);
        assert_eq!(s.background, Palette::BLUE.get(500));
    }

    #[test]
    fn text_red_700_matches_palette() {
        let s = Style::default().text_red(700);
        assert_eq!(s.foreground, Palette::RED.get(700));
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn invalid_palette_stop_is_noop() {
        let s = Style::default().bg_blue(42);
        assert_eq!(s.background, None);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "invalid palette stop")]
    fn invalid_palette_stop_panics_in_debug() {
        let _ = Style::default().bg_blue(42);
    }

    #[test]
    fn all_22_hue_bg_methods_exist() {
        // Compile-time proof that all 22 bg methods exist and work
        let _ = Style::default().bg_slate(500);
        let _ = Style::default().bg_gray(500);
        let _ = Style::default().bg_zinc(500);
        let _ = Style::default().bg_neutral(500);
        let _ = Style::default().bg_stone(500);
        let _ = Style::default().bg_red(500);
        let _ = Style::default().bg_orange(500);
        let _ = Style::default().bg_amber(500);
        let _ = Style::default().bg_yellow(500);
        let _ = Style::default().bg_lime(500);
        let _ = Style::default().bg_green(500);
        let _ = Style::default().bg_emerald(500);
        let _ = Style::default().bg_teal(500);
        let _ = Style::default().bg_cyan(500);
        let _ = Style::default().bg_sky(500);
        let _ = Style::default().bg_blue(500);
        let _ = Style::default().bg_indigo(500);
        let _ = Style::default().bg_violet(500);
        let _ = Style::default().bg_purple(500);
        let _ = Style::default().bg_fuchsia(500);
        let _ = Style::default().bg_pink(500);
        let _ = Style::default().bg_rose(500);
    }

    // -- Border --

    #[test]
    fn border_sets_width_and_color() {
        let s = Style::default().border(1.0, Color::BLACK);
        assert_eq!(s.border_width, Edges::all(1.0));
        assert_eq!(s.border_color, Some(Color::BLACK));
    }

    #[test]
    fn border_sides() {
        let s = Style::default()
            .border_t(1.0)
            .border_r(2.0)
            .border_b(3.0)
            .border_l(4.0);
        assert_eq!(s.border_width.top, Some(1.0));
        assert_eq!(s.border_width.right, Some(2.0));
        assert_eq!(s.border_width.bottom, Some(3.0));
        assert_eq!(s.border_width.left, Some(4.0));
    }

    // -- Radius --

    #[test]
    fn rounded_md_equals_6px() {
        let s = Style::default().rounded_md();
        assert_eq!(s.border_radius, Corners::all(6.0));
    }

    #[test]
    fn rounded_full_equals_9999px() {
        let s = Style::default().rounded_full();
        assert_eq!(s.border_radius, Corners::all(9999.0));
    }

    #[test]
    fn rounded_none_equals_0px() {
        let s = Style::default().rounded_none();
        assert_eq!(s.border_radius, Corners::all(0.0));
    }

    // -- Shadow --

    #[test]
    fn shadow_lg_matches_token() {
        let s = Style::default().shadow_lg();
        assert_eq!(s.shadow.as_ref().map(Vec::len), Some(2));
    }

    #[test]
    fn shadow_none_is_empty_vec() {
        let s = Style::default().shadow_none();
        assert_eq!(s.shadow, Some(vec![]));
    }

    #[test]
    fn shadow_inner_is_inset() {
        let s = Style::default().shadow_inner();
        assert!(s
            .shadow
            .as_ref()
            .map_or(false, |v| v.first().map_or(false, |s| s.inset)));
    }

    // -- Typography --

    #[test]
    fn font_bold_sets_weight() {
        let s = Style::default().font_bold();
        assert_eq!(s.font.weight, Some(FontWeight::BOLD));
    }

    #[test]
    fn font_size_sets_px() {
        let s = Style::default().font_size(16.0);
        assert_eq!(s.font.size, Some(16.0));
    }

    #[test]
    fn font_family_from_str() {
        let s = Style::default().font_family("Inter");
        assert_eq!(s.font.family, Some("Inter".into()));
    }

    #[test]
    fn italic_sets_slant() {
        let s = Style::default().italic();
        assert_eq!(s.font.slant, Some(FontSlant::Italic));
    }

    #[test]
    fn leading_and_tracking() {
        let s = Style::default().leading(1.5).tracking(0.5);
        assert_eq!(s.font.line_height, Some(1.5));
        assert_eq!(s.font.letter_spacing, Some(0.5));
    }

    // -- Overflow + Opacity --

    #[test]
    fn opacity_sets_value() {
        let s = Style::default().opacity(0.5);
        assert_eq!(s.opacity, Some(0.5));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "out of range")]
    fn opacity_rejects_negative() {
        let _ = Style::default().opacity(-0.5);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "out of range")]
    fn opacity_rejects_above_one() {
        let _ = Style::default().opacity(1.5);
    }

    #[test]
    fn overflow_methods() {
        assert_eq!(
            Style::default().overflow_hidden().overflow,
            Some(Overflow::Hidden)
        );
        assert_eq!(
            Style::default().overflow_scroll().overflow,
            Some(Overflow::Scroll)
        );
        assert_eq!(
            Style::default().overflow_auto().overflow,
            Some(Overflow::Auto)
        );
        assert_eq!(
            Style::default().overflow_visible().overflow,
            Some(Overflow::Visible)
        );
    }

    // -- State modifiers --

    #[test]
    fn hover_stores_style() {
        let s = Style::default().hover(|s| s.bg_blue(600).shadow_md());
        assert!(s.hover.is_some());
        let h = s.hover.as_ref().unwrap();
        assert_eq!(h.background, Palette::BLUE.get(600));
        assert!(h.shadow.is_some());
    }

    #[test]
    fn focus_stores_style() {
        let s = Style::default().focus(|s| s.border_color(Color::hex(0x3b82f6)));
        assert!(s.focus.is_some());
        assert_eq!(
            s.focus.as_ref().unwrap().border_color,
            Some(Color::hex(0x3b82f6))
        );
    }

    #[test]
    fn active_stores_style() {
        let s = Style::default().active(|s| s.opacity(0.8));
        assert!(s.active.is_some());
        assert_eq!(s.active.as_ref().unwrap().opacity, Some(0.8));
    }

    #[test]
    fn disabled_stores_style() {
        let s = Style::default().disabled(|s| s.opacity(0.5));
        assert!(s.disabled.is_some());
        assert_eq!(s.disabled.as_ref().unwrap().opacity, Some(0.5));
    }

    // -- Conditional --

    #[test]
    fn when_true_applies() {
        let s = Style::default().when(true, |s| s.bg_blue(500));
        assert_eq!(s.background, Palette::BLUE.get(500));
    }

    #[test]
    fn when_false_is_noop() {
        let s = Style::default().when(false, |s| s.bg_blue(500));
        assert_eq!(s.background, None);
    }

    #[test]
    fn apply_always_applies() {
        let s = Style::default().apply(|s| s.p(4.0).bg_white());
        assert_eq!(s.padding, Edges::all(16.0));
        assert_eq!(s.background, Some(Color::WHITE));
    }

    // -- Gradient --

    #[test]
    fn bg_gradient_to_r_sets_direction() {
        let stops = vec![
            ColorStop {
                color: Color::WHITE,
                position: 0.0,
            },
            ColorStop {
                color: Color::BLACK,
                position: 1.0,
            },
        ];
        let s = Style::default().bg_gradient_to_r(stops);
        let g = s.background_gradient.as_ref().unwrap();
        assert_eq!(g.direction, GradientDirection::ToRight);
        assert_eq!(g.stops.len(), 2);
    }

    #[test]
    fn bg_gradient_to_l_sets_direction() {
        let stops = vec![ColorStop {
            color: Color::WHITE,
            position: 0.0,
        }];
        let s = Style::default().bg_gradient_to_l(stops);
        assert_eq!(
            s.background_gradient.as_ref().unwrap().direction,
            GradientDirection::ToLeft
        );
    }

    #[test]
    fn bg_gradient_to_t_sets_direction() {
        let stops = vec![ColorStop {
            color: Color::WHITE,
            position: 0.0,
        }];
        let s = Style::default().bg_gradient_to_t(stops);
        assert_eq!(
            s.background_gradient.as_ref().unwrap().direction,
            GradientDirection::ToTop
        );
    }

    #[test]
    fn bg_gradient_to_b_sets_direction() {
        let stops = vec![ColorStop {
            color: Color::WHITE,
            position: 0.0,
        }];
        let s = Style::default().bg_gradient_to_b(stops);
        assert_eq!(
            s.background_gradient.as_ref().unwrap().direction,
            GradientDirection::ToBottom
        );
    }

    #[test]
    fn bg_gradient_angle_sets_degrees() {
        let stops = vec![ColorStop {
            color: Color::WHITE,
            position: 0.0,
        }];
        let s = Style::default().bg_gradient_angle(45.0, stops);
        assert_eq!(
            s.background_gradient.as_ref().unwrap().direction,
            GradientDirection::Angle(45.0)
        );
    }

    #[test]
    fn bg_gradient_sets_full_gradient() {
        let gradient = LinearGradient {
            direction: GradientDirection::Angle(135.0),
            stops: vec![
                ColorStop {
                    color: Color::rgb(1.0, 0.0, 0.0),
                    position: 0.0,
                },
                ColorStop {
                    color: Color::rgb(0.0, 0.0, 1.0),
                    position: 1.0,
                },
            ],
        };
        let s = Style::default().bg_gradient(gradient.clone());
        assert_eq!(s.background_gradient, Some(gradient));
    }

    // -- Chaining --

    #[test]
    fn full_builder_chain() {
        let s = Style::default()
            .flex_col()
            .p(4.0)
            .gap(2.0)
            .bg_white()
            .rounded_lg()
            .shadow_md()
            .font_size(14.0)
            .font_bold()
            .text_slate(900);

        assert_eq!(s.flex_direction, Some(FlexDirection::Column));
        assert_eq!(s.padding, Edges::all(16.0));
        assert_eq!(s.gap, Some(8.0));
        assert_eq!(s.background, Some(Color::WHITE));
        assert_eq!(s.border_radius, Corners::all(8.0));
        assert!(s.shadow.is_some());
        assert_eq!(s.font.size, Some(14.0));
        assert_eq!(s.font.weight, Some(FontWeight::BOLD));
        assert_eq!(s.foreground, Palette::SLATE.get(900));
    }
}
