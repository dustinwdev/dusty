//! Command encoder — converts styles and layout rects into draw commands.

use crate::clip::ClipStack;
use crate::primitive::{
    ClipRegion, DrawCommand, DrawPrimitive, GradientData, PrimitiveFlags, Rect, ShadowPrimitive,
    MAX_GRADIENT_STOPS,
};
use dusty_style::{Overflow, Style};

/// Encodes styled elements into a sequence of [`DrawCommand`]s.
///
/// Manages the clip stack and converts `Style` + layout `Rect` pairs
/// into GPU-ready draw primitives.
///
/// # Examples
///
/// ```
/// use dusty_render::{CommandEncoder, Rect};
/// use dusty_style::{Color, Style};
///
/// let mut encoder = CommandEncoder::new();
/// let style = Style { background: Some(Color::WHITE), ..Style::default() };
/// let rect = Rect { x: 0.0, y: 0.0, width: 100.0, height: 50.0 };
/// let commands = encoder.encode_element(&style, &rect);
/// assert_eq!(commands.len(), 1);
/// ```
#[derive(Debug)]
pub struct CommandEncoder {
    clip_stack: ClipStack,
}

impl CommandEncoder {
    /// Creates a new command encoder with an empty clip stack.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            clip_stack: ClipStack::new(),
        }
    }

    /// Encodes a styled element at the given rect into draw commands.
    ///
    /// Returns commands in draw order: shadows first, then the element rect.
    /// Handles clipping by pushing/popping clip regions for overflow != Visible.
    pub fn encode_element(&mut self, style: &Style, rect: &Rect) -> Vec<DrawCommand> {
        let mut commands = Vec::new();

        let opacity = style.opacity.unwrap_or(1.0);

        // Skip fully transparent elements
        if opacity <= 0.0 {
            return commands;
        }

        let clip_rect = self.clip_stack.current().map(|c| c.rect);
        let clip_radii = self.clip_stack.current().map_or([0.0; 4], |c| c.radii);

        // Encode shadows before the element
        if let Some(shadows) = &style.shadow {
            for shadow in shadows {
                let shadow_rect = if shadow.inset {
                    *rect
                } else {
                    Rect {
                        x: rect.x + shadow.offset_x - shadow.spread_radius,
                        y: rect.y + shadow.offset_y - shadow.spread_radius,
                        width: shadow.spread_radius.mul_add(2.0, rect.width),
                        height: shadow.spread_radius.mul_add(2.0, rect.height),
                    }
                };

                let radii = extract_radii(style);

                commands.push(DrawCommand::Shadow(ShadowPrimitive {
                    rect: shadow_rect,
                    radii,
                    color: shadow.color,
                    blur_radius: shadow.blur_radius,
                    inset: shadow.inset,
                    opacity,
                    clip_rect,
                }));
            }
        }

        // Determine if we need to draw the rect at all
        let has_background = style.background.is_some();
        let has_gradient = style.background_gradient.is_some();
        let has_border = has_nonzero_border(style);

        if !has_background && !has_gradient && !has_border {
            return commands;
        }

        let radii = extract_radii(style);
        let mut flags = PrimitiveFlags::empty();

        if radii.iter().any(|&r| r > 0.0) {
            flags |= PrimitiveFlags::ROUNDED;
        }

        let border_widths = extract_border_widths(style);
        let border_color = style
            .border_color
            .unwrap_or(dusty_style::Color::TRANSPARENT);
        if has_border {
            flags |= PrimitiveFlags::BORDERED;
        }

        let clip_has_radii = clip_radii.iter().any(|&r| r > 0.0);
        if clip_has_radii {
            flags |= PrimitiveFlags::CLIP_ROUNDED;
        }

        // Gradient
        let gradient = style.background_gradient.as_ref().map(|lg| {
            flags |= PrimitiveFlags::GRADIENT;
            let angle_radians = lg.direction.to_angle().to_radians();
            let stops: Vec<(dusty_style::Color, f32)> = lg
                .stops
                .iter()
                .take(MAX_GRADIENT_STOPS)
                .map(|s| (s.color, s.position))
                .collect();
            GradientData {
                angle_radians,
                stops,
            }
        });

        let fill_color = style.background.unwrap_or(dusty_style::Color::TRANSPARENT);

        commands.push(DrawCommand::Rect(DrawPrimitive {
            rect: *rect,
            radii,
            fill_color,
            border_color,
            border_widths,
            opacity,
            clip_rect,
            clip_radii,
            flags,
            gradient,
        }));

        commands
    }

    /// Pushes a clip region for an element with overflow != Visible.
    pub fn push_clip(&mut self, region: ClipRegion) -> DrawCommand {
        self.clip_stack.push(region);
        DrawCommand::PushClip(region)
    }

    /// Pops the most recent clip region.
    pub fn pop_clip(&mut self) -> DrawCommand {
        self.clip_stack.pop();
        DrawCommand::PopClip
    }

    /// Encodes a clip push if the style has overflow != Visible,
    /// returning the push command if one was generated.
    pub fn maybe_push_clip(&mut self, style: &Style, rect: &Rect) -> Option<DrawCommand> {
        let overflow = style.overflow.unwrap_or(Overflow::Visible);
        if overflow == Overflow::Visible {
            return None;
        }
        let radii = extract_radii(style);
        Some(self.push_clip(ClipRegion { rect: *rect, radii }))
    }

    /// Returns a reference to the internal clip stack.
    #[must_use]
    pub const fn clip_stack(&self) -> &ClipStack {
        &self.clip_stack
    }
}

impl Default for CommandEncoder {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_radii(style: &Style) -> [f32; 4] {
    [
        style.border_radius.top_left.unwrap_or(0.0),
        style.border_radius.top_right.unwrap_or(0.0),
        style.border_radius.bottom_right.unwrap_or(0.0),
        style.border_radius.bottom_left.unwrap_or(0.0),
    ]
}

fn extract_border_widths(style: &Style) -> [f32; 4] {
    [
        style.border_width.top.unwrap_or(0.0),
        style.border_width.right.unwrap_or(0.0),
        style.border_width.bottom.unwrap_or(0.0),
        style.border_width.left.unwrap_or(0.0),
    ]
}

fn has_nonzero_border(style: &Style) -> bool {
    let bw = &style.border_width;
    style.border_color.is_some()
        && (bw.top.unwrap_or(0.0) > 0.0
            || bw.right.unwrap_or(0.0) > 0.0
            || bw.bottom.unwrap_or(0.0) > 0.0
            || bw.left.unwrap_or(0.0) > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusty_style::{
        BoxShadow, Color, ColorStop, Corners, Edges, GradientDirection, LinearGradient,
    };

    fn white_rect() -> (Style, Rect) {
        (
            Style {
                background: Some(Color::WHITE),
                ..Style::default()
            },
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
            },
        )
    }

    // -- Filled rect --

    #[test]
    fn solid_background_produces_one_rect_command() {
        let mut encoder = CommandEncoder::new();
        let (style, rect) = white_rect();
        let cmds = encoder.encode_element(&style, &rect);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(&cmds[0], DrawCommand::Rect(_)));
    }

    #[test]
    fn no_background_no_border_produces_no_commands() {
        let mut encoder = CommandEncoder::new();
        let style = Style::default();
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        assert!(cmds.is_empty());
    }

    #[test]
    fn opacity_applied_to_rect() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            opacity: Some(0.5),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert_eq!(prim.opacity, 0.5);
        } else {
            panic!("expected Rect command");
        }
    }

    #[test]
    fn zero_opacity_produces_no_commands() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            opacity: Some(0.0),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        assert!(cmds.is_empty());
    }

    // -- Rounded + bordered rects --

    #[test]
    fn rounded_rect_sets_flag_and_radii() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            border_radius: Corners::all(8.0),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert!(prim.flags.contains(PrimitiveFlags::ROUNDED));
            assert_eq!(prim.radii, [8.0; 4]);
        } else {
            panic!("expected Rect command");
        }
    }

    #[test]
    fn bordered_rect_sets_flag_and_widths() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            border_width: Edges::all(2.0),
            border_color: Some(Color::BLACK),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert!(prim.flags.contains(PrimitiveFlags::BORDERED));
            assert_eq!(prim.border_widths, [2.0; 4]);
            assert_eq!(prim.border_color, Color::BLACK);
        } else {
            panic!("expected Rect command");
        }
    }

    #[test]
    fn zero_border_width_no_flag() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            border_color: Some(Color::BLACK),
            // border_width defaults to all None (0.0)
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert!(!prim.flags.contains(PrimitiveFlags::BORDERED));
        } else {
            panic!("expected Rect command");
        }
    }

    #[test]
    fn border_without_color_no_flag() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            border_width: Edges::all(2.0),
            // no border_color
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert!(!prim.flags.contains(PrimitiveFlags::BORDERED));
        } else {
            panic!("expected Rect command");
        }
    }

    #[test]
    fn per_side_border_widths() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            border_width: Edges::new(1.0, 2.0, 3.0, 4.0),
            border_color: Some(Color::BLACK),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert_eq!(prim.border_widths, [1.0, 2.0, 3.0, 4.0]);
        } else {
            panic!("expected Rect command");
        }
    }

    #[test]
    fn per_corner_radii() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            border_radius: Corners::new(1.0, 2.0, 3.0, 4.0),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert_eq!(prim.radii, [1.0, 2.0, 3.0, 4.0]);
        } else {
            panic!("expected Rect command");
        }
    }

    // -- Shadows --

    #[test]
    fn shadow_before_rect() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            shadow: Some(vec![BoxShadow {
                offset_x: 0.0,
                offset_y: 4.0,
                blur_radius: 8.0,
                spread_radius: 0.0,
                color: Color::rgba(0.0, 0.0, 0.0, 0.25),
                inset: false,
            }]),
            ..Style::default()
        };
        let rect = Rect {
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        assert_eq!(cmds.len(), 2);
        assert!(matches!(&cmds[0], DrawCommand::Shadow(_)));
        assert!(matches!(&cmds[1], DrawCommand::Rect(_)));
    }

    #[test]
    fn shadow_offset_and_spread() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            shadow: Some(vec![BoxShadow {
                offset_x: 5.0,
                offset_y: 10.0,
                blur_radius: 8.0,
                spread_radius: 3.0,
                color: Color::BLACK,
                inset: false,
            }]),
            ..Style::default()
        };
        let rect = Rect {
            x: 20.0,
            y: 30.0,
            width: 100.0,
            height: 60.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Shadow(shadow) = &cmds[0] {
            // rect offset by (5,10), expanded by spread 3 on each side
            assert_eq!(shadow.rect.x, 20.0 + 5.0 - 3.0);
            assert_eq!(shadow.rect.y, 30.0 + 10.0 - 3.0);
            assert_eq!(shadow.rect.width, 100.0 + 6.0);
            assert_eq!(shadow.rect.height, 60.0 + 6.0);
        } else {
            panic!("expected Shadow command");
        }
    }

    #[test]
    fn multiple_shadows() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            shadow: Some(vec![
                BoxShadow {
                    offset_x: 0.0,
                    offset_y: 2.0,
                    blur_radius: 4.0,
                    spread_radius: 0.0,
                    color: Color::BLACK,
                    inset: false,
                },
                BoxShadow {
                    offset_x: 0.0,
                    offset_y: 10.0,
                    blur_radius: 20.0,
                    spread_radius: 0.0,
                    color: Color::BLACK,
                    inset: false,
                },
            ]),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        // 2 shadows + 1 rect
        assert_eq!(cmds.len(), 3);
        assert!(matches!(&cmds[0], DrawCommand::Shadow(_)));
        assert!(matches!(&cmds[1], DrawCommand::Shadow(_)));
        assert!(matches!(&cmds[2], DrawCommand::Rect(_)));
    }

    #[test]
    fn inset_shadow_uses_element_rect() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            shadow: Some(vec![BoxShadow {
                offset_x: 0.0,
                offset_y: 2.0,
                blur_radius: 4.0,
                spread_radius: 0.0,
                color: Color::BLACK,
                inset: true,
            }]),
            ..Style::default()
        };
        let rect = Rect {
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Shadow(shadow) = &cmds[0] {
            assert!(shadow.inset);
            // Inset shadow uses element rect directly
            assert_eq!(shadow.rect.x, rect.x);
            assert_eq!(shadow.rect.y, rect.y);
            assert_eq!(shadow.rect.width, rect.width);
            assert_eq!(shadow.rect.height, rect.height);
        } else {
            panic!("expected Shadow command");
        }
    }

    #[test]
    fn shadow_inherits_opacity() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background: Some(Color::WHITE),
            opacity: Some(0.7),
            shadow: Some(vec![BoxShadow {
                offset_x: 0.0,
                offset_y: 2.0,
                blur_radius: 4.0,
                spread_radius: 0.0,
                color: Color::BLACK,
                inset: false,
            }]),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Shadow(shadow) = &cmds[0] {
            assert_eq!(shadow.opacity, 0.7);
        } else {
            panic!("expected Shadow command");
        }
    }

    // -- Clipping --

    #[test]
    fn overflow_hidden_pushes_clip() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            overflow: Some(Overflow::Hidden),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let cmd = encoder.maybe_push_clip(&style, &rect);
        assert!(cmd.is_some());
        assert!(matches!(cmd, Some(DrawCommand::PushClip(_))));
        assert_eq!(encoder.clip_stack().depth(), 1);
    }

    #[test]
    fn overflow_visible_no_clip() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            overflow: Some(Overflow::Visible),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let cmd = encoder.maybe_push_clip(&style, &rect);
        assert!(cmd.is_none());
        assert!(encoder.clip_stack().is_empty());
    }

    #[test]
    fn clip_rect_propagates_to_draw_commands() {
        let mut encoder = CommandEncoder::new();

        // Push a clip
        encoder.push_clip(ClipRegion {
            rect: Rect {
                x: 10.0,
                y: 10.0,
                width: 80.0,
                height: 80.0,
            },
            radii: [0.0; 4],
        });

        // Encode an element within the clip
        let style = Style {
            background: Some(Color::WHITE),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert!(prim.clip_rect.is_some());
            let cr = prim.clip_rect.as_ref().unwrap();
            assert_eq!(cr.x, 10.0);
            assert_eq!(cr.width, 80.0);
        } else {
            panic!("expected Rect command");
        }
    }

    #[test]
    fn nested_clips_intersect_in_commands() {
        let mut encoder = CommandEncoder::new();

        encoder.push_clip(ClipRegion {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            radii: [0.0; 4],
        });

        encoder.push_clip(ClipRegion {
            rect: Rect {
                x: 50.0,
                y: 50.0,
                width: 100.0,
                height: 100.0,
            },
            radii: [0.0; 4],
        });

        let style = Style {
            background: Some(Color::WHITE),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            let cr = prim.clip_rect.as_ref().unwrap();
            assert_eq!(cr.x, 50.0);
            assert_eq!(cr.y, 50.0);
            assert_eq!(cr.width, 50.0);
            assert_eq!(cr.height, 50.0);
        } else {
            panic!("expected Rect command");
        }
    }

    #[test]
    fn clip_with_radii_sets_clip_rounded_flag() {
        let mut encoder = CommandEncoder::new();

        encoder.push_clip(ClipRegion {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            radii: [8.0; 4],
        });

        let style = Style {
            background: Some(Color::WHITE),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert!(prim.flags.contains(PrimitiveFlags::CLIP_ROUNDED));
            assert_eq!(prim.clip_radii, [8.0; 4]);
        } else {
            panic!("expected Rect command");
        }
    }

    // -- Gradients --

    #[test]
    fn gradient_sets_flag_and_data() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background_gradient: Some(LinearGradient {
                direction: GradientDirection::ToRight,
                stops: vec![
                    ColorStop {
                        color: Color::WHITE,
                        position: 0.0,
                    },
                    ColorStop {
                        color: Color::BLACK,
                        position: 1.0,
                    },
                ],
            }),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        assert_eq!(cmds.len(), 1);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert!(prim.flags.contains(PrimitiveFlags::GRADIENT));
            let gd = prim.gradient.as_ref().unwrap();
            assert_eq!(gd.stops.len(), 2);
            // ToRight = 90 degrees = pi/2 radians
            assert!((gd.angle_radians - std::f32::consts::FRAC_PI_2).abs() < 0.001);
        } else {
            panic!("expected Rect command");
        }
    }

    #[test]
    fn gradient_without_bg_still_renders() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            background_gradient: Some(LinearGradient {
                direction: GradientDirection::ToBottom,
                stops: vec![
                    ColorStop {
                        color: Color::WHITE,
                        position: 0.0,
                    },
                    ColorStop {
                        color: Color::BLACK,
                        position: 1.0,
                    },
                ],
            }),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        assert_eq!(cmds.len(), 1);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert_eq!(prim.fill_color, Color::TRANSPARENT);
        } else {
            panic!("expected Rect command");
        }
    }

    #[test]
    fn gradient_stops_capped_at_max() {
        let mut encoder = CommandEncoder::new();
        let stops: Vec<ColorStop> = (0..12)
            .map(|i| ColorStop {
                color: Color::WHITE,
                #[allow(clippy::cast_precision_loss)]
                position: i as f32 / 11.0,
            })
            .collect();
        let style = Style {
            background_gradient: Some(LinearGradient {
                direction: GradientDirection::ToRight,
                stops,
            }),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            let gd = prim.gradient.as_ref().unwrap();
            assert_eq!(gd.stops.len(), MAX_GRADIENT_STOPS);
        } else {
            panic!("expected Rect command");
        }
    }

    // -- Border only (no background) --

    #[test]
    fn border_only_produces_rect_command() {
        let mut encoder = CommandEncoder::new();
        let style = Style {
            border_width: Edges::all(1.0),
            border_color: Some(Color::BLACK),
            ..Style::default()
        };
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let cmds = encoder.encode_element(&style, &rect);
        assert_eq!(cmds.len(), 1);
        if let DrawCommand::Rect(prim) = &cmds[0] {
            assert!(prim.flags.contains(PrimitiveFlags::BORDERED));
            assert_eq!(prim.fill_color, Color::TRANSPARENT);
        } else {
            panic!("expected Rect command");
        }
    }

    // -- Default encoder --

    #[test]
    fn default_creates_new_encoder() {
        let encoder = CommandEncoder::default();
        assert!(encoder.clip_stack().is_empty());
    }
}
