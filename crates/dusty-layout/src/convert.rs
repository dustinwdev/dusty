//! Conversion from `dusty_style::Style` to `taffy::Style`.

use dusty_style::{AlignItems, AlignSelf, FlexDirection, FlexWrap, JustifyContent, Overflow};

/// Converts a `dusty_style::Style` to a `taffy::Style`.
pub fn to_taffy_style(style: &dusty_style::Style) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,

        // Flex container
        flex_direction: convert_flex_direction(style.flex_direction.unwrap_or_default()),
        flex_wrap: convert_flex_wrap(style.flex_wrap.unwrap_or_default()),
        align_items: style.align_items.map(convert_align_items),
        align_self: convert_align_self(style.align_self.unwrap_or_default()),
        justify_content: style.justify_content.map(convert_justify_content),

        // Flex item
        flex_grow: style.flex_grow.unwrap_or(0.0),
        flex_shrink: style.flex_shrink.unwrap_or(1.0),
        flex_basis: option_to_dimension(style.flex_basis),

        // Size
        size: taffy::Size {
            width: option_to_dimension(style.width),
            height: option_to_dimension(style.height),
        },
        min_size: taffy::Size {
            width: option_to_dimension(style.min_width),
            height: option_to_dimension(style.min_height),
        },
        max_size: taffy::Size {
            width: option_to_dimension(style.max_width),
            height: option_to_dimension(style.max_height),
        },

        // Spacing
        padding: edges_to_length_percentage(&style.padding),
        margin: edges_to_length_percentage_auto(&style.margin),
        border: edges_to_length_percentage(&style.border_width),
        gap: convert_gap(style.gap, style.row_gap, style.column_gap),

        // Overflow
        overflow: convert_overflow(style.overflow.unwrap_or_default()),

        ..taffy::Style::DEFAULT
    }
}

const fn option_to_dimension(val: Option<f32>) -> taffy::Dimension {
    match val {
        Some(v) => taffy::Dimension::length(v),
        None => taffy::Dimension::auto(),
    }
}

fn edges_to_length_percentage(
    edges: &dusty_style::Edges<f32>,
) -> taffy::Rect<taffy::LengthPercentage> {
    taffy::Rect {
        left: taffy::LengthPercentage::length(edges.left.unwrap_or(0.0)),
        right: taffy::LengthPercentage::length(edges.right.unwrap_or(0.0)),
        top: taffy::LengthPercentage::length(edges.top.unwrap_or(0.0)),
        bottom: taffy::LengthPercentage::length(edges.bottom.unwrap_or(0.0)),
    }
}

fn edges_to_length_percentage_auto(
    edges: &dusty_style::Edges<f32>,
) -> taffy::Rect<taffy::LengthPercentageAuto> {
    taffy::Rect {
        left: taffy::LengthPercentageAuto::length(edges.left.unwrap_or(0.0)),
        right: taffy::LengthPercentageAuto::length(edges.right.unwrap_or(0.0)),
        top: taffy::LengthPercentageAuto::length(edges.top.unwrap_or(0.0)),
        bottom: taffy::LengthPercentageAuto::length(edges.bottom.unwrap_or(0.0)),
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
    gap: Option<f32>,
    row_gap: Option<f32>,
    column_gap: Option<f32>,
) -> taffy::Size<taffy::LengthPercentage> {
    let base = gap.unwrap_or(0.0);
    taffy::Size {
        width: taffy::LengthPercentage::length(column_gap.unwrap_or(base)),
        height: taffy::LengthPercentage::length(row_gap.unwrap_or(base)),
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

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use dusty_style::{Edges, Style};

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
            width: Some(200.0),
            height: Some(100.0),
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
    fn min_max_size_converts() {
        let dusty = Style {
            min_width: Some(50.0),
            max_height: Some(500.0),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.min_size.width, taffy::Dimension::length(50.0));
        assert!(t.min_size.height.is_auto());
        assert!(t.max_size.width.is_auto());
        assert_eq!(t.max_size.height, taffy::Dimension::length(500.0));
    }

    #[test]
    fn padding_converts() {
        let dusty = Style {
            padding: Edges::new(1.0, 2.0, 3.0, 4.0),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.padding.top, taffy::LengthPercentage::length(1.0));
        assert_eq!(t.padding.right, taffy::LengthPercentage::length(2.0));
        assert_eq!(t.padding.bottom, taffy::LengthPercentage::length(3.0));
        assert_eq!(t.padding.left, taffy::LengthPercentage::length(4.0));
    }

    #[test]
    fn margin_converts() {
        let dusty = Style {
            margin: Edges::all(10.0),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.margin.top, taffy::LengthPercentageAuto::length(10.0));
        assert_eq!(t.margin.left, taffy::LengthPercentageAuto::length(10.0));
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
            gap: Some(10.0),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.gap.width, taffy::LengthPercentage::length(10.0));
        assert_eq!(t.gap.height, taffy::LengthPercentage::length(10.0));
    }

    #[test]
    fn row_gap_overrides_gap() {
        let dusty = Style {
            gap: Some(10.0),
            row_gap: Some(20.0),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.gap.height, taffy::LengthPercentage::length(20.0));
        assert_eq!(t.gap.width, taffy::LengthPercentage::length(10.0));
    }

    #[test]
    fn column_gap_overrides_gap() {
        let dusty = Style {
            gap: Some(10.0),
            column_gap: Some(30.0),
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
            flex_basis: Some(100.0),
            ..Style::default()
        };
        let t = to_taffy_style(&dusty);
        assert_eq!(t.flex_grow, 2.0);
        assert_eq!(t.flex_shrink, 0.5);
        assert_eq!(t.flex_basis, taffy::Dimension::length(100.0));
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
}
