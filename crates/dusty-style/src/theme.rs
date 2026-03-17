//! Theme struct and context-based theme propagation.

use crate::palette::{ColorScale, Palette};
use crate::Color;

/// A theme providing semantic color mappings and surface/text colors.
///
/// # Known limitations
///
/// Both [`Theme::light()`] and [`Theme::dark()`] use the same underlying
/// [`Palette`](crate::Palette) color scales (e.g., `Palette::BLUE` for primary).
/// Only the surface/text/border colors differ between modes. A future release
/// will support distinct palette tints per mode (e.g., lighter blues for dark
/// backgrounds).
///
/// # Examples
///
/// ```
/// use dusty_style::theme::Theme;
///
/// let light = Theme::light();
/// let dark = Theme::dark();
/// assert_ne!(light.background, dark.background);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Primary brand color scale.
    pub primary: ColorScale,
    /// Secondary color scale.
    pub secondary: ColorScale,
    /// Accent color scale.
    pub accent: ColorScale,
    /// Danger/error color scale.
    pub danger: ColorScale,
    /// Warning color scale.
    pub warning: ColorScale,
    /// Success color scale.
    pub success: ColorScale,
    /// Informational color scale.
    pub info: ColorScale,
    /// Page/window background.
    pub background: Color,
    /// Card/elevated surface background.
    pub surface: Color,
    /// Primary text color.
    pub foreground: Color,
    /// Muted/secondary text color.
    pub muted: Color,
    /// Default border color.
    pub border: Color,
}

impl Theme {
    /// Light theme — blue primary, white backgrounds, dark text.
    #[must_use]
    pub fn light() -> Self {
        Self {
            primary: Palette::BLUE,
            secondary: Palette::SLATE,
            accent: Palette::VIOLET,
            danger: Palette::RED,
            warning: Palette::AMBER,
            success: Palette::GREEN,
            info: Palette::SKY,
            background: Color::WHITE,
            surface: Color::WHITE,
            foreground: Palette::SLATE.get(900).unwrap_or(Color::BLACK),
            muted: Palette::SLATE.get(500).unwrap_or(Color::BLACK),
            border: Palette::SLATE.get(200).unwrap_or(Color::BLACK),
        }
    }

    /// Dark theme — dark backgrounds, light text.
    ///
    /// Uses the same palette color scales as [`Theme::light()`]; see the
    /// type-level docs for details on this limitation.
    #[must_use]
    pub fn dark() -> Self {
        Self {
            primary: Palette::BLUE,
            secondary: Palette::SLATE,
            accent: Palette::VIOLET,
            danger: Palette::RED,
            warning: Palette::AMBER,
            success: Palette::GREEN,
            info: Palette::SKY,
            background: Palette::SLATE.get(950).unwrap_or(Color::BLACK),
            surface: Palette::SLATE.get(900).unwrap_or(Color::BLACK),
            foreground: Palette::SLATE.get(50).unwrap_or(Color::WHITE),
            muted: Palette::SLATE.get(400).unwrap_or(Color::WHITE),
            border: Palette::SLATE.get(800).unwrap_or(Color::BLACK),
        }
    }
}

/// Provide a [`Theme`] to the current reactive scope and its descendants.
///
/// # Panics
///
/// Panics if no reactive scope is active.
pub fn provide_theme(theme: Theme) {
    dusty_reactive::provide_context(theme);
}

/// Retrieve the current [`Theme`] from the scope tree. Falls back to
/// [`Theme::light()`] when no provider exists.
///
/// # Panics
///
/// Panics if the reactive runtime is not initialized.
pub fn use_theme() -> Theme {
    dusty_reactive::use_context::<Theme>().unwrap_or_else(Theme::light)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_theme_defaults() {
        let t = Theme::light();
        assert_eq!(t.background, Color::WHITE);
        assert_eq!(t.primary, Palette::BLUE);
        assert_eq!(
            t.foreground,
            Palette::SLATE.get(900).unwrap_or(Color::BLACK)
        );
    }

    #[test]
    fn dark_theme_defaults() {
        let t = Theme::dark();
        assert_eq!(
            t.background,
            Palette::SLATE.get(950).unwrap_or(Color::BLACK)
        );
        assert_eq!(t.foreground, Palette::SLATE.get(50).unwrap_or(Color::WHITE));
    }

    #[test]
    fn light_and_dark_differ() {
        assert_ne!(Theme::light(), Theme::dark());
    }

    #[test]
    fn provide_and_use_theme_roundtrip() {
        dusty_reactive::initialize_runtime();
        let _scope = dusty_reactive::create_scope(|_s| {
            provide_theme(Theme::dark());
            let t = use_theme();
            assert_eq!(t, Theme::dark());
        });
        dusty_reactive::dispose_runtime();
    }

    #[test]
    fn nested_scope_inherits_theme() {
        dusty_reactive::initialize_runtime();
        let _scope = dusty_reactive::create_scope(|p| {
            provide_theme(Theme::dark());
            let _child = dusty_reactive::create_child_scope(p, |_c| {
                let t = use_theme();
                assert_eq!(t, Theme::dark());
            });
        });
        dusty_reactive::dispose_runtime();
    }

    #[test]
    fn child_overrides_theme() {
        dusty_reactive::initialize_runtime();
        let _scope = dusty_reactive::create_scope(|p| {
            provide_theme(Theme::dark());
            let _child = dusty_reactive::create_child_scope(p, |_c| {
                provide_theme(Theme::light());
                let t = use_theme();
                assert_eq!(t, Theme::light());
            });
            // Parent still has dark
            let t = use_theme();
            assert_eq!(t, Theme::dark());
        });
        dusty_reactive::dispose_runtime();
    }

    #[test]
    fn fallback_to_light_when_no_provider() {
        dusty_reactive::initialize_runtime();
        let _scope = dusty_reactive::create_scope(|_s| {
            let t = use_theme();
            assert_eq!(t, Theme::light());
        });
        dusty_reactive::dispose_runtime();
    }
}
