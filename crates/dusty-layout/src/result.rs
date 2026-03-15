//! Layout result types — node identifiers and computed rectangles.

/// Opaque identifier for a node in the layout result.
///
/// Assigned to each `Element` and `Text` node during tree walk.
/// `Fragment` and `Component` nodes are transparent and do not receive IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutNodeId(pub usize);

/// An absolute screen-space rectangle.
///
/// Positions are computed from the root of the layout tree. The origin
/// `(0, 0)` is the top-left corner of the available space.
///
/// # Examples
///
/// ```
/// use dusty_layout::Rect;
///
/// let r = Rect { x: 10.0, y: 20.0, width: 100.0, height: 50.0 };
/// assert_eq!(r.x, 10.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Absolute X position (left edge).
    pub x: f32,
    /// Absolute Y position (top edge).
    pub y: f32,
    /// Width in pixels.
    pub width: f32,
    /// Height in pixels.
    pub height: f32,
}

/// The result of a layout computation.
///
/// Maps each [`LayoutNodeId`] to its computed [`Rect`].
#[derive(Debug)]
pub struct LayoutResult {
    rects: Vec<Rect>,
    root: LayoutNodeId,
}

impl LayoutResult {
    pub(crate) const fn new(rects: Vec<Rect>, root: LayoutNodeId) -> Self {
        Self { rects, root }
    }

    /// Returns the rectangle for the given node, or `None` if the ID is invalid.
    #[must_use]
    pub fn get(&self, id: LayoutNodeId) -> Option<&Rect> {
        self.rects.get(id.0)
    }

    /// Returns the root node's [`LayoutNodeId`].
    #[must_use]
    pub const fn root_id(&self) -> LayoutNodeId {
        self.root
    }

    /// Returns the root node's rectangle.
    #[must_use]
    pub fn root_rect(&self) -> Option<&Rect> {
        self.get(self.root)
    }

    /// Iterates over all `(LayoutNodeId, &Rect)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (LayoutNodeId, &Rect)> {
        self.rects
            .iter()
            .enumerate()
            .map(|(i, r)| (LayoutNodeId(i), r))
    }

    /// Returns the number of laid-out nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rects.len()
    }

    /// Returns `true` if no nodes were laid out.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_result() -> LayoutResult {
        LayoutResult::new(
            vec![
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                },
                Rect {
                    x: 10.0,
                    y: 10.0,
                    width: 80.0,
                    height: 30.0,
                },
            ],
            LayoutNodeId(0),
        )
    }

    #[test]
    fn get_valid_id() {
        let result = sample_result();
        let rect = result.get(LayoutNodeId(0));
        assert!(rect.is_some());
        assert_eq!(rect.map(|r| r.width), Some(100.0));
    }

    #[test]
    fn get_invalid_id() {
        let result = sample_result();
        assert!(result.get(LayoutNodeId(99)).is_none());
    }

    #[test]
    fn root_id_and_rect() {
        let result = sample_result();
        assert_eq!(result.root_id(), LayoutNodeId(0));
        let root = result.root_rect();
        assert!(root.is_some());
        assert_eq!(root.map(|r| r.x), Some(0.0));
    }

    #[test]
    fn iter_returns_all_nodes() {
        let result = sample_result();
        let items: Vec<_> = result.iter().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, LayoutNodeId(0));
        assert_eq!(items[1].0, LayoutNodeId(1));
    }

    #[test]
    fn len_and_is_empty() {
        let result = sample_result();
        assert_eq!(result.len(), 2);
        assert!(!result.is_empty());

        let empty = LayoutResult::new(vec![], LayoutNodeId(0));
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn rect_equality() {
        let a = Rect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        };
        let b = Rect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn layout_node_id_equality_and_hash() {
        use std::collections::HashSet;
        let a = LayoutNodeId(0);
        let b = LayoutNodeId(0);
        let c = LayoutNodeId(1);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
        assert!(!set.contains(&c));
    }
}
