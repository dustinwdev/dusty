//! Glyph cache — atlas management and glyph lookup.

use std::collections::HashMap;

use dusty_text::{CacheKey, GlyphRasterizer, RasterizedGlyph};

use crate::atlas::{AtlasRegion, ShelfAllocator};

/// A cached glyph with atlas coordinates and metadata.
#[derive(Debug, Clone, Copy)]
pub struct CachedGlyph {
    /// UV coordinates in the atlas `[u_min, v_min, u_max, v_max]`.
    pub uv: [f32; 4],
    /// Bearing offset `[left, top]` in pixels.
    pub offset: [i32; 2],
    /// Glyph bitmap size `[width, height]` in pixels.
    pub size: [u32; 2],
    /// Frame generation when this glyph was last used.
    pub last_used: u64,
}

/// Caches rasterized glyphs in a texture atlas.
///
/// Uses a [`ShelfAllocator`] for atlas packing and a `HashMap` keyed by
/// cosmic-text's `CacheKey` for fast lookup.
pub struct GlyphCache {
    atlas: ShelfAllocator,
    cache: HashMap<CacheKey, CachedGlyph>,
    generation: u64,
    staging_buffer: Vec<u8>,
    dirty: bool,
    max_entries: usize,
}

impl GlyphCache {
    /// Creates a new glyph cache with the given initial atlas dimensions.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let staging_buffer = vec![0u8; (width * height) as usize];
        Self {
            atlas: ShelfAllocator::new(width, height),
            cache: HashMap::new(),
            generation: 0,
            staging_buffer,
            dirty: false,
            max_entries: 4096,
        }
    }

    /// Creates a new glyph cache with the given atlas dimensions and max entry count.
    ///
    /// When the cache exceeds `max_entries`, an eviction pass is triggered
    /// automatically on the next insert.
    #[must_use]
    pub fn with_capacity(width: u32, height: u32, max_entries: usize) -> Self {
        let staging_buffer = vec![0u8; (width * height) as usize];
        Self {
            atlas: ShelfAllocator::new(width, height),
            cache: HashMap::new(),
            generation: 0,
            staging_buffer,
            dirty: false,
            max_entries,
        }
    }

    /// Looks up a glyph in the cache, rasterizing on miss.
    ///
    /// Returns `None` if the glyph cannot be rasterized (e.g. space)
    /// or the atlas is full.
    pub fn get_or_rasterize(
        &mut self,
        key: CacheKey,
        rasterizer: &mut GlyphRasterizer,
        font_system: &mut cosmic_text::FontSystem,
    ) -> Option<CachedGlyph> {
        // Cache hit
        if let Some(cached) = self.cache.get_mut(&key) {
            cached.last_used = self.generation;
            return Some(*cached);
        }

        // Cache miss — rasterize
        let glyph = rasterizer.rasterize(font_system, key)?;

        // Evict if over capacity
        if self.cache.len() >= self.max_entries {
            // Evict glyphs unused for 30+ frames
            self.evict_unused(30);
        }

        self.insert_glyph(key, &glyph)
    }

    /// Inserts a rasterized glyph into the atlas.
    fn insert_glyph(&mut self, key: CacheKey, glyph: &RasterizedGlyph) -> Option<CachedGlyph> {
        let region = self.atlas.allocate(glyph.width, glyph.height)?;

        // Copy bitmap into staging buffer
        self.blit_to_staging(&region, &glyph.data, glyph.width, glyph.is_color);
        self.dirty = true;

        let (atlas_w, atlas_h) = self.atlas.size();
        #[allow(clippy::cast_precision_loss)]
        let uv = [
            region.x as f32 / atlas_w as f32,
            region.y as f32 / atlas_h as f32,
            (region.x + region.width) as f32 / atlas_w as f32,
            (region.y + region.height) as f32 / atlas_h as f32,
        ];

        let cached = CachedGlyph {
            uv,
            offset: [glyph.left, glyph.top],
            size: [glyph.width, glyph.height],
            last_used: self.generation,
        };

        self.cache.insert(key, cached);
        Some(cached)
    }

    /// Copies glyph bitmap data into the staging buffer at the given atlas region.
    fn blit_to_staging(
        &mut self,
        region: &AtlasRegion,
        data: &[u8],
        src_width: u32,
        is_color: bool,
    ) {
        let (atlas_w, _) = self.atlas.size();
        let bytes_per_pixel: u32 = if is_color { 4 } else { 1 };

        for row in 0..region.height {
            let src_start = (row * src_width * bytes_per_pixel) as usize;
            let src_end = src_start + (region.width * bytes_per_pixel) as usize;
            if src_end > data.len() {
                break;
            }

            let dst_y = region.y + row;
            let dst_x = region.x;
            // Staging buffer is 1 byte per pixel (alpha atlas)
            // For color glyphs, we store just the alpha channel for now.
            if is_color {
                for col in 0..region.width {
                    let src_idx = (row * src_width * 4 + col * 4 + 3) as usize; // alpha
                    let dst_idx = (dst_y * atlas_w + dst_x + col) as usize;
                    if src_idx < data.len() && dst_idx < self.staging_buffer.len() {
                        self.staging_buffer[dst_idx] = data[src_idx];
                    }
                }
            } else {
                let dst_start = (dst_y * atlas_w + dst_x) as usize;
                let dst_end = dst_start + region.width as usize;
                if dst_end <= self.staging_buffer.len() {
                    self.staging_buffer[dst_start..dst_end]
                        .copy_from_slice(&data[src_start..src_end]);
                }
            }
        }
    }

    /// Increments the frame generation counter.
    pub fn advance_generation(&mut self) {
        self.generation += 1;
    }

    /// Removes glyphs not used for `threshold` or more frames.
    ///
    /// After eviction the atlas is cleared and surviving glyphs are
    /// **not** repacked — callers should re-rasterize as needed. This
    /// keeps the eviction path simple and fast.
    pub fn evict_unused(&mut self, threshold: u64) {
        let gen = self.generation;
        let before = self.cache.len();
        self.cache.retain(|_, v| gen - v.last_used < threshold);

        if self.cache.len() < before {
            // Atlas topology is invalid after removal, so clear and let
            // future rasterizations re-fill it.
            self.atlas.clear();
            self.staging_buffer.fill(0);
            self.dirty = true;

            // Re-insert is handled lazily on next get_or_rasterize call.
            // We clear the cache entirely since atlas positions are invalid.
            self.cache.clear();
        }
    }

    /// Returns the CPU-side atlas pixel data for GPU upload.
    #[must_use]
    pub fn staging_data(&self) -> &[u8] {
        &self.staging_buffer
    }

    /// Returns the current atlas dimensions `(width, height)`.
    #[must_use]
    pub const fn atlas_size(&self) -> (u32, u32) {
        self.atlas.size()
    }

    /// Returns `true` if the staging buffer has been modified since the
    /// last call to [`mark_clean`](Self::mark_clean).
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the staging buffer as clean (e.g. after GPU upload).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Returns the current frame generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns the number of cached glyphs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns `true` if no glyphs are cached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic_text::{Buffer, Metrics, Shaping, Wrap};
    use dusty_text::TextSystem;

    fn get_cache_key(font_system: &mut cosmic_text::FontSystem, ch: char) -> Option<CacheKey> {
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
    fn cache_miss_triggers_rasterization() {
        let system = TextSystem::new();
        let mut font_system = system.font_system_mut().unwrap();
        let mut rasterizer = GlyphRasterizer::new();
        let mut cache = GlyphCache::new(256, 256);

        let key = get_cache_key(&mut font_system, 'A').unwrap();
        let result = cache.get_or_rasterize(key, &mut rasterizer, &mut font_system);
        assert!(result.is_some(), "should rasterize on cache miss");
        assert_eq!(cache.len(), 1);
        assert!(cache.is_dirty());
    }

    #[test]
    fn cache_hit_returns_same_uv() {
        let system = TextSystem::new();
        let mut font_system = system.font_system_mut().unwrap();
        let mut rasterizer = GlyphRasterizer::new();
        let mut cache = GlyphCache::new(256, 256);

        let key = get_cache_key(&mut font_system, 'B').unwrap();
        let first = cache
            .get_or_rasterize(key, &mut rasterizer, &mut font_system)
            .unwrap();
        let second = cache
            .get_or_rasterize(key, &mut rasterizer, &mut font_system)
            .unwrap();

        assert_eq!(first.uv, second.uv);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn eviction_removes_old_glyphs() {
        let system = TextSystem::new();
        let mut font_system = system.font_system_mut().unwrap();
        let mut rasterizer = GlyphRasterizer::new();
        let mut cache = GlyphCache::new(256, 256);

        // Insert a glyph at generation 0
        let key_a = get_cache_key(&mut font_system, 'X').unwrap();
        let _ = cache.get_or_rasterize(key_a, &mut rasterizer, &mut font_system);

        // Advance many generations
        for _ in 0..100 {
            cache.advance_generation();
        }

        // Insert a recent glyph
        let key_b = get_cache_key(&mut font_system, 'Y').unwrap();
        let _ = cache.get_or_rasterize(key_b, &mut rasterizer, &mut font_system);

        assert_eq!(cache.len(), 2);

        // Evict glyphs unused for 60+ frames
        cache.evict_unused(60);

        // Both are evicted because eviction clears the whole cache
        // (atlas topology becomes invalid)
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn atlas_full_returns_none() {
        let system = TextSystem::new();
        let mut font_system = system.font_system_mut().unwrap();
        let mut rasterizer = GlyphRasterizer::new();
        // Tiny atlas
        let mut cache = GlyphCache::new(4, 4);

        let key = get_cache_key(&mut font_system, 'W').unwrap();
        // 'W' at 16px is likely larger than 4x4
        let result = cache.get_or_rasterize(key, &mut rasterizer, &mut font_system);
        assert!(result.is_none(), "should fail on tiny atlas");
    }

    #[test]
    fn mark_clean_clears_dirty_flag() {
        let mut cache = GlyphCache::new(64, 64);
        assert!(!cache.is_dirty());

        // Manually trigger dirty
        let system = TextSystem::new();
        let mut font_system = system.font_system_mut().unwrap();
        let mut rasterizer = GlyphRasterizer::new();
        let key = get_cache_key(&mut font_system, 'Z').unwrap();
        let _ = cache.get_or_rasterize(key, &mut rasterizer, &mut font_system);
        assert!(cache.is_dirty());

        cache.mark_clean();
        assert!(!cache.is_dirty());
    }

    #[test]
    fn empty_cache() {
        let cache = GlyphCache::new(128, 128);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.generation(), 0);
        assert_eq!(cache.atlas_size(), (128, 128));
    }

    #[test]
    fn max_entries_triggers_eviction() {
        let system = TextSystem::new();
        let mut font_system = system.font_system_mut().unwrap();
        let mut rasterizer = GlyphRasterizer::new();
        let mut cache = GlyphCache::with_capacity(512, 512, 3);

        // Insert 3 glyphs at generation 0
        for ch in ['A', 'B', 'C'] {
            if let Some(key) = get_cache_key(&mut font_system, ch) {
                let _ = cache.get_or_rasterize(key, &mut rasterizer, &mut font_system);
            }
        }
        assert!(cache.len() <= 3);

        // Advance generation so existing entries are "old"
        for _ in 0..40 {
            cache.advance_generation();
        }

        // Insert a 4th glyph — should trigger eviction of old entries
        if let Some(key) = get_cache_key(&mut font_system, 'D') {
            let _ = cache.get_or_rasterize(key, &mut rasterizer, &mut font_system);
        }

        // Cache should have been cleaned up (eviction clears all after retain)
        // The 4th glyph was inserted after eviction cleared the cache
        assert!(cache.len() <= 3);
    }
}
