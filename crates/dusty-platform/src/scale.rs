//! Scale factor conversion between logical and physical coordinates.

use crate::config::{LogicalSize, PhysicalSize};

/// A logical position.
#[derive(Debug, Clone, Copy)]
pub struct LogicalPosition {
    /// X coordinate in logical pixels.
    pub x: f64,
    /// Y coordinate in logical pixels.
    pub y: f64,
}

impl PartialEq for LogicalPosition {
    fn eq(&self, other: &Self) -> bool {
        self.x.total_cmp(&other.x).is_eq() && self.y.total_cmp(&other.y).is_eq()
    }
}

/// A physical position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalPosition {
    /// X coordinate in physical pixels.
    pub x: u32,
    /// Y coordinate in physical pixels.
    pub y: u32,
}

/// A display scale factor (ratio of physical to logical pixels).
///
/// # Example
///
/// ```
/// use dusty_platform::{ScaleFactor, LogicalSize, PhysicalSize};
///
/// fn example() -> Option<()> {
///     let scale = ScaleFactor::new(2.0)?;
///     let physical = scale.to_physical(LogicalSize { width: 800.0, height: 600.0 });
///     assert_eq!(physical, PhysicalSize { width: 1600, height: 1200 });
///     Some(())
/// }
/// assert!(example().is_some());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ScaleFactor(f64);

impl PartialEq for ScaleFactor {
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0).is_eq()
    }
}

/// The default scale factor (1.0).
const DEFAULT_SCALE: ScaleFactor = ScaleFactor(1.0);

impl ScaleFactor {
    /// Creates a new scale factor, validating it is positive and finite.
    ///
    /// Returns `None` if the value is not positive or not finite.
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if value > 0.0 && value.is_finite() {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the raw scale factor value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Returns the default 1.0 scale factor.
    #[must_use]
    pub(crate) const fn default_scale() -> Self {
        DEFAULT_SCALE
    }

    /// Converts a logical size to physical pixels.
    #[must_use]
    pub fn to_physical(self, logical: LogicalSize) -> PhysicalSize {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        PhysicalSize {
            width: (logical.width * self.0).round() as u32,
            height: (logical.height * self.0).round() as u32,
        }
    }

    /// Converts a physical size to logical pixels.
    #[must_use]
    pub fn to_logical(self, physical: PhysicalSize) -> LogicalSize {
        LogicalSize {
            width: f64::from(physical.width) / self.0,
            height: f64::from(physical.height) / self.0,
        }
    }

    /// Converts a logical position to physical pixels.
    #[must_use]
    pub fn to_physical_position(self, logical: LogicalPosition) -> PhysicalPosition {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        PhysicalPosition {
            x: (logical.x * self.0).round() as u32,
            y: (logical.y * self.0).round() as u32,
        }
    }

    /// Converts a physical position to logical pixels.
    #[must_use]
    pub fn to_logical_position(self, physical: PhysicalPosition) -> LogicalPosition {
        LogicalPosition {
            x: f64::from(physical.x) / self.0,
            y: f64::from(physical.y) / self.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_valid_scale() {
        assert!(ScaleFactor::new(1.0).is_some());
        assert!(ScaleFactor::new(2.0).is_some());
        assert!(ScaleFactor::new(0.5).is_some());
    }

    #[test]
    fn new_rejects_zero() {
        assert!(ScaleFactor::new(0.0).is_none());
    }

    #[test]
    fn new_rejects_negative() {
        assert!(ScaleFactor::new(-1.0).is_none());
    }

    #[test]
    fn new_rejects_nan() {
        assert!(ScaleFactor::new(f64::NAN).is_none());
    }

    #[test]
    fn new_rejects_infinity() {
        assert!(ScaleFactor::new(f64::INFINITY).is_none());
        assert!(ScaleFactor::new(f64::NEG_INFINITY).is_none());
    }

    #[test]
    fn value_returns_inner() {
        let scale = ScaleFactor::new(2.0).unwrap();
        assert!((scale.value() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn to_physical_1x() {
        let scale = ScaleFactor::new(1.0).unwrap();
        let result = scale.to_physical(LogicalSize {
            width: 800.0,
            height: 600.0,
        });
        assert_eq!(
            result,
            PhysicalSize {
                width: 800,
                height: 600
            }
        );
    }

    #[test]
    fn to_physical_2x() {
        let scale = ScaleFactor::new(2.0).unwrap();
        let result = scale.to_physical(LogicalSize {
            width: 800.0,
            height: 600.0,
        });
        assert_eq!(
            result,
            PhysicalSize {
                width: 1600,
                height: 1200
            }
        );
    }

    #[test]
    fn to_logical_2x() {
        let scale = ScaleFactor::new(2.0).unwrap();
        let result = scale.to_logical(PhysicalSize {
            width: 1600,
            height: 1200,
        });
        assert_eq!(
            result,
            LogicalSize {
                width: 800.0,
                height: 600.0
            }
        );
    }

    #[test]
    fn to_physical_fractional() {
        let scale = ScaleFactor::new(1.5).unwrap();
        let result = scale.to_physical(LogicalSize {
            width: 100.0,
            height: 100.0,
        });
        assert_eq!(
            result,
            PhysicalSize {
                width: 150,
                height: 150
            }
        );
    }

    #[test]
    fn to_physical_rounds() {
        let scale = ScaleFactor::new(1.5).unwrap();
        let result = scale.to_physical(LogicalSize {
            width: 101.0,
            height: 101.0,
        });
        // 101.0 * 1.5 = 151.5 → rounds to 152
        assert_eq!(
            result,
            PhysicalSize {
                width: 152,
                height: 152
            }
        );
    }

    #[test]
    fn to_physical_position_2x() {
        let scale = ScaleFactor::new(2.0).unwrap();
        let result = scale.to_physical_position(LogicalPosition { x: 10.0, y: 20.0 });
        assert_eq!(result, PhysicalPosition { x: 20, y: 40 });
    }

    #[test]
    fn to_logical_position_2x() {
        let scale = ScaleFactor::new(2.0).unwrap();
        let result = scale.to_logical_position(PhysicalPosition { x: 20, y: 40 });
        assert_eq!(result, LogicalPosition { x: 10.0, y: 20.0 });
    }

    #[test]
    fn scale_factor_equality() {
        assert_eq!(ScaleFactor::new(2.0), ScaleFactor::new(2.0));
        assert_ne!(ScaleFactor::new(1.0), ScaleFactor::new(2.0));
    }

    #[test]
    fn default_scale_is_1() {
        let scale = ScaleFactor::default_scale();
        assert!((scale.value() - 1.0).abs() < f64::EPSILON);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn logical_physical_size_round_trip(
            w in 1u32..4096,
            h in 1u32..4096,
            scale in 0.5f64..4.0,
        ) {
            let scale = ScaleFactor::new(scale).unwrap();
            let physical = PhysicalSize { width: w, height: h };
            let logical = scale.to_logical(physical);
            let back = scale.to_physical(logical);
            // Round-trip should be within 1 pixel due to rounding
            let dw = i64::from(back.width) - i64::from(w);
            let dh = i64::from(back.height) - i64::from(h);
            prop_assert!(dw.abs() <= 1, "width delta: {dw}");
            prop_assert!(dh.abs() <= 1, "height delta: {dh}");
        }

        #[test]
        fn logical_physical_position_round_trip(
            x in 0u32..4096,
            y in 0u32..4096,
            scale in 0.5f64..4.0,
        ) {
            let scale = ScaleFactor::new(scale).unwrap();
            let physical = PhysicalPosition { x, y };
            let logical = scale.to_logical_position(physical);
            let back = scale.to_physical_position(logical);
            let dx = i64::from(back.x) - i64::from(x);
            let dy = i64::from(back.y) - i64::from(y);
            prop_assert!(dx.abs() <= 1, "x delta: {dx}");
            prop_assert!(dy.abs() <= 1, "y delta: {dy}");
        }
    }
}
