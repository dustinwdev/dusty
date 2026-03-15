//! Glyph rasterization — bridges cosmic-text's `SwashCache` to pixel data.
//!
//! Provides [`GlyphRasterizer`] for rasterizing individual glyphs into
//! alpha bitmaps suitable for uploading to a GPU texture atlas.
//!
//! # Examples
//!
//! ```
//! use dusty_text::rasterize::GlyphRasterizer;
//!
//! let rasterizer = GlyphRasterizer::new();
//! ```

use cosmic_text::{CacheKey, FontSystem, SwashCache, SwashContent};

/// A rasterized glyph bitmap.
#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    /// Pixel data. For alpha-mask glyphs, one byte per pixel (coverage).
    /// For color emoji, RGBA (4 bytes per pixel).
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Left bearing (horizontal offset from the glyph origin).
    pub left: i32,
    /// Top bearing (vertical offset from the baseline).
    pub top: i32,
    /// Whether this glyph contains color data (e.g. emoji).
    pub is_color: bool,
}

/// Rasterizes glyphs using cosmic-text's Swash integration.
pub struct GlyphRasterizer {
    swash_cache: SwashCache,
}

impl GlyphRasterizer {
    /// Creates a new glyph rasterizer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            swash_cache: SwashCache::new(),
        }
    }

    /// Rasterizes a single glyph identified by its `CacheKey`.
    ///
    /// Returns `None` if the glyph cannot be rasterized (e.g. space characters,
    /// missing glyphs, or rasterization failure).
    pub fn rasterize(
        &mut self,
        font_system: &mut FontSystem,
        cache_key: CacheKey,
    ) -> Option<RasterizedGlyph> {
        let image = self
            .swash_cache
            .get_image_uncached(font_system, cache_key)?;

        let is_color = matches!(image.content, SwashContent::Color);

        if image.placement.width == 0 || image.placement.height == 0 {
            return None;
        }

        // For alpha-mask content, cosmic-text returns one byte per pixel (coverage).
        // For color content, it returns RGBA (4 bytes per pixel).
        // SubpixelMask also has special handling but we treat it as alpha for now.
        let data = match image.content {
            SwashContent::Mask | SwashContent::Color => image.data.clone(),
            SwashContent::SubpixelMask => {
                // Subpixel data has 3 bytes per pixel (RGB). Extract luma as alpha.
                image
                    .data
                    .chunks_exact(3)
                    .map(|rgb| {
                        // Simple luma: (r + g + b) / 3
                        let sum = u16::from(rgb[0]) + u16::from(rgb[1]) + u16::from(rgb[2]);
                        #[allow(clippy::cast_possible_truncation)]
                        let luma = (sum / 3) as u8;
                        luma
                    })
                    .collect()
            }
        };

        Some(RasterizedGlyph {
            data,
            width: image.placement.width,
            height: image.placement.height,
            left: image.placement.left,
            top: image.placement.top,
            is_color,
        })
    }
}

impl Default for GlyphRasterizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TextSystem;
    use cosmic_text::{Buffer, Metrics, Shaping, Wrap};

    /// Helper: get a `CacheKey` by shaping a known character.
    fn get_cache_key_for_char(font_system: &mut FontSystem, ch: char) -> Option<CacheKey> {
        let metrics = Metrics::new(16.0, 20.0);
        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_wrap(font_system, Wrap::None);
        buffer.set_size(font_system, None, None);
        buffer.set_text(
            font_system,
            &ch.to_string(),
            &cosmic_text::Attrs::new(),
            Shaping::Advanced,
            None,
        );

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((0., 0.), 1.0);
                return Some(physical.cache_key);
            }
        }
        None
    }

    #[test]
    fn rasterizer_creates_successfully() {
        let _rasterizer = GlyphRasterizer::new();
    }

    #[test]
    fn rasterizer_default_works() {
        let _rasterizer = GlyphRasterizer::default();
    }

    #[test]
    fn rasterize_known_glyph_has_positive_dimensions() {
        let system = TextSystem::new();
        let mut font_system = system.font_system_mut().unwrap();
        let mut rasterizer = GlyphRasterizer::new();

        let key = get_cache_key_for_char(&mut font_system, 'A');
        assert!(key.is_some(), "should get cache key for 'A'");

        let glyph = rasterizer.rasterize(&mut font_system, key.unwrap());
        assert!(glyph.is_some(), "should rasterize 'A'");

        let g = glyph.unwrap();
        assert!(g.width > 0, "width should be positive: {}", g.width);
        assert!(g.height > 0, "height should be positive: {}", g.height);
        assert!(!g.data.is_empty(), "bitmap data should be non-empty");
    }

    #[test]
    fn rasterize_space_returns_none() {
        let system = TextSystem::new();
        let mut font_system = system.font_system_mut().unwrap();
        let mut rasterizer = GlyphRasterizer::new();

        // Space glyphs have zero-size bitmaps.
        let key = get_cache_key_for_char(&mut font_system, ' ');
        if let Some(k) = key {
            let glyph = rasterizer.rasterize(&mut font_system, k);
            // Spaces typically return None (zero-width bitmap)
            assert!(
                glyph.is_none(),
                "space should not produce a rasterized glyph"
            );
        }
    }

    #[test]
    fn rasterized_glyph_data_length_matches_dimensions() {
        let system = TextSystem::new();
        let mut font_system = system.font_system_mut().unwrap();
        let mut rasterizer = GlyphRasterizer::new();

        let key = get_cache_key_for_char(&mut font_system, 'M');
        if let Some(k) = key {
            if let Some(g) = rasterizer.rasterize(&mut font_system, k) {
                let expected_len = if g.is_color {
                    (g.width * g.height * 4) as usize // RGBA
                } else {
                    (g.width * g.height) as usize // alpha mask
                };
                assert_eq!(
                    g.data.len(),
                    expected_len,
                    "data length should match w*h (or w*h*4 for color)"
                );
            }
        }
    }

    #[test]
    fn rasterize_multiple_glyphs() {
        let system = TextSystem::new();
        let mut font_system = system.font_system_mut().unwrap();
        let mut rasterizer = GlyphRasterizer::new();

        for ch in ['H', 'e', 'l', 'o'] {
            let key = get_cache_key_for_char(&mut font_system, ch);
            assert!(key.is_some(), "should get cache key for '{ch}'");
            let glyph = rasterizer.rasterize(&mut font_system, key.unwrap());
            assert!(glyph.is_some(), "should rasterize '{ch}'");
        }
    }
}
