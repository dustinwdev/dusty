//! Shelf-based texture atlas allocator.
//!
//! Pure data structure — no GPU dependency. Manages a 2D texture atlas
//! by packing allocations into horizontal shelves.
//!
//! # Examples
//!
//! ```
//! use dusty_render::atlas::ShelfAllocator;
//!
//! let mut atlas = ShelfAllocator::new(512, 512);
//! let region = atlas.allocate(32, 32);
//! assert!(region.is_some());
//! ```

/// An allocated region within the atlas, in texels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasRegion {
    /// X offset in texels.
    pub x: u32,
    /// Y offset in texels.
    pub y: u32,
    /// Width in texels.
    pub width: u32,
    /// Height in texels.
    pub height: u32,
}

/// A horizontal shelf in the atlas.
#[derive(Debug)]
struct Shelf {
    /// Y position of this shelf's top edge.
    y: u32,
    /// Height of this shelf (tallest item placed so far).
    height: u32,
    /// Next free X position on this shelf.
    next_x: u32,
}

/// Shelf-based 2D texture atlas allocator.
///
/// Packs rectangles into horizontal rows (shelves). When a rectangle
/// doesn't fit on any existing shelf, a new shelf is created below.
///
/// Does **not** interact with the GPU — it only tracks allocations.
#[derive(Debug)]
pub struct ShelfAllocator {
    width: u32,
    height: u32,
    shelves: Vec<Shelf>,
    /// Y position for the next shelf to be created.
    next_y: u32,
}

impl ShelfAllocator {
    /// Creates a new allocator with the given dimensions.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            shelves: Vec::new(),
            next_y: 0,
        }
    }

    /// Attempts to allocate a region of `w × h` texels.
    ///
    /// Returns `None` if the atlas is full and cannot fit the requested size.
    #[allow(clippy::similar_names)]
    pub fn allocate(&mut self, w: u32, h: u32) -> Option<AtlasRegion> {
        if w == 0 || h == 0 || w > self.width || h > self.height {
            return None;
        }

        // Try existing shelves (best-fit: smallest shelf height that fits).
        let mut chosen_shelf: Option<usize> = None;
        let mut min_waste = u32::MAX;

        for (i, shelf) in self.shelves.iter().enumerate() {
            if shelf.height >= h && (self.width - shelf.next_x) >= w {
                let waste = shelf.height - h;
                if waste < min_waste {
                    min_waste = waste;
                    chosen_shelf = Some(i);
                }
            }
        }

        if let Some(idx) = chosen_shelf {
            let shelf = &mut self.shelves[idx];
            let region = AtlasRegion {
                x: shelf.next_x,
                y: shelf.y,
                width: w,
                height: h,
            };
            shelf.next_x += w;
            return Some(region);
        }

        // Create a new shelf if there's vertical space.
        if self.next_y + h <= self.height {
            let region = AtlasRegion {
                x: 0,
                y: self.next_y,
                width: w,
                height: h,
            };
            self.shelves.push(Shelf {
                y: self.next_y,
                height: h,
                next_x: w,
            });
            self.next_y += h;
            Some(region)
        } else {
            None
        }
    }

    /// Resets all allocations. The atlas dimensions remain unchanged.
    pub fn clear(&mut self) {
        self.shelves.clear();
        self.next_y = 0;
    }

    /// Returns the fraction of atlas area currently allocated (0.0–1.0).
    #[must_use]
    pub fn utilization(&self) -> f32 {
        let total = f64::from(self.width) * f64::from(self.height);
        if total == 0.0 {
            return 0.0;
        }
        let used: f64 = self
            .shelves
            .iter()
            .map(|s| f64::from(s.next_x) * f64::from(s.height))
            .sum();
        #[allow(clippy::cast_possible_truncation)]
        let ratio = (used / total) as f32;
        ratio
    }

    /// Returns the atlas dimensions `(width, height)`.
    #[must_use]
    pub const fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_glyph_allocates_at_origin() {
        let mut atlas = ShelfAllocator::new(512, 512);
        let region = atlas.allocate(16, 16);
        assert_eq!(
            region,
            Some(AtlasRegion {
                x: 0,
                y: 0,
                width: 16,
                height: 16
            })
        );
    }

    #[test]
    fn same_height_glyphs_pack_horizontally() {
        let mut atlas = ShelfAllocator::new(512, 512);
        let a = atlas.allocate(16, 16).unwrap();
        let b = atlas.allocate(20, 16).unwrap();
        let c = atlas.allocate(10, 16).unwrap();

        // All on the same shelf (y=0)
        assert_eq!(a.y, 0);
        assert_eq!(b.y, 0);
        assert_eq!(c.y, 0);

        // Packed left-to-right
        assert_eq!(a.x, 0);
        assert_eq!(b.x, 16);
        assert_eq!(c.x, 36);
    }

    #[test]
    fn different_height_glyphs_create_new_shelves() {
        let mut atlas = ShelfAllocator::new(512, 512);
        let a = atlas.allocate(16, 16).unwrap();
        let b = atlas.allocate(16, 32).unwrap();

        // Different heights → different shelves
        assert_eq!(a.y, 0);
        assert_eq!(b.y, 16);
    }

    #[test]
    fn full_atlas_returns_none() {
        let mut atlas = ShelfAllocator::new(32, 32);
        // Fill the atlas
        let _ = atlas.allocate(32, 32);
        // No room left
        assert!(atlas.allocate(1, 1).is_none());
    }

    #[test]
    fn clear_resets_everything() {
        let mut atlas = ShelfAllocator::new(64, 64);
        let _ = atlas.allocate(32, 32);
        let _ = atlas.allocate(32, 32);
        assert!(atlas.utilization() > 0.0);

        atlas.clear();
        assert_eq!(atlas.utilization(), 0.0);

        // Can allocate again at origin
        let region = atlas.allocate(16, 16);
        assert_eq!(
            region,
            Some(AtlasRegion {
                x: 0,
                y: 0,
                width: 16,
                height: 16
            })
        );
    }

    #[test]
    fn zero_size_allocation_returns_none() {
        let mut atlas = ShelfAllocator::new(512, 512);
        assert!(atlas.allocate(0, 16).is_none());
        assert!(atlas.allocate(16, 0).is_none());
        assert!(atlas.allocate(0, 0).is_none());
    }

    #[test]
    fn too_large_allocation_returns_none() {
        let mut atlas = ShelfAllocator::new(64, 64);
        assert!(atlas.allocate(65, 16).is_none());
        assert!(atlas.allocate(16, 65).is_none());
    }

    #[test]
    fn utilization_increases_with_allocations() {
        let mut atlas = ShelfAllocator::new(100, 100);
        assert_eq!(atlas.utilization(), 0.0);

        let _ = atlas.allocate(50, 50);
        let u = atlas.utilization();
        assert!(u > 0.0, "utilization should be positive: {u}");
        assert!(u <= 1.0);
    }

    #[test]
    fn size_returns_dimensions() {
        let atlas = ShelfAllocator::new(256, 128);
        assert_eq!(atlas.size(), (256, 128));
    }

    #[test]
    fn shelf_reuse_best_fit() {
        let mut atlas = ShelfAllocator::new(512, 512);
        // Create a tall shelf — fill it horizontally
        let _ = atlas.allocate(512, 32);
        // Create a short shelf
        let _ = atlas.allocate(100, 16);

        // A 14px glyph should fit in the 16px shelf (less waste than 32px shelf)
        let region = atlas.allocate(50, 14).unwrap();
        assert_eq!(region.y, 32); // on the 16px shelf (y=32 because 32px shelf is above)
    }

    #[test]
    fn horizontal_overflow_creates_new_shelf() {
        let mut atlas = ShelfAllocator::new(64, 128);
        // Fill first shelf horizontally
        let _ = atlas.allocate(32, 16);
        let _ = atlas.allocate(32, 16);
        // No horizontal room → new shelf
        let c = atlas.allocate(16, 16).unwrap();
        assert_eq!(c.y, 16);
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn allocations_never_overlap(
            sizes in proptest::collection::vec((1u32..64, 1u32..64), 1..50)
        ) {
            let mut atlas = ShelfAllocator::new(512, 512);
            let mut regions = Vec::new();

            for (w, h) in sizes {
                if let Some(region) = atlas.allocate(w, h) {
                    // Check no overlap with existing regions
                    for existing in &regions {
                        assert!(
                            !rects_overlap(&region, existing),
                            "overlap: {region:?} vs {existing:?}"
                        );
                    }
                    // Check within bounds
                    assert!(region.x + region.width <= 512);
                    assert!(region.y + region.height <= 512);
                    regions.push(region);
                }
            }
        }

        #[test]
        fn utilization_in_valid_range(
            sizes in proptest::collection::vec((1u32..32, 1u32..32), 0..20)
        ) {
            let mut atlas = ShelfAllocator::new(256, 256);
            for (w, h) in sizes {
                let _ = atlas.allocate(w, h);
            }
            let u = atlas.utilization();
            assert!((0.0..=1.0).contains(&u), "utilization out of range: {u}");
        }
    }

    fn rects_overlap(a: &AtlasRegion, b: &AtlasRegion) -> bool {
        a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
    }
}
