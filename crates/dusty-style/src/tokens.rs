//! Design token scales for spacing, border-radius, and shadows.

use std::borrow::Cow;

use crate::{BoxShadow, Color};

/// Returns the spacing value for a given key.
///
/// Follows Tailwind's 4px base unit: `spacing(key) = key * 4.0`.
///
/// # Examples
///
/// ```
/// use dusty_style::tokens::spacing;
///
/// assert_eq!(spacing(1.0), 4.0);
/// assert_eq!(spacing(4.0), 16.0);
/// assert_eq!(spacing(0.5), 2.0);
/// ```
#[must_use]
pub fn spacing(key: f32) -> f32 {
    key * 4.0
}

/// Predefined border-radius tokens matching Tailwind's scale.
///
/// # Examples
///
/// ```
/// use dusty_style::tokens::RadiusToken;
///
/// assert_eq!(RadiusToken::Md.to_px(), 6.0);
/// assert_eq!(RadiusToken::Full.to_px(), 9999.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadiusToken {
    /// No rounding (0px).
    None,
    /// Small rounding (2px).
    Sm,
    /// Medium rounding (6px).
    Md,
    /// Large rounding (8px).
    Lg,
    /// Extra-large rounding (12px).
    Xl,
    /// 2x extra-large rounding (16px).
    Xl2,
    /// 3x extra-large rounding (24px).
    Xl3,
    /// Full/pill rounding (9999px).
    Full,
}

impl RadiusToken {
    /// Returns the pixel value for this radius token.
    #[must_use]
    pub const fn to_px(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Sm => 2.0,
            Self::Md => 6.0,
            Self::Lg => 8.0,
            Self::Xl => 12.0,
            Self::Xl2 => 16.0,
            Self::Xl3 => 24.0,
            Self::Full => 9999.0,
        }
    }
}

/// Predefined box-shadow tokens matching Tailwind's shadow scale.
///
/// # Examples
///
/// ```
/// use dusty_style::tokens::ShadowToken;
///
/// let shadows = ShadowToken::Md.to_shadows();
/// assert_eq!(shadows.len(), 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShadowToken {
    /// No shadow.
    None,
    /// Small shadow.
    Sm,
    /// Medium shadow (default).
    Md,
    /// Large shadow.
    Lg,
    /// Extra-large shadow.
    Xl,
    /// 2x extra-large shadow.
    Xl2,
    /// Inner (inset) shadow.
    Inner,
}

/// Shadow color: black at 5% opacity.
const SHADOW_05: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.05,
};

/// Shadow color: black at 10% opacity.
const SHADOW_10: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.1,
};

/// Shadow color: black at 25% opacity.
const SHADOW_25: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.25,
};

/// Shadow constants used by [`ShadowToken::to_shadows`].
const SHADOW_SM: &[BoxShadow] = &[BoxShadow {
    offset_x: 0.0,
    offset_y: 1.0,
    blur_radius: 2.0,
    spread_radius: 0.0,
    color: SHADOW_05,
    inset: false,
}];

const SHADOW_MD: &[BoxShadow] = &[
    BoxShadow {
        offset_x: 0.0,
        offset_y: 1.0,
        blur_radius: 3.0,
        spread_radius: 0.0,
        color: SHADOW_10,
        inset: false,
    },
    BoxShadow {
        offset_x: 0.0,
        offset_y: 1.0,
        blur_radius: 2.0,
        spread_radius: -1.0,
        color: SHADOW_10,
        inset: false,
    },
];

const SHADOW_LG: &[BoxShadow] = &[
    BoxShadow {
        offset_x: 0.0,
        offset_y: 10.0,
        blur_radius: 15.0,
        spread_radius: -3.0,
        color: SHADOW_10,
        inset: false,
    },
    BoxShadow {
        offset_x: 0.0,
        offset_y: 4.0,
        blur_radius: 6.0,
        spread_radius: -4.0,
        color: SHADOW_10,
        inset: false,
    },
];

const SHADOW_XL: &[BoxShadow] = &[
    BoxShadow {
        offset_x: 0.0,
        offset_y: 20.0,
        blur_radius: 25.0,
        spread_radius: -5.0,
        color: SHADOW_10,
        inset: false,
    },
    BoxShadow {
        offset_x: 0.0,
        offset_y: 8.0,
        blur_radius: 10.0,
        spread_radius: -6.0,
        color: SHADOW_10,
        inset: false,
    },
];

const SHADOW_XL2: &[BoxShadow] = &[BoxShadow {
    offset_x: 0.0,
    offset_y: 25.0,
    blur_radius: 50.0,
    spread_radius: -12.0,
    color: SHADOW_25,
    inset: false,
}];

const SHADOW_INNER: &[BoxShadow] = &[BoxShadow {
    offset_x: 0.0,
    offset_y: 2.0,
    blur_radius: 4.0,
    spread_radius: 0.0,
    color: SHADOW_05,
    inset: true,
}];

impl ShadowToken {
    /// Returns the shadow layers for this token, matching Tailwind's defaults.
    ///
    /// Returns a borrowed static slice for all variants, avoiding allocation.
    #[must_use]
    pub const fn to_shadows(self) -> Cow<'static, [BoxShadow]> {
        match self {
            Self::None => Cow::Borrowed(&[]),
            Self::Sm => Cow::Borrowed(SHADOW_SM),
            Self::Md => Cow::Borrowed(SHADOW_MD),
            Self::Lg => Cow::Borrowed(SHADOW_LG),
            Self::Xl => Cow::Borrowed(SHADOW_XL),
            Self::Xl2 => Cow::Borrowed(SHADOW_XL2),
            Self::Inner => Cow::Borrowed(SHADOW_INNER),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_formula() {
        assert_eq!(spacing(0.0), 0.0);
        assert_eq!(spacing(0.5), 2.0);
        assert_eq!(spacing(1.0), 4.0);
        assert_eq!(spacing(2.0), 8.0);
        assert_eq!(spacing(4.0), 16.0);
        assert_eq!(spacing(8.0), 32.0);
        assert_eq!(spacing(16.0), 64.0);
    }

    #[test]
    fn radius_token_px_values() {
        assert_eq!(RadiusToken::None.to_px(), 0.0);
        assert_eq!(RadiusToken::Sm.to_px(), 2.0);
        assert_eq!(RadiusToken::Md.to_px(), 6.0);
        assert_eq!(RadiusToken::Lg.to_px(), 8.0);
        assert_eq!(RadiusToken::Xl.to_px(), 12.0);
        assert_eq!(RadiusToken::Xl2.to_px(), 16.0);
        assert_eq!(RadiusToken::Xl3.to_px(), 24.0);
        assert_eq!(RadiusToken::Full.to_px(), 9999.0);
    }

    #[test]
    fn shadow_none_is_empty() {
        assert!(ShadowToken::None.to_shadows().is_empty());
    }

    #[test]
    fn shadow_sm_is_single_layer() {
        let shadows = ShadowToken::Sm.to_shadows();
        assert_eq!(shadows.len(), 1);
        assert!(!shadows[0].inset);
        assert_eq!(shadows[0].offset_y, 1.0);
    }

    #[test]
    fn shadow_md_is_two_layers() {
        let shadows = ShadowToken::Md.to_shadows();
        assert_eq!(shadows.len(), 2);
    }

    #[test]
    fn shadow_lg_is_two_layers() {
        let shadows = ShadowToken::Lg.to_shadows();
        assert_eq!(shadows.len(), 2);
        assert_eq!(shadows[0].offset_y, 10.0);
    }

    #[test]
    fn shadow_xl_is_two_layers() {
        let shadows = ShadowToken::Xl.to_shadows();
        assert_eq!(shadows.len(), 2);
        assert_eq!(shadows[0].offset_y, 20.0);
    }

    #[test]
    fn shadow_xl2_is_single_deep_layer() {
        let shadows = ShadowToken::Xl2.to_shadows();
        assert_eq!(shadows.len(), 1);
        assert_eq!(shadows[0].offset_y, 25.0);
        assert_eq!(shadows[0].blur_radius, 50.0);
    }

    #[test]
    fn shadow_inner_is_inset() {
        let shadows = ShadowToken::Inner.to_shadows();
        assert_eq!(shadows.len(), 1);
        assert!(shadows[0].inset);
    }

    #[test]
    fn shadow_to_shadows_returns_borrowed() {
        // All variants return static borrows, avoiding allocation
        let cow = ShadowToken::Md.to_shadows();
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));

        let cow_none = ShadowToken::None.to_shadows();
        assert!(matches!(cow_none, std::borrow::Cow::Borrowed(_)));
        assert!(cow_none.is_empty());
    }
}
