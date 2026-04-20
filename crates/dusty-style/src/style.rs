//! Core `Style` struct and layout enums.

use crate::{BoxShadow, Color, Corners, Edges, FontStyle, LinearGradient};

/// Flex container direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlexDirection {
    /// Items laid out left-to-right (default).
    #[default]
    Row,
    /// Items laid out right-to-left.
    RowReverse,
    /// Items laid out top-to-bottom.
    Column,
    /// Items laid out bottom-to-top.
    ColumnReverse,
}

/// Flex line wrapping behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlexWrap {
    /// Single line, no wrapping (default).
    #[default]
    NoWrap,
    /// Wrap onto multiple lines.
    Wrap,
    /// Wrap onto multiple lines in reverse.
    WrapReverse,
}

/// Cross-axis alignment for flex items within a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlignItems {
    /// Align to the start of the cross axis.
    FlexStart,
    /// Align to the end of the cross axis.
    FlexEnd,
    /// Center along the cross axis.
    Center,
    /// Align to the text baseline.
    Baseline,
    /// Stretch to fill the container (default).
    #[default]
    Stretch,
}

/// Cross-axis alignment override for a single flex item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlignSelf {
    /// Inherit from parent's `align_items` (default).
    #[default]
    Auto,
    /// Align to the start of the cross axis.
    FlexStart,
    /// Align to the end of the cross axis.
    FlexEnd,
    /// Center along the cross axis.
    Center,
    /// Align to the text baseline.
    Baseline,
    /// Stretch to fill the container.
    Stretch,
}

/// Main-axis distribution of flex items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum JustifyContent {
    /// Pack items toward the start (default).
    #[default]
    FlexStart,
    /// Pack items toward the end.
    FlexEnd,
    /// Center items along the main axis.
    Center,
    /// Distribute with equal space between items.
    SpaceBetween,
    /// Distribute with equal space around items.
    SpaceAround,
    /// Distribute with equal space between and around items.
    SpaceEvenly,
}

/// A dimensional value: pixels, a percentage of the parent, or `auto`.
///
/// `Percent` is a fraction in `0.0..=1.0` (so `Percent(0.5)` means 50%).
/// `Auto` defers sizing to the layout engine — on margin it enables
/// flex-container centering; on size fields it behaves like "unset".
///
/// # Examples
///
/// ```
/// use dusty_style::Length;
///
/// let fixed = Length::Px(10.0);
/// let half = Length::Percent(0.5);
/// let auto = Length::Auto;
/// assert_ne!(fixed, auto);
/// assert_ne!(fixed, half);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    /// A fixed length in pixels.
    Px(f32),
    /// A fraction of the parent's corresponding dimension (0.0–1.0).
    Percent(f32),
    /// Auto — defer to the layout engine.
    Auto,
}

impl Default for Length {
    fn default() -> Self {
        Self::Px(0.0)
    }
}

/// A dimensional value without `Auto`. Used for fields where `Auto` has no
/// layout meaning (padding, gap, border).
///
/// # Examples
///
/// ```
/// use dusty_style::LengthPercent;
///
/// let fixed = LengthPercent::Px(8.0);
/// let half = LengthPercent::Percent(0.5);
/// assert_ne!(fixed, half);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthPercent {
    /// A fixed length in pixels.
    Px(f32),
    /// A fraction of the parent's corresponding dimension (0.0–1.0).
    Percent(f32),
}

impl Default for LengthPercent {
    fn default() -> Self {
        Self::Px(0.0)
    }
}

impl LengthPercent {
    /// Returns the pixel value if this is `Px`, otherwise `None`.
    /// Percent values need a resolved base to convert; callers that have one
    /// should inspect the variant directly.
    #[must_use]
    pub const fn as_px(self) -> Option<f32> {
        match self {
            Self::Px(v) => Some(v),
            Self::Percent(_) => None,
        }
    }
}

impl Length {
    /// Returns the pixel value if this is `Px`, otherwise `None`.
    #[must_use]
    pub const fn as_px(self) -> Option<f32> {
        match self {
            Self::Px(v) => Some(v),
            Self::Percent(_) | Self::Auto => None,
        }
    }
}

/// Overflow behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Overflow {
    /// Content is not clipped (default).
    #[default]
    Visible,
    /// Content is clipped without scrollbars.
    Hidden,
    /// Content is clipped with scrollbars.
    Scroll,
    /// Scrollbars appear only when needed.
    Auto,
}

/// Display mode for an element.
///
/// Controls how the element participates in layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Display {
    /// Flexbox layout (default).
    #[default]
    Flex,
    /// Block layout.
    Block,
    /// Element is removed from layout entirely.
    None,
}

/// Positioning scheme for an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Position {
    /// Positioned relative to its normal layout position (default).
    #[default]
    Relative,
    /// Positioned relative to its nearest positioned ancestor.
    Absolute,
}

/// Current interaction state for resolving state-based style overrides.
///
/// # Examples
///
/// ```
/// use dusty_style::InteractionState;
///
/// let hovered = InteractionState { hovered: true, ..InteractionState::default() };
/// assert!(hovered.hovered);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct InteractionState {
    /// Whether the element is hovered.
    pub hovered: bool,
    /// Whether the element has keyboard focus.
    pub focused: bool,
    /// Whether the element is being pressed.
    pub active: bool,
    /// Whether the element is disabled.
    pub disabled: bool,
}

/// Complete style specification for an element. All `Option` fields default to
/// `None` (not set), enabling cascade/merge semantics.
///
/// # Examples
///
/// ```
/// use dusty_style::{Style, Color, Edges, FlexDirection, LengthPercent};
///
/// let card = Style {
///     padding: Edges::all(LengthPercent::Px(16.0)),
///     background: Some(Color::WHITE),
///     flex_direction: Some(FlexDirection::Column),
///     ..Style::default()
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Style {
    // -- Box model --
    /// Padding (inner spacing). Supports pixels and percentages.
    pub padding: Edges<LengthPercent>,
    /// Margin (outer spacing). Supports pixels, percentages, and `Auto` for centering.
    pub margin: Edges<Length>,
    /// Explicit width.
    pub width: Option<Length>,
    /// Explicit height.
    pub height: Option<Length>,
    /// Minimum width.
    pub min_width: Option<Length>,
    /// Minimum height.
    pub min_height: Option<Length>,
    /// Maximum width.
    pub max_width: Option<Length>,
    /// Maximum height.
    pub max_height: Option<Length>,
    /// Aspect ratio (`width / height`). Respected when exactly one of
    /// `width` / `height` is constrained.
    pub aspect_ratio: Option<f32>,

    // -- Layout mode --
    /// Display mode.
    pub display: Option<Display>,
    /// Positioning scheme.
    pub position: Option<Position>,
    /// Inset values for positioned elements (top, right, bottom, left).
    pub inset: Edges<Length>,

    // -- Flex layout --
    /// Flex container direction.
    pub flex_direction: Option<FlexDirection>,
    /// Flex line wrapping.
    pub flex_wrap: Option<FlexWrap>,
    /// Flex grow factor.
    pub flex_grow: Option<f32>,
    /// Flex shrink factor.
    pub flex_shrink: Option<f32>,
    /// Flex basis.
    pub flex_basis: Option<Length>,
    /// Cross-axis alignment for children.
    pub align_items: Option<AlignItems>,
    /// Cross-axis alignment override for this item.
    pub align_self: Option<AlignSelf>,
    /// Main-axis distribution.
    pub justify_content: Option<JustifyContent>,
    /// Gap between items (both axes).
    pub gap: Option<LengthPercent>,
    /// Gap between rows.
    pub row_gap: Option<LengthPercent>,
    /// Gap between columns.
    pub column_gap: Option<LengthPercent>,

    // -- Visual --
    /// Background color.
    pub background: Option<Color>,
    /// Background gradient (takes precedence over solid background when set).
    pub background_gradient: Option<LinearGradient>,
    /// Foreground (text) color.
    pub foreground: Option<Color>,
    /// Border color.
    pub border_color: Option<Color>,
    /// Border width per side.
    pub border_width: Edges<f32>,
    /// Border radius per corner.
    pub border_radius: Corners<f32>,
    /// Opacity (0.0–1.0).
    pub opacity: Option<f32>,

    // -- Shadows --
    /// Box shadows. `None` = not set; `Some(vec![])` = explicitly no shadows.
    pub shadow: Option<Vec<BoxShadow>>,

    // -- Typography --
    /// Font/typography properties.
    pub font: FontStyle,

    // -- Overflow --
    /// Overflow behavior.
    pub overflow: Option<Overflow>,

    // -- State overrides --
    /// Style overrides applied on hover.
    pub hover: Option<Box<Self>>,
    /// Style overrides applied on focus.
    pub focus: Option<Box<Self>>,
    /// Style overrides applied on active (pressed).
    pub active: Option<Box<Self>>,
    /// Style overrides applied when disabled.
    pub disabled: Option<Box<Self>>,
}

fn merge_state_style(base: Option<&Style>, other: Option<&Style>) -> Option<Box<Style>> {
    match (base, other) {
        (Some(b), Some(o)) => Some(Box::new(b.merge(o))),
        (None, Some(o)) => Some(Box::new(o.clone())),
        (Some(b), None) => Some(Box::new(b.clone())),
        (None, None) => None,
    }
}

impl Style {
    /// Merges `other` on top of `self`. Other's `Some` values win.
    /// `Edges`, `Corners`, and `FontStyle` merge per-field.
    /// `shadow` is replaced entirely (not element-wise merged).
    /// State overrides (`hover`, `focus`, `active`, `disabled`) merge per-field recursively.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            // Box model
            padding: self.padding.merge(&other.padding),
            margin: self.margin.merge(&other.margin),
            width: other.width.or(self.width),
            height: other.height.or(self.height),
            min_width: other.min_width.or(self.min_width),
            min_height: other.min_height.or(self.min_height),
            max_width: other.max_width.or(self.max_width),
            max_height: other.max_height.or(self.max_height),
            aspect_ratio: other.aspect_ratio.or(self.aspect_ratio),

            // Layout mode
            display: other.display.or(self.display),
            position: other.position.or(self.position),
            inset: self.inset.merge(&other.inset),

            // Flex layout
            flex_direction: other.flex_direction.or(self.flex_direction),
            flex_wrap: other.flex_wrap.or(self.flex_wrap),
            flex_grow: other.flex_grow.or(self.flex_grow),
            flex_shrink: other.flex_shrink.or(self.flex_shrink),
            flex_basis: other.flex_basis.or(self.flex_basis),
            align_items: other.align_items.or(self.align_items),
            align_self: other.align_self.or(self.align_self),
            justify_content: other.justify_content.or(self.justify_content),
            gap: other.gap.or(self.gap),
            row_gap: other.row_gap.or(self.row_gap),
            column_gap: other.column_gap.or(self.column_gap),

            // Visual
            background: other.background.or(self.background),
            background_gradient: other
                .background_gradient
                .clone()
                .or_else(|| self.background_gradient.clone()),
            foreground: other.foreground.or(self.foreground),
            border_color: other.border_color.or(self.border_color),
            border_width: self.border_width.merge(&other.border_width),
            border_radius: self.border_radius.merge(&other.border_radius),
            opacity: other.opacity.or(self.opacity),

            // Shadows — replaced entirely
            shadow: other.shadow.clone().or_else(|| self.shadow.clone()),

            // Typography
            font: self.font.merge(&other.font),

            // Overflow
            overflow: other.overflow.or(self.overflow),

            // State overrides — per-field recursive merge
            hover: merge_state_style(self.hover.as_deref(), other.hover.as_deref()),
            focus: merge_state_style(self.focus.as_deref(), other.focus.as_deref()),
            active: merge_state_style(self.active.as_deref(), other.active.as_deref()),
            disabled: merge_state_style(self.disabled.as_deref(), other.disabled.as_deref()),
        }
    }

    /// Flattens state overrides for a given interaction state, producing a
    /// final `Style` with no remaining state fields.
    ///
    /// Merge order: hover → focus → active → disabled (later wins on conflict).
    #[must_use]
    pub fn resolve(&self, state: &InteractionState) -> Self {
        let mut result = self.clone();
        result.hover = None;
        result.focus = None;
        result.active = None;
        result.disabled = None;

        if state.hovered {
            if let Some(h) = &self.hover {
                result = result.merge(h);
            }
        }
        if state.focused {
            if let Some(f) = &self.focus {
                result = result.merge(f);
            }
        }
        if state.active {
            if let Some(a) = &self.active {
                result = result.merge(a);
            }
        }
        if state.disabled {
            if let Some(d) = &self.disabled {
                result = result.merge(d);
            }
        }

        // Clear state fields again after merges — state overrides may have
        // contained nested state styles that merge() carried through.
        result.hover = None;
        result.focus = None;
        result.active = None;
        result.disabled = None;

        result
    }

    /// Returns the effective row gap in pixels. Percent gaps resolve to `0.0`
    /// here because this helper has no parent reference; layout resolves
    /// percent gaps through the taffy conversion path.
    #[must_use]
    pub fn resolved_row_gap(&self) -> f32 {
        match self.row_gap.or(self.gap) {
            Some(LengthPercent::Px(v)) => v,
            Some(LengthPercent::Percent(_)) | None => 0.0,
        }
    }

    /// Returns the effective column gap in pixels. Percent gaps resolve to
    /// `0.0` here — see [`Self::resolved_row_gap`].
    #[must_use]
    pub fn resolved_column_gap(&self) -> f32 {
        match self.column_gap.or(self.gap) {
            Some(LengthPercent::Px(v)) => v,
            Some(LengthPercent::Percent(_)) | None => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FontWeight;

    #[test]
    fn style_default_is_empty() {
        let s = Style::default();
        assert_eq!(s.padding, Edges::default());
        assert_eq!(s.margin, Edges::default());
        assert_eq!(s.width, None);
        assert_eq!(s.height, None);
        assert_eq!(s.background, None);
        assert_eq!(s.foreground, None);
        assert_eq!(s.flex_direction, None);
        assert_eq!(s.opacity, None);
        assert_eq!(s.shadow, None);
        assert_eq!(s.overflow, None);
        assert_eq!(s.font, FontStyle::default());
    }

    #[test]
    fn merge_option_fields_other_wins() {
        let base = Style {
            width: Some(Length::Px(100.0)),
            height: Some(Length::Px(200.0)),
            background: Some(Color::WHITE),
            ..Style::default()
        };
        let over = Style {
            width: Some(Length::Px(300.0)),
            background: Some(Color::BLACK),
            ..Style::default()
        };
        let merged = base.merge(&over);
        assert_eq!(merged.width, Some(Length::Px(300.0)));
        assert_eq!(merged.height, Some(Length::Px(200.0)));
        assert_eq!(merged.background, Some(Color::BLACK));
    }

    #[test]
    fn merge_preserves_unset_fields() {
        let base = Style {
            flex_grow: Some(1.0),
            opacity: Some(0.5),
            ..Style::default()
        };
        let over = Style::default();
        let merged = base.merge(&over);
        assert_eq!(merged.flex_grow, Some(1.0));
        assert_eq!(merged.opacity, Some(0.5));
    }

    #[test]
    fn merge_edges_per_field() {
        let base = Style {
            padding: Edges::all(LengthPercent::Px(8.0)),
            ..Style::default()
        };
        let over = Style {
            padding: Edges {
                top: Some(LengthPercent::Px(16.0)),
                ..Edges::default()
            },
            ..Style::default()
        };
        let merged = base.merge(&over);
        assert_eq!(merged.padding.top, Some(LengthPercent::Px(16.0)));
        assert_eq!(merged.padding.right, Some(LengthPercent::Px(8.0)));
        assert_eq!(merged.padding.bottom, Some(LengthPercent::Px(8.0)));
        assert_eq!(merged.padding.left, Some(LengthPercent::Px(8.0)));
    }

    #[test]
    fn merge_corners_per_field() {
        let base = Style {
            border_radius: Corners::all(4.0),
            ..Style::default()
        };
        let over = Style {
            border_radius: Corners {
                top_left: Some(12.0),
                ..Corners::default()
            },
            ..Style::default()
        };
        let merged = base.merge(&over);
        assert_eq!(merged.border_radius.top_left, Some(12.0));
        assert_eq!(merged.border_radius.top_right, Some(4.0));
    }

    #[test]
    fn merge_shadow_replaced_entirely() {
        let shadow_a = BoxShadow {
            offset_x: 0.0,
            offset_y: 2.0,
            blur_radius: 4.0,
            spread_radius: 0.0,
            color: Color::BLACK,
            inset: false,
        };
        let shadow_b = BoxShadow {
            offset_x: 0.0,
            offset_y: 10.0,
            blur_radius: 20.0,
            spread_radius: 0.0,
            color: Color::BLACK,
            inset: false,
        };

        let base = Style {
            shadow: Some(vec![shadow_a]),
            ..Style::default()
        };
        let over = Style {
            shadow: Some(vec![shadow_b]),
            ..Style::default()
        };
        let merged = base.merge(&over);
        assert_eq!(merged.shadow.as_ref().map(Vec::len), Some(1));
        assert_eq!(merged.shadow.as_ref().map(|v| v[0].offset_y), Some(10.0));
    }

    #[test]
    fn merge_shadow_none_preserves_base() {
        let shadow = BoxShadow {
            offset_x: 0.0,
            offset_y: 2.0,
            blur_radius: 4.0,
            spread_radius: 0.0,
            color: Color::BLACK,
            inset: false,
        };
        let base = Style {
            shadow: Some(vec![shadow]),
            ..Style::default()
        };
        let over = Style::default();
        let merged = base.merge(&over);
        assert!(merged.shadow.is_some());
    }

    #[test]
    fn merge_font_per_field() {
        let base = Style {
            font: FontStyle {
                size: Some(16.0),
                weight: Some(FontWeight::NORMAL),
                ..FontStyle::default()
            },
            ..Style::default()
        };
        let over = Style {
            font: FontStyle {
                weight: Some(FontWeight::BOLD),
                ..FontStyle::default()
            },
            ..Style::default()
        };
        let merged = base.merge(&over);
        assert_eq!(merged.font.size, Some(16.0));
        assert_eq!(merged.font.weight, Some(FontWeight::BOLD));
    }

    #[test]
    fn three_style_cascade_chain() {
        let base = Style {
            padding: Edges::all(LengthPercent::Px(8.0)),
            background: Some(Color::WHITE),
            font: FontStyle {
                size: Some(14.0),
                ..FontStyle::default()
            },
            ..Style::default()
        };
        let component = Style {
            padding: Edges {
                top: Some(LengthPercent::Px(16.0)),
                bottom: Some(LengthPercent::Px(16.0)),
                ..Edges::default()
            },
            background: Some(Color::hex(0xF0F0F0)),
            font: FontStyle {
                weight: Some(FontWeight::BOLD),
                ..FontStyle::default()
            },
            ..Style::default()
        };
        let hover = Style {
            background: Some(Color::hex(0xE0E0E0)),
            opacity: Some(0.9),
            ..Style::default()
        };

        let result = base.merge(&component).merge(&hover);

        // Padding: top/bottom from component, left/right from base
        assert_eq!(result.padding.top, Some(LengthPercent::Px(16.0)));
        assert_eq!(result.padding.left, Some(LengthPercent::Px(8.0)));
        // Background: hover wins
        assert_eq!(result.background, Some(Color::hex(0xE0E0E0)));
        // Font: size from base, weight from component
        assert_eq!(result.font.size, Some(14.0));
        assert_eq!(result.font.weight, Some(FontWeight::BOLD));
        // Opacity: only hover set it
        assert_eq!(result.opacity, Some(0.9));
    }

    #[test]
    fn style_is_dyn_any_compatible() {
        use std::any::Any;

        let style = Style {
            width: Some(Length::Px(100.0)),
            ..Style::default()
        };
        let boxed: Box<dyn Any> = Box::new(style);
        let downcast = boxed.downcast_ref::<Style>();
        assert!(downcast.is_some());
        assert_eq!(downcast.map(|s| s.width), Some(Some(Length::Px(100.0))));
    }

    #[test]
    fn state_fields_default_to_none() {
        let s = Style::default();
        assert!(s.hover.is_none());
        assert!(s.focus.is_none());
        assert!(s.active.is_none());
        assert!(s.disabled.is_none());
    }

    #[test]
    fn merge_state_styles_per_field() {
        let base = Style {
            hover: Some(Box::new(Style {
                background: Some(Color::WHITE),
                ..Style::default()
            })),
            ..Style::default()
        };
        let over = Style {
            hover: Some(Box::new(Style {
                opacity: Some(0.8),
                ..Style::default()
            })),
            ..Style::default()
        };
        let merged = base.merge(&over);
        let hover = merged.hover.as_ref().unwrap();
        assert_eq!(hover.background, Some(Color::WHITE));
        assert_eq!(hover.opacity, Some(0.8));
    }

    #[test]
    fn merge_state_none_preserves_base() {
        let base = Style {
            focus: Some(Box::new(Style {
                border_color: Some(Color::hex(0x3b82f6)),
                ..Style::default()
            })),
            ..Style::default()
        };
        let over = Style::default();
        let merged = base.merge(&over);
        assert!(merged.focus.is_some());
    }

    #[test]
    fn resolve_applies_hover() {
        let style = Style {
            background: Some(Color::WHITE),
            hover: Some(Box::new(Style {
                background: Some(Color::BLACK),
                ..Style::default()
            })),
            ..Style::default()
        };
        let normal = style.resolve(&InteractionState::default());
        assert_eq!(normal.background, Some(Color::WHITE));
        assert!(normal.hover.is_none());

        let hovered = style.resolve(&InteractionState {
            hovered: true,
            ..InteractionState::default()
        });
        assert_eq!(hovered.background, Some(Color::BLACK));
    }

    #[test]
    fn resolve_disabled_overrides_hover() {
        let style = Style {
            hover: Some(Box::new(Style {
                opacity: Some(0.9),
                ..Style::default()
            })),
            disabled: Some(Box::new(Style {
                opacity: Some(0.5),
                ..Style::default()
            })),
            ..Style::default()
        };
        let resolved = style.resolve(&InteractionState {
            hovered: true,
            disabled: true,
            ..InteractionState::default()
        });
        assert_eq!(resolved.opacity, Some(0.5));
    }

    #[test]
    fn resolve_clears_state_fields() {
        let style = Style {
            hover: Some(Box::new(Style::default())),
            focus: Some(Box::new(Style::default())),
            active: Some(Box::new(Style::default())),
            disabled: Some(Box::new(Style::default())),
            ..Style::default()
        };
        let resolved = style.resolve(&InteractionState {
            hovered: true,
            focused: true,
            active: true,
            disabled: true,
        });
        assert!(resolved.hover.is_none());
        assert!(resolved.focus.is_none());
        assert!(resolved.active.is_none());
        assert!(resolved.disabled.is_none());
    }

    #[test]
    fn resolve_clears_nested_state_from_overrides() {
        let style = Style {
            hover: Some(Box::new(Style {
                background: Some(Color::BLACK),
                hover: Some(Box::new(Style {
                    opacity: Some(0.5),
                    ..Style::default()
                })),
                ..Style::default()
            })),
            ..Style::default()
        };
        let resolved = style.resolve(&InteractionState {
            hovered: true,
            ..InteractionState::default()
        });
        assert_eq!(resolved.background, Some(Color::BLACK));
        // Nested state fields must be cleared after resolve
        assert!(resolved.hover.is_none());
        assert!(resolved.focus.is_none());
        assert!(resolved.active.is_none());
        assert!(resolved.disabled.is_none());
    }

    #[test]
    fn resolved_row_gap_prefers_row_gap_over_gap() {
        let s = Style {
            gap: Some(LengthPercent::Px(10.0)),
            row_gap: Some(LengthPercent::Px(20.0)),
            ..Style::default()
        };
        assert_eq!(s.resolved_row_gap(), 20.0);
    }

    #[test]
    fn resolved_row_gap_falls_back_to_gap() {
        let s = Style {
            gap: Some(LengthPercent::Px(10.0)),
            ..Style::default()
        };
        assert_eq!(s.resolved_row_gap(), 10.0);
    }

    #[test]
    fn resolved_row_gap_defaults_to_zero() {
        let s = Style::default();
        assert_eq!(s.resolved_row_gap(), 0.0);
    }

    #[test]
    fn resolved_row_gap_percent_reports_zero() {
        let s = Style {
            gap: Some(LengthPercent::Percent(0.5)),
            ..Style::default()
        };
        assert_eq!(s.resolved_row_gap(), 0.0);
    }

    #[test]
    fn resolved_column_gap_prefers_column_gap_over_gap() {
        let s = Style {
            gap: Some(LengthPercent::Px(10.0)),
            column_gap: Some(LengthPercent::Px(30.0)),
            ..Style::default()
        };
        assert_eq!(s.resolved_column_gap(), 30.0);
    }

    #[test]
    fn merge_background_gradient_other_wins() {
        use crate::gradient::{ColorStop, GradientDirection};

        let g1 = LinearGradient {
            direction: GradientDirection::ToRight,
            stops: vec![ColorStop {
                color: Color::WHITE,
                position: 0.0,
            }],
        };
        let g2 = LinearGradient {
            direction: GradientDirection::ToBottom,
            stops: vec![ColorStop {
                color: Color::BLACK,
                position: 1.0,
            }],
        };

        let base = Style {
            background_gradient: Some(g1),
            ..Style::default()
        };
        let over = Style {
            background_gradient: Some(g2.clone()),
            ..Style::default()
        };
        let merged = base.merge(&over);
        assert_eq!(merged.background_gradient, Some(g2));
    }

    #[test]
    fn merge_background_gradient_none_preserves_base() {
        use crate::gradient::{ColorStop, GradientDirection};

        let g = LinearGradient {
            direction: GradientDirection::ToRight,
            stops: vec![ColorStop {
                color: Color::WHITE,
                position: 0.0,
            }],
        };
        let base = Style {
            background_gradient: Some(g.clone()),
            ..Style::default()
        };
        let over = Style::default();
        let merged = base.merge(&over);
        assert_eq!(merged.background_gradient, Some(g));
    }

    #[test]
    fn resolve_applies_gradient_from_hover() {
        use crate::gradient::{ColorStop, GradientDirection};

        let g = LinearGradient {
            direction: GradientDirection::ToRight,
            stops: vec![ColorStop {
                color: Color::BLACK,
                position: 0.0,
            }],
        };
        let style = Style {
            hover: Some(Box::new(Style {
                background_gradient: Some(g.clone()),
                ..Style::default()
            })),
            ..Style::default()
        };
        let resolved = style.resolve(&InteractionState {
            hovered: true,
            ..InteractionState::default()
        });
        assert_eq!(resolved.background_gradient, Some(g));
    }

    #[test]
    fn layout_enum_defaults() {
        assert_eq!(FlexDirection::default(), FlexDirection::Row);
        assert_eq!(FlexWrap::default(), FlexWrap::NoWrap);
        assert_eq!(AlignItems::default(), AlignItems::Stretch);
        assert_eq!(AlignSelf::default(), AlignSelf::Auto);
        assert_eq!(JustifyContent::default(), JustifyContent::FlexStart);
        assert_eq!(Overflow::default(), Overflow::Visible);
    }
}
