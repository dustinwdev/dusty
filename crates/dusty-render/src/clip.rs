//! Clip stack for managing nested clip regions.

use crate::primitive::{ClipRegion, Rect};

/// A stack of clip regions supporting push/pop and intersection.
///
/// When empty, no clipping is applied (full viewport is visible).
/// Each push intersects the new clip with the current effective clip.
///
/// # Examples
///
/// ```
/// use dusty_render::{ClipStack, ClipRegion, Rect};
///
/// let mut stack = ClipStack::new();
/// assert!(stack.current().is_none());
///
/// stack.push(ClipRegion {
///     rect: Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 },
///     radii: [0.0; 4],
/// });
/// assert!(stack.current().is_some());
/// ```
#[derive(Debug)]
pub struct ClipStack {
    stack: Vec<ClipRegion>,
}

impl ClipStack {
    /// Creates an empty clip stack (no clipping).
    #[must_use]
    pub const fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Returns the current effective clip region, or `None` if no clip is active.
    #[must_use]
    pub fn current(&self) -> Option<&ClipRegion> {
        self.stack.last()
    }

    /// Pushes a new clip region. The effective clip is the intersection
    /// of the new region with the current top.
    pub fn push(&mut self, region: ClipRegion) {
        let effective = self.stack.last().map_or(region, |current| {
            // Intersect with current clip
            let rect = current.rect.intersect(&region.rect).unwrap_or(Rect {
                x: region.rect.x,
                y: region.rect.y,
                width: 0.0,
                height: 0.0,
            });
            // Use the tighter radii (the new region's radii when it's a
            // sub-clip, since the parent clip already constrains the shape).
            // If the new region has all-zero radii, inherit from parent.
            let has_radii = region.radii.iter().any(|&r| r.abs() > f32::EPSILON);
            let radii = if has_radii {
                region.radii
            } else {
                current.radii
            };
            ClipRegion { rect, radii }
        });
        self.stack.push(effective);
    }

    /// Pops the most recent clip region, restoring the previous one.
    ///
    /// Returns `None` if the stack was empty.
    pub fn pop(&mut self) -> Option<ClipRegion> {
        self.stack.pop()
    }

    /// Returns `true` if no clip regions are active.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Returns the number of clip regions on the stack.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Clears all clip regions.
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

impl Default for ClipStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_clip() -> ClipRegion {
        ClipRegion {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 1000.0,
                height: 1000.0,
            },
            radii: [0.0; 4],
        }
    }

    #[test]
    fn empty_stack_has_no_clip() {
        let stack = ClipStack::new();
        assert!(stack.current().is_none());
        assert!(stack.is_empty());
        assert_eq!(stack.depth(), 0);
    }

    #[test]
    fn single_push_sets_clip() {
        let mut stack = ClipStack::new();
        let clip = ClipRegion {
            rect: Rect {
                x: 10.0,
                y: 20.0,
                width: 80.0,
                height: 60.0,
            },
            radii: [4.0; 4],
        };
        stack.push(clip);
        assert!(!stack.is_empty());
        assert_eq!(stack.depth(), 1);

        let current = stack.current();
        assert!(current.is_some());
        let c = current.unwrap();
        assert_eq!(c.rect.x, 10.0);
        assert_eq!(c.rect.width, 80.0);
        assert_eq!(c.radii, [4.0; 4]);
    }

    #[test]
    fn nested_clips_intersect() {
        let mut stack = ClipStack::new();

        // First clip: 0,0 -> 100,100
        stack.push(ClipRegion {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            radii: [0.0; 4],
        });

        // Second clip: 50,50 -> 150,150, but intersected = 50,50 -> 100,100
        stack.push(ClipRegion {
            rect: Rect {
                x: 50.0,
                y: 50.0,
                width: 100.0,
                height: 100.0,
            },
            radii: [0.0; 4],
        });

        let current = stack.current().unwrap();
        assert_eq!(current.rect.x, 50.0);
        assert_eq!(current.rect.y, 50.0);
        assert_eq!(current.rect.width, 50.0);
        assert_eq!(current.rect.height, 50.0);
    }

    #[test]
    fn pop_restores_previous() {
        let mut stack = ClipStack::new();

        let clip1 = ClipRegion {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
            },
            radii: [0.0; 4],
        };
        stack.push(clip1);

        stack.push(ClipRegion {
            rect: Rect {
                x: 50.0,
                y: 50.0,
                width: 100.0,
                height: 100.0,
            },
            radii: [0.0; 4],
        });

        assert_eq!(stack.depth(), 2);

        // Pop the nested clip
        stack.pop();
        assert_eq!(stack.depth(), 1);

        // Should be back to the first clip
        let current = stack.current().unwrap();
        assert_eq!(current.rect.width, 200.0);
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut stack = ClipStack::new();
        assert!(stack.pop().is_none());
    }

    #[test]
    fn pop_last_returns_to_no_clip() {
        let mut stack = ClipStack::new();
        stack.push(full_clip());
        stack.pop();
        assert!(stack.current().is_none());
        assert!(stack.is_empty());
    }

    #[test]
    fn disjoint_clips_produce_zero_area() {
        let mut stack = ClipStack::new();
        stack.push(ClipRegion {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
            },
            radii: [0.0; 4],
        });
        stack.push(ClipRegion {
            rect: Rect {
                x: 100.0,
                y: 100.0,
                width: 50.0,
                height: 50.0,
            },
            radii: [0.0; 4],
        });

        let current = stack.current().unwrap();
        assert_eq!(current.rect.width, 0.0);
        assert_eq!(current.rect.height, 0.0);
    }

    #[test]
    fn nested_radii_uses_new_when_nonzero() {
        let mut stack = ClipStack::new();
        stack.push(ClipRegion {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
            },
            radii: [8.0; 4],
        });
        stack.push(ClipRegion {
            rect: Rect {
                x: 10.0,
                y: 10.0,
                width: 100.0,
                height: 100.0,
            },
            radii: [4.0; 4],
        });

        let current = stack.current().unwrap();
        assert_eq!(current.radii, [4.0; 4]);
    }

    #[test]
    fn nested_radii_inherits_when_zero() {
        let mut stack = ClipStack::new();
        stack.push(ClipRegion {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
            },
            radii: [8.0; 4],
        });
        stack.push(ClipRegion {
            rect: Rect {
                x: 10.0,
                y: 10.0,
                width: 100.0,
                height: 100.0,
            },
            radii: [0.0; 4],
        });

        let current = stack.current().unwrap();
        assert_eq!(current.radii, [8.0; 4]);
    }

    #[test]
    fn clear_resets_stack() {
        let mut stack = ClipStack::new();
        stack.push(full_clip());
        stack.push(full_clip());
        assert_eq!(stack.depth(), 2);

        stack.clear();
        assert!(stack.is_empty());
        assert!(stack.current().is_none());
    }

    #[test]
    fn default_creates_empty_stack() {
        let stack = ClipStack::default();
        assert!(stack.is_empty());
    }
}
