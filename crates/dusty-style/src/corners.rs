//! Generic four-corner values for border-radius.

/// Four-corner values. `None` means "not set" (inherits or defaults).
///
/// # Examples
///
/// ```
/// use dusty_style::Corners;
///
/// let uniform = Corners::all(4.0);
/// assert_eq!(uniform.top_left, Some(4.0));
/// ```
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Corners<T> {
    /// Top-left corner value.
    pub top_left: Option<T>,
    /// Top-right corner value.
    pub top_right: Option<T>,
    /// Bottom-right corner value.
    pub bottom_right: Option<T>,
    /// Bottom-left corner value.
    pub bottom_left: Option<T>,
}

impl<T: Copy> Corners<T> {
    /// All four corners set to the same value.
    #[must_use]
    pub const fn all(value: T) -> Self {
        Self {
            top_left: Some(value),
            top_right: Some(value),
            bottom_right: Some(value),
            bottom_left: Some(value),
        }
    }

    /// Top corners and bottom corners as two groups.
    #[must_use]
    pub const fn top_bottom(top: T, bottom: T) -> Self {
        Self {
            top_left: Some(top),
            top_right: Some(top),
            bottom_right: Some(bottom),
            bottom_left: Some(bottom),
        }
    }

    /// Each corner specified individually.
    #[must_use]
    pub const fn new(top_left: T, top_right: T, bottom_right: T, bottom_left: T) -> Self {
        Self {
            top_left: Some(top_left),
            top_right: Some(top_right),
            bottom_right: Some(bottom_right),
            bottom_left: Some(bottom_left),
        }
    }

    /// Merges `other` on top of `self`. Other's `Some` values win.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            top_left: other.top_left.or(self.top_left),
            top_right: other.top_right.or(self.top_right),
            bottom_right: other.bottom_right.or(self.bottom_right),
            bottom_left: other.bottom_left.or(self.bottom_left),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_none() {
        let c: Corners<f32> = Corners::default();
        assert_eq!(c.top_left, None);
        assert_eq!(c.top_right, None);
        assert_eq!(c.bottom_right, None);
        assert_eq!(c.bottom_left, None);
    }

    #[test]
    fn all_sets_every_corner() {
        let c = Corners::all(8.0);
        assert_eq!(c.top_left, Some(8.0));
        assert_eq!(c.top_right, Some(8.0));
        assert_eq!(c.bottom_right, Some(8.0));
        assert_eq!(c.bottom_left, Some(8.0));
    }

    #[test]
    fn top_bottom_sets_top_and_bottom() {
        let c = Corners::top_bottom(4.0, 8.0);
        assert_eq!(c.top_left, Some(4.0));
        assert_eq!(c.top_right, Some(4.0));
        assert_eq!(c.bottom_right, Some(8.0));
        assert_eq!(c.bottom_left, Some(8.0));
    }

    #[test]
    fn new_sets_each_corner() {
        let c = Corners::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(c.top_left, Some(1.0));
        assert_eq!(c.top_right, Some(2.0));
        assert_eq!(c.bottom_right, Some(3.0));
        assert_eq!(c.bottom_left, Some(4.0));
    }

    #[test]
    fn merge_other_some_wins() {
        let base = Corners::all(4.0);
        let over = Corners {
            top_left: Some(16.0),
            top_right: None,
            bottom_right: None,
            bottom_left: None,
        };
        let merged = base.merge(&over);
        assert_eq!(merged.top_left, Some(16.0));
        assert_eq!(merged.top_right, Some(4.0));
        assert_eq!(merged.bottom_right, Some(4.0));
        assert_eq!(merged.bottom_left, Some(4.0));
    }

    #[test]
    fn merge_preserves_base_when_other_is_none() {
        let base = Corners::all(8.0);
        let over: Corners<f32> = Corners::default();
        let merged = base.merge(&over);
        assert_eq!(merged, base);
    }
}
