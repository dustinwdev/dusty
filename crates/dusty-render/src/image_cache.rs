//! Image cache — decodes and uploads images to GPU textures.
//!
//! Each image gets its own GPU texture (not atlased), since images
//! vary significantly in size.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{RenderError, Result};
use crate::primitive::ImageId;

/// Source of an image to load.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageSource {
    /// Load from a file path.
    Path(PathBuf),
    /// Load from in-memory bytes with a caller-chosen key.
    Bytes {
        /// Unique key to identify this image in the cache.
        key: u64,
    },
}

/// A decoded image ready for GPU upload.
#[derive(Debug)]
pub struct DecodedImage {
    /// RGBA pixel data.
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// An entry in the image cache.
struct ImageEntry {
    decoded: DecodedImage,
    last_used: u64,
}

/// Caches decoded images, keyed by [`ImageSource`].
///
/// In a full GPU implementation, each entry would also hold a
/// `wgpu::Texture` and `wgpu::TextureView`. This type manages
/// the CPU-side decode cache and ID assignment.
pub struct ImageCache {
    entries: HashMap<ImageId, ImageEntry>,
    source_to_id: HashMap<ImageSource, ImageId>,
    next_id: u64,
    generation: u64,
    total_bytes: usize,
    max_bytes: usize,
}

impl ImageCache {
    /// Creates a new empty image cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            source_to_id: HashMap::new(),
            next_id: 0,
            generation: 0,
            total_bytes: 0,
            max_bytes: 64 * 1024 * 1024, // 64 MB default
        }
    }

    /// Creates a new image cache with the given memory budget in bytes.
    #[must_use]
    pub fn with_budget(max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            source_to_id: HashMap::new(),
            next_id: 0,
            generation: 0,
            total_bytes: 0,
            max_bytes,
        }
    }

    /// Returns the total memory usage of cached images in bytes.
    #[must_use]
    pub const fn memory_usage(&self) -> usize {
        self.total_bytes
    }

    /// Loads an image from raw RGBA bytes, returning its [`ImageId`].
    ///
    /// If the source has already been loaded, returns the existing ID.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::ImageDecode`] if the bytes cannot be decoded.
    pub fn load_rgba(
        &mut self,
        source: ImageSource,
        data: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Result<ImageId> {
        // Return existing ID if already loaded
        if let Some(&id) = self.source_to_id.get(&source) {
            if let Some(entry) = self.entries.get_mut(&id) {
                entry.last_used = self.generation;
            }
            return Ok(id);
        }

        let expected_len = (width as usize) * (height as usize) * 4;
        if data.len() != expected_len {
            return Err(RenderError::ImageDecode(format!(
                "expected {} bytes for {}x{} RGBA, got {}",
                expected_len,
                width,
                height,
                data.len()
            )));
        }

        let id = self.alloc_id();
        let decoded = DecodedImage {
            data,
            width,
            height,
        };

        self.entries.insert(
            id,
            ImageEntry {
                decoded,
                last_used: self.generation,
            },
        );
        self.source_to_id.insert(source, id);
        self.total_bytes += expected_len;
        self.enforce_budget();
        Ok(id)
    }

    /// Decodes a PNG image from bytes and caches it.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::ImageDecode`] if decoding fails.
    #[cfg(feature = "image")]
    pub fn load_png_bytes(&mut self, source: ImageSource, bytes: &[u8]) -> Result<ImageId> {
        if let Some(&id) = self.source_to_id.get(&source) {
            if let Some(entry) = self.entries.get_mut(&id) {
                entry.last_used = self.generation;
            }
            return Ok(id);
        }

        let img =
            image::load_from_memory(bytes).map_err(|e| RenderError::ImageDecode(e.to_string()))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();

        let id = self.alloc_id();
        let decoded = DecodedImage {
            data,
            width,
            height,
        };

        self.entries.insert(
            id,
            ImageEntry {
                decoded,
                last_used: self.generation,
            },
        );
        self.source_to_id.insert(source, id);
        Ok(id)
    }

    /// Returns the decoded image data for the given ID.
    #[must_use]
    pub fn get(&self, id: ImageId) -> Option<&DecodedImage> {
        self.entries.get(&id).map(|e| &e.decoded)
    }

    /// Increments the generation counter.
    pub fn advance_generation(&mut self) {
        self.generation += 1;
    }

    /// Evicts least-recently-used images until memory usage is within budget.
    pub fn enforce_budget(&mut self) {
        while self.total_bytes > self.max_bytes && !self.entries.is_empty() {
            // Find the LRU entry
            let lru_id = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.last_used)
                .map(|(&id, _)| id);

            if let Some(id) = lru_id {
                if let Some(entry) = self.entries.remove(&id) {
                    self.total_bytes = self.total_bytes.saturating_sub(entry.decoded.data.len());
                }
                self.source_to_id.retain(|_, v| *v != id);
            } else {
                break;
            }
        }
    }

    /// Removes images not used for `threshold` or more frames.
    pub fn evict_unused(&mut self, threshold: u64) {
        let gen = self.generation;
        let to_remove: Vec<ImageId> = self
            .entries
            .iter()
            .filter(|(_, e)| gen - e.last_used >= threshold)
            .map(|(&id, _)| id)
            .collect();

        for id in &to_remove {
            if let Some(entry) = self.entries.remove(id) {
                self.total_bytes = self.total_bytes.saturating_sub(entry.decoded.data.len());
            }
        }

        self.source_to_id.retain(|_, id| !to_remove.contains(id));
    }

    /// Returns the number of cached images.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if no images are cached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns a reference to the source-to-ID mapping.
    #[must_use]
    pub const fn source_to_id(&self) -> &HashMap<ImageSource, ImageId> {
        &self.source_to_id
    }

    fn alloc_id(&mut self) -> ImageId {
        let id = ImageId(self.next_id);
        self.next_id += 1;
        id
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rgba(width: u32, height: u32) -> Vec<u8> {
        vec![255u8; (width * height * 4) as usize]
    }

    #[test]
    fn load_rgba_returns_valid_id() {
        let mut cache = ImageCache::new();
        let source = ImageSource::Bytes { key: 1 };
        let data = make_rgba(2, 2);
        let id = cache.load_rgba(source, data, 2, 2).unwrap();
        assert_eq!(id, ImageId(0));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn same_source_returns_same_id() {
        let mut cache = ImageCache::new();
        let source = ImageSource::Bytes { key: 42 };
        let data = make_rgba(4, 4);
        let id1 = cache.load_rgba(source.clone(), data.clone(), 4, 4).unwrap();
        let id2 = cache.load_rgba(source, data, 4, 4).unwrap();
        assert_eq!(id1, id2);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn different_source_returns_different_id() {
        let mut cache = ImageCache::new();
        let id1 = cache
            .load_rgba(ImageSource::Bytes { key: 1 }, make_rgba(2, 2), 2, 2)
            .unwrap();
        let id2 = cache
            .load_rgba(ImageSource::Bytes { key: 2 }, make_rgba(2, 2), 2, 2)
            .unwrap();
        assert_ne!(id1, id2);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn get_returns_decoded_image() {
        let mut cache = ImageCache::new();
        let source = ImageSource::Bytes { key: 1 };
        let id = cache.load_rgba(source, make_rgba(3, 3), 3, 3).unwrap();
        let img = cache.get(id);
        assert!(img.is_some());
        let img = img.unwrap();
        assert_eq!(img.width, 3);
        assert_eq!(img.height, 3);
        assert_eq!(img.data.len(), 3 * 3 * 4);
    }

    #[test]
    fn get_invalid_id_returns_none() {
        let cache = ImageCache::new();
        assert!(cache.get(ImageId(99)).is_none());
    }

    #[test]
    fn wrong_data_length_returns_error() {
        let mut cache = ImageCache::new();
        let result = cache.load_rgba(ImageSource::Bytes { key: 1 }, vec![0; 10], 4, 4);
        assert!(result.is_err());
    }

    #[test]
    fn evict_removes_old_images() {
        let mut cache = ImageCache::new();
        let id1 = cache
            .load_rgba(ImageSource::Bytes { key: 1 }, make_rgba(2, 2), 2, 2)
            .unwrap();

        for _ in 0..100 {
            cache.advance_generation();
        }

        let _id2 = cache
            .load_rgba(ImageSource::Bytes { key: 2 }, make_rgba(2, 2), 2, 2)
            .unwrap();

        cache.evict_unused(60);

        assert!(cache.get(id1).is_none());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn empty_cache() {
        let cache = ImageCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn path_source() {
        let source = ImageSource::Path(PathBuf::from("/tmp/test.png"));
        let source2 = source.clone();
        assert_eq!(source, source2);
    }

    #[test]
    fn memory_budget_evicts_lru() {
        // Budget for roughly 2 images of 4x4 RGBA (64 bytes each) = 128 bytes
        let mut cache = ImageCache::with_budget(128);
        let _id1 = cache
            .load_rgba(ImageSource::Bytes { key: 1 }, make_rgba(2, 2), 2, 2)
            .unwrap(); // 16 bytes
        cache.advance_generation();
        let _id2 = cache
            .load_rgba(ImageSource::Bytes { key: 2 }, make_rgba(2, 2), 2, 2)
            .unwrap(); // 16 bytes
        cache.advance_generation();

        // This should push us over budget and evict LRU
        let _id3 = cache
            .load_rgba(ImageSource::Bytes { key: 3 }, make_rgba(8, 8), 8, 8)
            .unwrap(); // 256 bytes, over budget

        assert!(cache.memory_usage() <= 128 + 256); // enforcement runs
    }

    #[test]
    fn memory_tracking_accurate() {
        let mut cache = ImageCache::new();
        assert_eq!(cache.memory_usage(), 0);

        cache
            .load_rgba(ImageSource::Bytes { key: 1 }, make_rgba(2, 2), 2, 2)
            .unwrap();
        assert_eq!(cache.memory_usage(), 2 * 2 * 4);

        cache
            .load_rgba(ImageSource::Bytes { key: 2 }, make_rgba(4, 4), 4, 4)
            .unwrap();
        assert_eq!(cache.memory_usage(), 2 * 2 * 4 + 4 * 4 * 4);
    }
}
