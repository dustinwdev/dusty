//! Generic four-sided edge values for padding, margin, and border width.

/// Four-sided edge values. `None` means "not set" (inherits or defaults).
///
/// # Examples
///
/// ```
/// use dusty_style::Edges;
///
/// let uniform = Edges::all(8.0);
/// assert_eq!(uniform.top, Some(8.0));
///
/// let xy = Edges::xy(16.0, 8.0);
/// assert_eq!(xy.left, Some(16.0));
/// assert_eq!(xy.top, Some(8.0));
/// ```
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Edges<T> {
    /// Top edge value.
    pub top: Option<T>,
    /// Right edge value.
    pub right: Option<T>,
    /// Bottom edge value.
    pub bottom: Option<T>,
    /// Left edge value.
    pub left: Option<T>,
}

impl<T: Copy> Edges<T> {
    /// All four sides set to the same value.
    #[must_use]
    pub const fn all(value: T) -> Self {
        Self {
            top: Some(value),
            right: Some(value),
            bottom: Some(value),
            left: Some(value),
        }
    }

    /// Horizontal (`x`) and vertical (`y`) pairs.
    #[must_use]
    pub const fn xy(x: T, y: T) -> Self {
        Self {
            top: Some(y),
            right: Some(x),
            bottom: Some(y),
            left: Some(x),
        }
    }

    /// Each side specified individually.
    #[must_use]
    pub const fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top: Some(top),
            right: Some(right),
            bottom: Some(bottom),
            left: Some(left),
        }
    }

    /// Merges `other` on top of `self`. Other's `Some` values win.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            top: other.top.or(self.top),
            right: other.right.or(self.right),
            bottom: other.bottom.or(self.bottom),
            left: other.left.or(self.left),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_none() {
        let e: Edges<f32> = Edges::default();
        assert_eq!(e.top, None);
        assert_eq!(e.right, None);
        assert_eq!(e.bottom, None);
        assert_eq!(e.left, None);
    }

    #[test]
    fn all_sets_every_side() {
        let e = Edges::all(10.0);
        assert_eq!(e.top, Some(10.0));
        assert_eq!(e.right, Some(10.0));
        assert_eq!(e.bottom, Some(10.0));
        assert_eq!(e.left, Some(10.0));
    }

    #[test]
    fn xy_sets_horizontal_and_vertical() {
        let e = Edges::xy(16.0, 8.0);
        assert_eq!(e.left, Some(16.0));
        assert_eq!(e.right, Some(16.0));
        assert_eq!(e.top, Some(8.0));
        assert_eq!(e.bottom, Some(8.0));
    }

    #[test]
    fn new_sets_each_side() {
        let e = Edges::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(e.top, Some(1.0));
        assert_eq!(e.right, Some(2.0));
        assert_eq!(e.bottom, Some(3.0));
        assert_eq!(e.left, Some(4.0));
    }

    #[test]
    fn merge_other_some_wins() {
        let base = Edges::all(10.0);
        let over = Edges {
            top: Some(20.0),
            right: None,
            bottom: None,
            left: None,
        };
        let merged = base.merge(&over);
        assert_eq!(merged.top, Some(20.0));
        assert_eq!(merged.right, Some(10.0));
        assert_eq!(merged.bottom, Some(10.0));
        assert_eq!(merged.left, Some(10.0));
    }

    #[test]
    fn merge_preserves_base_when_other_is_none() {
        let base = Edges::all(5.0);
        let over: Edges<f32> = Edges::default();
        let merged = base.merge(&over);
        assert_eq!(merged, base);
    }
}
