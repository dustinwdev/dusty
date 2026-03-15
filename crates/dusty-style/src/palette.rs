//! Tailwind-inspired color palette with 22 hues and 11 stops each.

#![allow(clippy::unreadable_literal)]

use crate::Color;

/// Decomposes a 24-bit hex value into RGB bytes.
const fn rgb(hex: u32) -> [u8; 3] {
    #[allow(clippy::cast_possible_truncation)]
    [
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    ]
}

/// A scale of 11 color stops (50, 100, 200, …, 900, 950) for a single hue.
///
/// # Examples
///
/// ```
/// use dusty_style::palette::Palette;
///
/// let blue_500 = Palette::BLUE.get(500).unwrap();
/// assert!((blue_500.r - 0.231).abs() < 0.01);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorScale {
    stops: [[u8; 3]; 11],
}

impl ColorScale {
    /// Creates a new color scale from 11 RGB byte triples.
    /// Stops correspond to: 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950.
    #[must_use]
    pub const fn new(stops: [[u8; 3]; 11]) -> Self {
        Self { stops }
    }

    /// Looks up a color by stop value. Valid stops: 50, 100, 200–900, 950.
    /// Returns `None` for invalid stop values.
    #[must_use]
    pub fn get(&self, stop: u16) -> Option<Color> {
        let index = match stop {
            50 => 0,
            100 => 1,
            200 => 2,
            300 => 3,
            400 => 4,
            500 => 5,
            600 => 6,
            700 => 7,
            800 => 8,
            900 => 9,
            950 => 10,
            _ => return None,
        };
        let [r, g, b] = self.stops[index];
        Some(Color::rgb8(r, g, b))
    }
}

/// All valid stop values in order.
pub const STOPS: [u16; 11] = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

/// Groups all 22 Tailwind hues as associated constants.
pub struct Palette;

impl Palette {
    /// Slate — cool gray with a blue undertone.
    pub const SLATE: ColorScale = ColorScale::new([
        rgb(0xf8fafc),
        rgb(0xf1f5f9),
        rgb(0xe2e8f0),
        rgb(0xcbd5e1),
        rgb(0x94a3b8),
        rgb(0x64748b),
        rgb(0x475569),
        rgb(0x334155),
        rgb(0x1e293b),
        rgb(0x0f172a),
        rgb(0x020617),
    ]);

    /// Gray — neutral gray.
    pub const GRAY: ColorScale = ColorScale::new([
        rgb(0xf9fafb),
        rgb(0xf3f4f6),
        rgb(0xe5e7eb),
        rgb(0xd1d5db),
        rgb(0x9ca3af),
        rgb(0x6b7280),
        rgb(0x4b5563),
        rgb(0x374151),
        rgb(0x1f2937),
        rgb(0x111827),
        rgb(0x030712),
    ]);

    /// Zinc — cool gray with slight warmth.
    pub const ZINC: ColorScale = ColorScale::new([
        rgb(0xfafafa),
        rgb(0xf4f4f5),
        rgb(0xe4e4e7),
        rgb(0xd4d4d8),
        rgb(0xa1a1aa),
        rgb(0x71717a),
        rgb(0x52525b),
        rgb(0x3f3f46),
        rgb(0x27272a),
        rgb(0x18181b),
        rgb(0x09090b),
    ]);

    /// Neutral — pure neutral gray.
    pub const NEUTRAL: ColorScale = ColorScale::new([
        rgb(0xfafafa),
        rgb(0xf5f5f5),
        rgb(0xe5e5e5),
        rgb(0xd4d4d4),
        rgb(0xa3a3a3),
        rgb(0x737373),
        rgb(0x525252),
        rgb(0x404040),
        rgb(0x262626),
        rgb(0x171717),
        rgb(0x0a0a0a),
    ]);

    /// Stone — warm gray.
    pub const STONE: ColorScale = ColorScale::new([
        rgb(0xfafaf9),
        rgb(0xf5f5f4),
        rgb(0xe7e5e4),
        rgb(0xd6d3d1),
        rgb(0xa8a29e),
        rgb(0x78716c),
        rgb(0x57534e),
        rgb(0x44403c),
        rgb(0x292524),
        rgb(0x1c1917),
        rgb(0x0c0a09),
    ]);

    /// Red.
    pub const RED: ColorScale = ColorScale::new([
        rgb(0xfef2f2),
        rgb(0xfee2e2),
        rgb(0xfecaca),
        rgb(0xfca5a5),
        rgb(0xf87171),
        rgb(0xef4444),
        rgb(0xdc2626),
        rgb(0xb91c1c),
        rgb(0x991b1b),
        rgb(0x7f1d1d),
        rgb(0x450a0a),
    ]);

    /// Orange.
    pub const ORANGE: ColorScale = ColorScale::new([
        rgb(0xfff7ed),
        rgb(0xffedd5),
        rgb(0xfed7aa),
        rgb(0xfdba74),
        rgb(0xfb923c),
        rgb(0xf97316),
        rgb(0xea580c),
        rgb(0xc2410c),
        rgb(0x9a3412),
        rgb(0x7c2d12),
        rgb(0x431407),
    ]);

    /// Amber.
    pub const AMBER: ColorScale = ColorScale::new([
        rgb(0xfffbeb),
        rgb(0xfef3c7),
        rgb(0xfde68a),
        rgb(0xfcd34d),
        rgb(0xfbbf24),
        rgb(0xf59e0b),
        rgb(0xd97706),
        rgb(0xb45309),
        rgb(0x92400e),
        rgb(0x78350f),
        rgb(0x451a03),
    ]);

    /// Yellow.
    pub const YELLOW: ColorScale = ColorScale::new([
        rgb(0xfefce8),
        rgb(0xfef9c3),
        rgb(0xfef08a),
        rgb(0xfde047),
        rgb(0xfacc15),
        rgb(0xeab308),
        rgb(0xca8a04),
        rgb(0xa16207),
        rgb(0x854d0e),
        rgb(0x713f12),
        rgb(0x422006),
    ]);

    /// Lime.
    pub const LIME: ColorScale = ColorScale::new([
        rgb(0xf7fee7),
        rgb(0xecfccb),
        rgb(0xd9f99d),
        rgb(0xbef264),
        rgb(0xa3e635),
        rgb(0x84cc16),
        rgb(0x65a30d),
        rgb(0x4d7c0f),
        rgb(0x3f6212),
        rgb(0x365314),
        rgb(0x1a2e05),
    ]);

    /// Green.
    pub const GREEN: ColorScale = ColorScale::new([
        rgb(0xf0fdf4),
        rgb(0xdcfce7),
        rgb(0xbbf7d0),
        rgb(0x86efac),
        rgb(0x4ade80),
        rgb(0x22c55e),
        rgb(0x16a34a),
        rgb(0x15803d),
        rgb(0x166534),
        rgb(0x14532d),
        rgb(0x052e16),
    ]);

    /// Emerald.
    pub const EMERALD: ColorScale = ColorScale::new([
        rgb(0xecfdf5),
        rgb(0xd1fae5),
        rgb(0xa7f3d0),
        rgb(0x6ee7b7),
        rgb(0x34d399),
        rgb(0x10b981),
        rgb(0x059669),
        rgb(0x047857),
        rgb(0x065f46),
        rgb(0x064e3b),
        rgb(0x022c22),
    ]);

    /// Teal.
    pub const TEAL: ColorScale = ColorScale::new([
        rgb(0xf0fdfa),
        rgb(0xccfbf1),
        rgb(0x99f6e4),
        rgb(0x5eead4),
        rgb(0x2dd4bf),
        rgb(0x14b8a6),
        rgb(0x0d9488),
        rgb(0x0f766e),
        rgb(0x115e59),
        rgb(0x134e4a),
        rgb(0x042f2e),
    ]);

    /// Cyan.
    pub const CYAN: ColorScale = ColorScale::new([
        rgb(0xecfeff),
        rgb(0xcffafe),
        rgb(0xa5f3fc),
        rgb(0x67e8f9),
        rgb(0x22d3ee),
        rgb(0x06b6d4),
        rgb(0x0891b2),
        rgb(0x0e7490),
        rgb(0x155e75),
        rgb(0x164e63),
        rgb(0x083344),
    ]);

    /// Sky.
    pub const SKY: ColorScale = ColorScale::new([
        rgb(0xf0f9ff),
        rgb(0xe0f2fe),
        rgb(0xbae6fd),
        rgb(0x7dd3fc),
        rgb(0x38bdf8),
        rgb(0x0ea5e9),
        rgb(0x0284c7),
        rgb(0x0369a1),
        rgb(0x075985),
        rgb(0x0c4a6e),
        rgb(0x082f49),
    ]);

    /// Blue.
    pub const BLUE: ColorScale = ColorScale::new([
        rgb(0xeff6ff),
        rgb(0xdbeafe),
        rgb(0xbfdbfe),
        rgb(0x93c5fd),
        rgb(0x60a5fa),
        rgb(0x3b82f6),
        rgb(0x2563eb),
        rgb(0x1d4ed8),
        rgb(0x1e40af),
        rgb(0x1e3a8a),
        rgb(0x172554),
    ]);

    /// Indigo.
    pub const INDIGO: ColorScale = ColorScale::new([
        rgb(0xeef2ff),
        rgb(0xe0e7ff),
        rgb(0xc7d2fe),
        rgb(0xa5b4fc),
        rgb(0x818cf8),
        rgb(0x6366f1),
        rgb(0x4f46e5),
        rgb(0x4338ca),
        rgb(0x3730a3),
        rgb(0x312e81),
        rgb(0x1e1b4b),
    ]);

    /// Violet.
    pub const VIOLET: ColorScale = ColorScale::new([
        rgb(0xf5f3ff),
        rgb(0xede9fe),
        rgb(0xddd6fe),
        rgb(0xc4b5fd),
        rgb(0xa78bfa),
        rgb(0x8b5cf6),
        rgb(0x7c3aed),
        rgb(0x6d28d9),
        rgb(0x5b21b6),
        rgb(0x4c1d95),
        rgb(0x2e1065),
    ]);

    /// Purple.
    pub const PURPLE: ColorScale = ColorScale::new([
        rgb(0xfaf5ff),
        rgb(0xf3e8ff),
        rgb(0xe9d5ff),
        rgb(0xd8b4fe),
        rgb(0xc084fc),
        rgb(0xa855f7),
        rgb(0x9333ea),
        rgb(0x7e22ce),
        rgb(0x6b21a8),
        rgb(0x581c87),
        rgb(0x3b0764),
    ]);

    /// Fuchsia.
    pub const FUCHSIA: ColorScale = ColorScale::new([
        rgb(0xfdf4ff),
        rgb(0xfae8ff),
        rgb(0xf5d0fe),
        rgb(0xf0abfc),
        rgb(0xe879f9),
        rgb(0xd946ef),
        rgb(0xc026d3),
        rgb(0xa21caf),
        rgb(0x86198f),
        rgb(0x701a75),
        rgb(0x4a044e),
    ]);

    /// Pink.
    pub const PINK: ColorScale = ColorScale::new([
        rgb(0xfdf2f8),
        rgb(0xfce7f3),
        rgb(0xfbcfe8),
        rgb(0xf9a8d4),
        rgb(0xf472b6),
        rgb(0xec4899),
        rgb(0xdb2777),
        rgb(0xbe185d),
        rgb(0x9d174d),
        rgb(0x831843),
        rgb(0x500724),
    ]);

    /// Rose.
    pub const ROSE: ColorScale = ColorScale::new([
        rgb(0xfff1f2),
        rgb(0xffe4e6),
        rgb(0xfecdd3),
        rgb(0xfda4af),
        rgb(0xfb7185),
        rgb(0xf43f5e),
        rgb(0xe11d48),
        rgb(0xbe123c),
        rgb(0x9f1239),
        rgb(0x881337),
        rgb(0x4c0519),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blue_500_matches_tailwind() {
        // Tailwind blue-500: #3b82f6
        let c = Palette::BLUE.get(500);
        assert!(c.is_some());
        let c = c.unwrap_or(Color::BLACK);
        assert_eq!(c, Color::hex(0x3b82f6));
    }

    #[test]
    fn red_500_matches_tailwind() {
        // Tailwind red-500: #ef4444
        let c = Palette::RED.get(500).unwrap_or(Color::BLACK);
        assert_eq!(c, Color::hex(0xef4444));
    }

    #[test]
    fn slate_50_matches_tailwind() {
        // Tailwind slate-50: #f8fafc
        let c = Palette::SLATE.get(50).unwrap_or(Color::BLACK);
        assert_eq!(c, Color::hex(0xf8fafc));
    }

    #[test]
    fn green_900_matches_tailwind() {
        // Tailwind green-900: #14532d
        let c = Palette::GREEN.get(900).unwrap_or(Color::BLACK);
        assert_eq!(c, Color::hex(0x14532d));
    }

    #[test]
    fn invalid_stop_returns_none() {
        assert!(Palette::BLUE.get(0).is_none());
        assert!(Palette::BLUE.get(150).is_none());
        assert!(Palette::BLUE.get(1000).is_none());
        assert!(Palette::BLUE.get(42).is_none());
    }

    #[test]
    fn all_hues_have_all_stops() {
        let hues = [
            Palette::SLATE,
            Palette::GRAY,
            Palette::ZINC,
            Palette::NEUTRAL,
            Palette::STONE,
            Palette::RED,
            Palette::ORANGE,
            Palette::AMBER,
            Palette::YELLOW,
            Palette::LIME,
            Palette::GREEN,
            Palette::EMERALD,
            Palette::TEAL,
            Palette::CYAN,
            Palette::SKY,
            Palette::BLUE,
            Palette::INDIGO,
            Palette::VIOLET,
            Palette::PURPLE,
            Palette::FUCHSIA,
            Palette::PINK,
            Palette::ROSE,
        ];
        for hue in &hues {
            for &stop in &STOPS {
                assert!(hue.get(stop).is_some(), "missing stop {stop}");
            }
        }
    }

    #[test]
    fn all_22_hues_present() {
        // Verify we have exactly 22 hues by constructing the array
        let hues = [
            Palette::SLATE,
            Palette::GRAY,
            Palette::ZINC,
            Palette::NEUTRAL,
            Palette::STONE,
            Palette::RED,
            Palette::ORANGE,
            Palette::AMBER,
            Palette::YELLOW,
            Palette::LIME,
            Palette::GREEN,
            Palette::EMERALD,
            Palette::TEAL,
            Palette::CYAN,
            Palette::SKY,
            Palette::BLUE,
            Palette::INDIGO,
            Palette::VIOLET,
            Palette::PURPLE,
            Palette::FUCHSIA,
            Palette::PINK,
            Palette::ROSE,
        ];
        assert_eq!(hues.len(), 22);
    }

    #[test]
    fn stops_are_ordered_light_to_dark() {
        // For any hue, lower stops should be lighter (higher luminance rough proxy: r+g+b)
        let c50 = Palette::BLUE.get(50).unwrap_or(Color::BLACK);
        let c900 = Palette::BLUE.get(900).unwrap_or(Color::WHITE);
        let lum_50 = c50.r + c50.g + c50.b;
        let lum_900 = c900.r + c900.g + c900.b;
        assert!(lum_50 > lum_900, "50 should be lighter than 900");
    }
}
