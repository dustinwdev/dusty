//! Conversion from `dusty_style::Style` to `taffy::Style`.

use dusty_style::{
    AlignItems, AlignSelf, Display, Edges, FlexDirection, FlexWrap, JustifyContent, Length,
    LengthPercent, Overflow, Position,
};

fn edges_to_length_percentage_border(edges: &Edges<f32>) -> taffy::Rect<taffy::LengthPercentage> {
    taffy::Rect {
        left: taffy::LengthPercentage::length(edges.left.unwrap_or(0.0)),
        right: taffy::LengthPercentage::length(edges.right.unwrap_or(0.0)),
        top: taffy::LengthPercentage::length(edges.top.unwrap_or(0.0)),
        bottom: taffy::LengthPercentage::length(edges.bottom.unwrap_or(0.0)),
    }
}

/// Converts a `dusty_style::Style` to a `taffy::Style`.
pub fn to_taffy_style(style: &dusty_style::Style) -> taffy::Style {
    taffy::Style {
        display: convert_display(style.display.unwrap_or_default()),
        position: convert_position(style.position.unwrap_or_default()),
        inset: edges_inset_to_auto(&style.inset),

        // Flex container
        flex_direction: convert_flex_direction(style.flex_direction.unwrap_or_default()),
        flex_wrap: convert_flex_wrap(style.flex_wrap.unwrap_or_default()),
        align_items: style.align_items.map(convert_align_items),
        align_self: convert_align_self(style.align_self.unwrap_or_default()),
        justify_content: style.justify_content.map(convert_justify_content),

        // Flex item
        flex_grow: style.flex_grow.unwrap_or(0.0),
        flex_shrink: style.flex_shrink.unwrap_or(1.0),
        flex_basis: length_to_dimension(style.flex_basis),
        aspect_ratio: style.aspect_ratio,

        // Size
        size: taffy::Size {
            width: length_to_dimension(style.width),
            height: length_to_dimension(style.height),
        },
        min_size: taffy::Size {
            width: length_to_dimension(style.min_width),
            height: length_to_dimension(style.min_height),
        },
        max_size: taffy::Size {
            width: length_to_dimension(style.max_width),
            height: length_to_dimension(style.max_height),
        },

        // Spacing
        padding: edges_length_percent_to_taffy(&style.padding),
        margin: edges_length_to_auto(&style.margin),
        border: edges_to_length_percentage_border(&style.border_width),
        gap: convert_gap(style.gap, style.row_gap, style.column_gap),

        // Overflow
        overflow: convert_overflow(style.overflow.unwrap_or_default()),

        ..taffy::Style::DEFAULT
    }
}

const fn length_to_dimension(val: Option<Length>) -> taffy::Dimension {
    match val {
        Some(Length::Px(v)) => taffy::Dimension::length(v),
        Some(Length::Percent(v)) => taffy::Dimension::percent(v),
        Some(Length::Auto) | None => taffy::Dimension::auto(),
    }
}

const fn length_percent_to_taffy(val: LengthPercent) -> taffy::LengthPercentage {
    match val {
        LengthPercent::Px(v) => taffy::LengthPercentage::length(v),
        LengthPercent::Percent(v) => taffy::LengthPercentage::percent(v),
    }
}

const fn length_to_length_percentage_auto(val: Option<Length>) -> taffy::LengthPercentageAuto {
    match val {
        Some(Length::Px(v)) => taffy::LengthPercentageAuto::length(v),
        Some(Length::Percent(v)) => taffy::LengthPercentageAuto::percent(v),
        Some(Length::Auto) => taffy::LengthPercentageAuto::auto(),
        None => taffy::LengthPercentageAuto::length(0.0),
    }
}

const fn edges_length_percent_to_taffy(
    edges: &Edges<LengthPercent>,
) -> taffy::Rect<taffy::LengthPercentage> {
    taffy::Rect {
        left: match edges.left {
            Some(v) => length_percent_to_taffy(v),
            None => taffy::LengthPercentage::length(0.0),
        },
        right: match edges.right {
            Some(v) => length_percent_to_taffy(v),
            None => taffy::LengthPercentage::length(0.0),
        },
        top: match edges.top {
            Some(v) => length_percent_to_taffy(v),
            None => taffy::LengthPercentage::length(0.0),
        },
        bottom: match edges.bottom {
            Some(v) => length_percent_to_taffy(v),
            None => taffy::LengthPercentage::length(0.0),
        },
    }
}

const fn edges_length_to_auto(edges: &Edges<Length>) -> taffy::Rect<taffy::LengthPercentageAuto> {
    taffy::Rect {
        left: length_to_length_percentage_auto(edges.left),
        right: length_to_length_percentage_auto(edges.right),
        top: length_to_length_percentage_auto(edges.top),
        bottom: length_to_length_percentage_auto(edges.bottom),
    }
}

/// Inset variant: an unset side maps to `auto`, not zero. Unset inset means
/// "no constraint" rather than "pinned to zero offset".
const fn edges_inset_to_auto(edges: &Edges<Length>) -> taffy::Rect<taffy::LengthPercentageAuto> {
    taffy::Rect {
        left: inset_side_to_auto(edges.left),
        right: inset_side_to_auto(edges.right),
        top: inset_side_to_auto(edges.top),
        bottom: inset_side_to_auto(edges.bottom),
    }
}

const fn inset_side_to_auto(val: Option<Length>) -> taffy::LengthPercentageAuto {
    match val {
        Some(Length::Px(v)) => taffy::LengthPercentageAuto::length(v),
        Some(Length::Percent(v)) => taffy::LengthPercentageAuto::percent(v),
        Some(Length::Auto) | None => taffy::LengthPercentageAuto::auto(),
    }
}

const fn convert_flex_direction(dir: FlexDirection) -> taffy::FlexDirection {
    match dir {
        FlexDirection::Row => taffy::FlexDirection::Row,
        FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
        FlexDirection::Column => taffy::FlexDirection::Column,
        FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
    }
}

const fn convert_flex_wrap(wrap: FlexWrap) -> taffy::FlexWrap {
    match wrap {
        FlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
        FlexWrap::Wrap => taffy::FlexWrap::Wrap,
        FlexWrap::WrapReverse => taffy::FlexWrap::WrapReverse,
    }
}

const fn convert_align_items(align: AlignItems) -> taffy::AlignItems {
    match align {
        AlignItems::FlexStart => taffy::AlignItems::FlexStart,
        AlignItems::FlexEnd => taffy::AlignItems::FlexEnd,
        AlignItems::Center => taffy::AlignItems::Center,
        AlignItems::Baseline => taffy::AlignItems::Baseline,
        AlignItems::Stretch => taffy::AlignItems::Stretch,
    }
}

const fn convert_align_self(align: AlignSelf) -> Option<taffy::AlignSelf> {
    match align {
        AlignSelf::Auto => None,
        AlignSelf::FlexStart => Some(taffy::AlignSelf::FlexStart),
        AlignSelf::FlexEnd => Some(taffy::AlignSelf::FlexEnd),
        AlignSelf::Center => Some(taffy::AlignSelf::Center),
        AlignSelf::Baseline => Some(taffy::AlignSelf::Baseline),
        AlignSelf::Stretch => Some(taffy::AlignSelf::Stretch),
    }
}

const fn convert_justify_content(justify: JustifyContent) -> taffy::JustifyContent {
    match justify {
        JustifyContent::FlexStart => taffy::JustifyContent::FlexStart,
        JustifyContent::FlexEnd => taffy::JustifyContent::FlexEnd,
        JustifyContent::Center => taffy::JustifyContent::Center,
        JustifyContent::SpaceBetween => taffy::JustifyContent::SpaceBetween,
        JustifyContent::SpaceAround => taffy::JustifyContent::SpaceAround,
        JustifyContent::SpaceEvenly => taffy::JustifyContent::SpaceEvenly,
    }
}

fn convert_gap(
    gap: Option<LengthPercent>,
    row_gap: Option<LengthPercent>,
    column_gap: Option<LengthPercent>,
) -> taffy::Size<taffy::LengthPercentage> {
    let base = gap.map_or(
        taffy::LengthPercentage::length(0.0),
        length_percent_to_taffy,
    );
    taffy::Size {
        width: column_gap.map_or(base, length_percent_to_taffy),
        height: row_gap.map_or(base, length_percent_to_taffy),
    }
}

const fn convert_overflow(overflow: Overflow) -> taffy::Point<taffy::Overflow> {
    let val = match overflow {
        Overflow::Visible => taffy::Overflow::Visible,
        Overflow::Hidden => taffy::Overflow::Hidden,
        Overflow::Scroll | Overflow::Auto => taffy::Overflow::Scroll,
    };
    taffy::Point { x: val, y: val }
}

const fn convert_display(display: Display) -> taffy::Display {
    match display {
        Display::Flex => taffy::Display::Flex,
        Display::Block => taffy::Display::Block,
        Display::None => taffy::Display::None,
    }
}

const fn convert_position(position: Position) -> taffy::Position {
    match position {
        Position::Relative => taffy::Position::Relative,
        Position::Absolute => taffy::Position::Absolute,
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use dusty_style::{Display, Edges, Length, LengthPercent, Position, Style};

    #[test]
    fn default_style_converts() {
        let dusty = Style::default();
        let taffy_style = to_taffy_style(&dusty);
        assert_eq!(taffy_style.display, taffy::Display::Flex);
        assert_eq!(taffy_style.flex_direction, taffy::FlexDirection::Row);
        assert_eq!(taffy_style.flex_wrap, taffy::FlexWrap::NoWrap);
        assert_eq!(taffy_style.flex_grow, 0.0);
        assert_eq!(taffy_style.flex_shrink, 1.0);
    }

    #[test]
    fn fixed_size_converts() {
        let dusty = Style {
            width: Some(Length::Px(200.0)),
            height: Some(Length::Px(100.0)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.size.width, taffy::Dimension::length(200.0));
        assert_eq!(t.size.height, taffy::Dimension::length(100.0));
    }

    #[test]
    fn auto_size_when_none() {
        let dusty = Style::default();
        let t = to_taffy_style(&dusty);
        assert!(t.size.width.is_auto());
        assert!(t.size.height.is_auto());
    }

    #[test]
    fn length_auto_converts_to_taffy_auto() {
        let dusty = Style {
            width: Some(Length::Auto),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert!(t.size.width.is_auto());
    }

    #[test]
    fn width_percent_converts_to_taffy_percent() {
        let dusty = Style {
            width: Some(Length::Percent(0.5)),
            height: Some(Length::Percent(0.25)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.size.width, taffy::Dimension::percent(0.5));
        assert_eq!(t.size.height, taffy::Dimension::percent(0.25));
    }

    #[test]
    fn min_max_size_converts() {
        let dusty = Style {
            min_width: Some(Length::Px(50.0)),
            max_height: Some(Length::Px(500.0)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.min_size.width, taffy::Dimension::length(50.0));
        assert!(t.min_size.height.is_auto());
        assert!(t.max_size.width.is_auto());
        assert_eq!(t.max_size.height, taffy::Dimension::length(500.0));
    }

    #[test]
    fn min_max_percent_converts() {
        let dusty = Style {
            min_width: Some(Length::Percent(0.1)),
            max_width: Some(Length::Percent(0.9)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.min_size.width, taffy::Dimension::percent(0.1));
        assert_eq!(t.max_size.width, taffy::Dimension::percent(0.9));
    }

    #[test]
    fn padding_converts() {
        let dusty = Style {
            padding: Edges::new(
                LengthPercent::Px(1.0),
                LengthPercent::Px(2.0),
                LengthPercent::Px(3.0),
                LengthPercent::Px(4.0),
            ),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.padding.top, taffy::LengthPercentage::length(1.0));
        assert_eq!(t.padding.right, taffy::LengthPercentage::length(2.0));
        assert_eq!(t.padding.bottom, taffy::LengthPercentage::length(3.0));
        assert_eq!(t.padding.left, taffy::LengthPercentage::length(4.0));
    }

    #[test]
    fn padding_percent_converts() {
        let dusty = Style {
            padding: Edges::all(LengthPercent::Percent(0.25)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.padding.top, taffy::LengthPercentage::percent(0.25));
        assert_eq!(t.padding.bottom, taffy::LengthPercentage::percent(0.25));
    }

    #[test]
    fn margin_converts() {
        let dusty = Style {
            margin: Edges::all(Length::Px(10.0)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.margin.top, taffy::LengthPercentageAuto::length(10.0));
        assert_eq!(t.margin.left, taffy::LengthPercentageAuto::length(10.0));
    }

    #[test]
    fn margin_percent_converts() {
        let dusty = Style {
            margin: Edges::all(Length::Percent(0.1)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.margin.top, taffy::LengthPercentageAuto::percent(0.1));
        assert_eq!(t.margin.left, taffy::LengthPercentageAuto::percent(0.1));
    }

    #[test]
    fn aspect_ratio_field_propagates() {
        let dusty = Style {
            aspect_ratio: Some(16.0 / 9.0),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.aspect_ratio, Some(16.0 / 9.0));
    }

    #[test]
    fn aspect_ratio_none_propagates_none() {
        let dusty = Style::default();
        let t = to_taffy_style(&dusty);
        assert_eq!(t.aspect_ratio, None);
    }

    #[test]
    fn border_width_converts() {
        let dusty = Style {
            border_width: Edges::all(2.0),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.border.top, taffy::LengthPercentage::length(2.0));
    }

    #[test]
    fn flex_direction_converts() {
        for (dusty_dir, taffy_dir) in [
            (FlexDirection::Row, taffy::FlexDirection::Row),
            (FlexDirection::RowReverse, taffy::FlexDirection::RowReverse),
            (FlexDirection::Column, taffy::FlexDirection::Column),
            (
                FlexDirection::ColumnReverse,
                taffy::FlexDirection::ColumnReverse,
            ),
        ] {
            let dusty = Style {
                flex_direction: Some(dusty_dir),
                ..Style::default()
            };
            assert_eq!(to_taffy_style(&dusty).flex_direction, taffy_dir);
        }
    }

    #[test]
    fn flex_wrap_converts() {
        for (dusty_wrap, taffy_wrap) in [
            (FlexWrap::NoWrap, taffy::FlexWrap::NoWrap),
            (FlexWrap::Wrap, taffy::FlexWrap::Wrap),
            (FlexWrap::WrapReverse, taffy::FlexWrap::WrapReverse),
        ] {
            let dusty = Style {
                flex_wrap: Some(dusty_wrap),
                ..Style::default()
            };
            assert_eq!(to_taffy_style(&dusty).flex_wrap, taffy_wrap);
        }
    }

    #[test]
    fn align_items_converts() {
        for (dusty_align, taffy_align) in [
            (AlignItems::FlexStart, taffy::AlignItems::FlexStart),
            (AlignItems::FlexEnd, taffy::AlignItems::FlexEnd),
            (AlignItems::Center, taffy::AlignItems::Center),
            (AlignItems::Baseline, taffy::AlignItems::Baseline),
            (AlignItems::Stretch, taffy::AlignItems::Stretch),
        ] {
            let dusty = Style {
                align_items: Some(dusty_align),
                ..Style::default()
            };
            assert_eq!(to_taffy_style(&dusty).align_items, Some(taffy_align));
        }
    }

    #[test]
    fn align_self_auto_maps_to_none() {
        let dusty = Style {
            align_self: Some(AlignSelf::Auto),
            ..Style::default()
        };
        assert_eq!(to_taffy_style(&dusty).align_self, None);
    }

    #[test]
    fn align_self_converts() {
        let dusty = Style {
            align_self: Some(AlignSelf::Center),
            ..Style::default()
        };
        assert_eq!(
            to_taffy_style(&dusty).align_self,
            Some(taffy::AlignSelf::Center)
        );
    }

    #[test]
    fn justify_content_converts() {
        for (dusty_jc, taffy_jc) in [
            (JustifyContent::FlexStart, taffy::JustifyContent::FlexStart),
            (JustifyContent::FlexEnd, taffy::JustifyContent::FlexEnd),
            (JustifyContent::Center, taffy::JustifyContent::Center),
            (
                JustifyContent::SpaceBetween,
                taffy::JustifyContent::SpaceBetween,
            ),
            (
                JustifyContent::SpaceAround,
                taffy::JustifyContent::SpaceAround,
            ),
            (
                JustifyContent::SpaceEvenly,
                taffy::JustifyContent::SpaceEvenly,
            ),
        ] {
            let dusty = Style {
                justify_content: Some(dusty_jc),
                ..Style::default()
            };
            assert_eq!(to_taffy_style(&dusty).justify_content, Some(taffy_jc));
        }
    }

    #[test]
    fn gap_base_value() {
        let dusty = Style {
            gap: Some(LengthPercent::Px(10.0)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.gap.width, taffy::LengthPercentage::length(10.0));
        assert_eq!(t.gap.height, taffy::LengthPercentage::length(10.0));
    }

    #[test]
    fn gap_percent_converts() {
        let dusty = Style {
            gap: Some(LengthPercent::Percent(0.05)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.gap.width, taffy::LengthPercentage::percent(0.05));
        assert_eq!(t.gap.height, taffy::LengthPercentage::percent(0.05));
    }

    #[test]
    fn row_gap_overrides_gap() {
        let dusty = Style {
            gap: Some(LengthPercent::Px(10.0)),
            row_gap: Some(LengthPercent::Px(20.0)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.gap.height, taffy::LengthPercentage::length(20.0));
        assert_eq!(t.gap.width, taffy::LengthPercentage::length(10.0));
    }

    #[test]
    fn column_gap_overrides_gap() {
        let dusty = Style {
            gap: Some(LengthPercent::Px(10.0)),
            column_gap: Some(LengthPercent::Px(30.0)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.gap.width, taffy::LengthPercentage::length(30.0));
        assert_eq!(t.gap.height, taffy::LengthPercentage::length(10.0));
    }

    #[test]
    fn overflow_converts() {
        let cases = [
            (Overflow::Visible, taffy::Overflow::Visible),
            (Overflow::Hidden, taffy::Overflow::Hidden),
            (Overflow::Scroll, taffy::Overflow::Scroll),
            (Overflow::Auto, taffy::Overflow::Scroll),
        ];
        for (dusty_of, taffy_of) in cases {
            let dusty = Style {
                overflow: Some(dusty_of),
                ..Style::default()
            };
            let t = to_taffy_style(&dusty);
            assert_eq!(t.overflow.x, taffy_of);
            assert_eq!(t.overflow.y, taffy_of);
        }
    }

    #[test]
    fn flex_grow_shrink_basis() {
        let dusty = Style {
            flex_grow: Some(2.0),
            flex_shrink: Some(0.5),
            flex_basis: Some(Length::Px(100.0)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.flex_grow, 2.0);
        assert_eq!(t.flex_shrink, 0.5);
        assert_eq!(t.flex_basis, taffy::Dimension::length(100.0));
    }

    #[test]
    fn flex_basis_percent_converts() {
        let dusty = Style {
            flex_basis: Some(Length::Percent(0.5)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.flex_basis, taffy::Dimension::percent(0.5));
    }

    #[test]
    fn flex_basis_auto_when_none() {
        let dusty = Style::default();
        let t = to_taffy_style(&dusty);
        assert!(t.flex_basis.is_auto());
    }

    #[test]
    fn unset_padding_is_zero() {
        let dusty = Style::default();
        let t = to_taffy_style(&dusty);
        assert_eq!(t.padding.top, taffy::LengthPercentage::length(0.0));
    }

    #[test]
    fn unset_margin_is_zero() {
        let dusty = Style::default();
        let t = to_taffy_style(&dusty);
        assert_eq!(t.margin.top, taffy::LengthPercentageAuto::length(0.0));
    }

    #[test]
    fn margin_auto_converts_to_taffy_auto() {
        let dusty = Style {
            margin: Edges::all(Length::Auto),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert!(t.margin.left.is_auto());
        assert!(t.margin.right.is_auto());
        assert!(t.margin.top.is_auto());
        assert!(t.margin.bottom.is_auto());
    }

    #[test]
    fn margin_length_converts_to_taffy_length() {
        let dusty = Style {
            margin: Edges::new(
                Length::Px(5.0),
                Length::Auto,
                Length::Px(10.0),
                Length::Auto,
            ),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.margin.top, taffy::LengthPercentageAuto::length(5.0));
        assert!(t.margin.right.is_auto());
        assert_eq!(t.margin.bottom, taffy::LengthPercentageAuto::length(10.0));
        assert!(t.margin.left.is_auto());
    }

    #[test]
    fn margin_none_defaults_to_zero() {
        let dusty = Style::default();
        let t = to_taffy_style(&dusty);
        assert_eq!(t.margin.top, taffy::LengthPercentageAuto::length(0.0));
        assert_eq!(t.margin.right, taffy::LengthPercentageAuto::length(0.0));
        assert_eq!(t.margin.bottom, taffy::LengthPercentageAuto::length(0.0));
        assert_eq!(t.margin.left, taffy::LengthPercentageAuto::length(0.0));
    }

    #[test]
    fn align_items_none_when_unset() {
        let dusty = Style::default();
        let t = to_taffy_style(&dusty);
        assert_eq!(t.align_items, None);
    }

    #[test]
    fn justify_content_none_when_unset() {
        let dusty = Style::default();
        let t = to_taffy_style(&dusty);
        assert_eq!(t.justify_content, None);
    }

    #[test]
    fn display_flex_default() {
        let dusty = Style::default();
        let t = to_taffy_style(&dusty);
        assert_eq!(t.display, taffy::Display::Flex);
    }

    #[test]
    fn display_none_converts() {
        let dusty = Style {
            display: Some(Display::None),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.display, taffy::Display::None);
    }

    #[test]
    fn display_block_converts() {
        let dusty = Style {
            display: Some(Display::Block),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.display, taffy::Display::Block);
    }

    #[test]
    fn position_relative_default() {
        let dusty = Style::default();
        let t = to_taffy_style(&dusty);
        assert_eq!(t.position, taffy::Position::Relative);
    }

    #[test]
    fn position_absolute_converts() {
        let dusty = Style {
            position: Some(Position::Absolute),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.position, taffy::Position::Absolute);
    }

    #[test]
    fn inset_values_convert() {
        let dusty = Style {
            inset: Edges::new(
                Length::Px(10.0),
                Length::Px(20.0),
                Length::Px(30.0),
                Length::Px(40.0),
            ),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.inset.top, taffy::LengthPercentageAuto::length(10.0));
        assert_eq!(t.inset.right, taffy::LengthPercentageAuto::length(20.0));
        assert_eq!(t.inset.bottom, taffy::LengthPercentageAuto::length(30.0));
        assert_eq!(t.inset.left, taffy::LengthPercentageAuto::length(40.0));
    }

    #[test]
    fn inset_unset_side_is_auto() {
        let dusty = Style {
            inset: Edges {
                top: Some(Length::Px(5.0)),
                ..Edges::default()
            },
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.inset.top, taffy::LengthPercentageAuto::length(5.0));
        assert!(t.inset.left.is_auto());
        assert!(t.inset.right.is_auto());
        assert!(t.inset.bottom.is_auto());
    }

    #[test]
    fn inset_percent_converts() {
        let dusty = Style {
            inset: Edges::all(Length::Percent(0.1)),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.inset.top, taffy::LengthPercentageAuto::percent(0.1));
        assert_eq!(t.inset.left, taffy::LengthPercentageAuto::percent(0.1));
    }
}
