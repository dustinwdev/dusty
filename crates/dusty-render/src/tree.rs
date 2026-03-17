//! Render tree walker — converts a `Node` tree + layout into draw commands.
//!
//! Traverses the node tree in pre-order depth-first order (mirroring
//! `TreeBuilder::build_node` in `dusty-layout/src/tree.rs`) and emits
//! [`DrawCommand`]s for each visible node.
//!
//! # Examples
//!
//! ```no_run
//! use dusty_render::tree::walk_tree;
//! # // walk_tree requires a full node tree, layout, text system, etc.
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use dusty_core::Node;
use dusty_layout::{LayoutNodeId, LayoutResult};
use dusty_style::{Color, FontStyle, Style};
use dusty_text::{GlyphRasterizer, TextLayout, TextSystem};

use crate::command::CommandEncoder;
use crate::glyph_cache::GlyphCache;
use crate::image_cache::ImageCache;
use crate::primitive::{
    DrawCommand, DrawPrimitive, ImageId, ImagePrimitive, PrimitiveFlags, Rect, TextGlyph,
    TextPrimitive,
};

/// Default foreground (text) color when none is specified.
const DEFAULT_FOREGROUND: Color = Color::BLACK;

/// Walks the node tree and emits draw commands.
///
/// The traversal mirrors `TreeBuilder::build_node` in `dusty-layout`:
/// - `Element` and `Text` nodes consume a `LayoutNodeId` (incrementing).
/// - `Fragment` and `Component` nodes are transparent (no ID consumed).
///
/// # Parameters
///
/// - `root` — the root of the node tree.
/// - `layout` — the computed layout result.
/// - `text_system` — text system for creating `TextLayout`s.
/// - `glyph_cache` — cache for rasterized glyph bitmaps.
/// - `rasterizer` — glyph rasterizer.
/// - `scale_factor` — display scale factor (1.0 for 1x, 2.0 for Retina).
pub fn walk_tree(
    root: &Node,
    layout: &LayoutResult,
    text_system: &TextSystem,
    glyph_cache: &mut GlyphCache,
    rasterizer: &mut GlyphRasterizer,
    scale_factor: f32,
) -> Vec<DrawCommand> {
    walk_tree_with_images(
        root,
        layout,
        text_system,
        glyph_cache,
        rasterizer,
        scale_factor,
        None,
    )
}

/// Walks the node tree with an optional image cache for Image element support.
pub fn walk_tree_with_images(
    root: &Node,
    layout: &LayoutResult,
    text_system: &TextSystem,
    glyph_cache: &mut GlyphCache,
    rasterizer: &mut GlyphRasterizer,
    scale_factor: f32,
    image_cache: Option<&ImageCache>,
) -> Vec<DrawCommand> {
    let mut ctx = WalkContext {
        commands: Vec::new(),
        encoder: CommandEncoder::new(),
        next_id: 0,
        text_system,
        glyph_cache,
        rasterizer,
        scale_factor,
        image_cache,
    };

    ctx.walk_node(root, layout, &FontStyle::default(), DEFAULT_FOREGROUND);
    ctx.commands
}

struct WalkContext<'a> {
    commands: Vec<DrawCommand>,
    encoder: CommandEncoder,
    next_id: usize,
    text_system: &'a TextSystem,
    glyph_cache: &'a mut GlyphCache,
    rasterizer: &'a mut GlyphRasterizer,
    scale_factor: f32,
    image_cache: Option<&'a ImageCache>,
}

impl WalkContext<'_> {
    fn alloc_id(&mut self) -> LayoutNodeId {
        let id = LayoutNodeId(self.next_id);
        self.next_id += 1;
        id
    }

    fn walk_node(
        &mut self,
        node: &Node,
        layout: &LayoutResult,
        inherited_font: &FontStyle,
        inherited_fg: Color,
    ) {
        match node {
            Node::Element(el) => {
                self.walk_element(el, layout, inherited_font, inherited_fg);
            }
            Node::Text(text_node) => {
                self.walk_text(text_node, layout, inherited_font, inherited_fg);
            }
            Node::Fragment(children) => {
                for child in children {
                    self.walk_node(child, layout, inherited_font, inherited_fg);
                }
            }
            Node::Component(comp) => {
                self.walk_node(&comp.child, layout, inherited_font, inherited_fg);
            }
            Node::Dynamic(dn) => {
                let resolved = dn.current_node();
                self.walk_node(&resolved, layout, inherited_font, inherited_fg);
            }
        }
    }

    fn walk_element(
        &mut self,
        el: &dusty_core::Element,
        layout: &LayoutResult,
        inherited_font: &FontStyle,
        inherited_fg: Color,
    ) {
        let layout_id = self.alloc_id();
        let Some(layout_rect) = layout.get(layout_id) else {
            return;
        };

        let render_rect = to_render_rect(layout_rect);

        // Downcast style
        let dusty_style = if el.style().is::<()>() {
            Style::default()
        } else {
            el.style()
                .downcast_ref::<Style>()
                .cloned()
                .unwrap_or_default()
        };

        // Merge font for children
        let child_font = inherited_font.merge(&dusty_style.font);
        let child_fg = dusty_style.foreground.unwrap_or(inherited_fg);

        // Maybe push clip
        let pushed_clip = self.encoder.maybe_push_clip(&dusty_style, &render_rect);
        let has_clip = pushed_clip.is_some();
        if let Some(clip_cmd) = pushed_clip {
            self.commands.push(clip_cmd);
        }

        // Encode element visuals (shadows, rect)
        let element_cmds = self.encoder.encode_element(&dusty_style, &render_rect);
        self.commands.extend(element_cmds);

        // Image element — emit DrawCommand::Image
        if el.name() == "Image" {
            self.walk_image_element(el, &render_rect, &dusty_style);
        }

        // Canvas element — emit draw commands from custom_data
        if el.name() == "Canvas" {
            self.walk_canvas_element(el, &render_rect, &dusty_style);
        }

        // Recurse children
        for child in el.children() {
            self.walk_node(child, layout, &child_font, child_fg);
        }

        // Pop clip if we pushed one
        if has_clip {
            let pop = self.encoder.pop_clip();
            self.commands.push(pop);
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn walk_image_element(&mut self, el: &dusty_core::Element, render_rect: &Rect, style: &Style) {
        // Read attributes
        let src = el.attr("src").and_then(|v| match v {
            dusty_core::AttributeValue::String(s) => Some(s.as_str()),
            _ => None,
        });
        let sizing_mode = el
            .attr("sizing_mode")
            .and_then(|v| match v {
                dusty_core::AttributeValue::String(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("cover");

        let Some(src) = src else {
            return;
        };

        // Look up image in cache to get intrinsic dimensions
        let image_source = crate::image_cache::ImageSource::Path(std::path::PathBuf::from(src));
        let (intrinsic_w, intrinsic_h) = if let Some(cache) = self.image_cache {
            if let Some(&id) = cache.source_to_id().get(&image_source) {
                if let Some(decoded) = cache.get(id) {
                    (decoded.width as f32, decoded.height as f32)
                } else {
                    return;
                }
            } else {
                // Image not loaded — use fill UV as fallback
                (render_rect.width, render_rect.height)
            }
        } else {
            // No image cache — use fill UV
            (render_rect.width, render_rect.height)
        };

        let uv = compute_uv(sizing_mode, render_rect, intrinsic_w, intrinsic_h);
        let opacity = style.opacity.unwrap_or(1.0);
        let clip_rect = self.encoder.clip_stack().current().map(|c| c.rect);

        // Use a deterministic texture ID from the source path hash
        let texture_id = ImageId(hash_string(src));

        self.commands.push(DrawCommand::Image(ImagePrimitive {
            rect: *render_rect,
            texture_id,
            uv,
            opacity,
            clip_rect,
        }));
    }

    fn walk_canvas_element(&mut self, el: &dusty_core::Element, render_rect: &Rect, style: &Style) {
        type CanvasCommands = Rc<RefCell<Vec<dusty_widgets_canvas::CanvasCommand>>>;

        let Some(cmds_rc) = el.custom_data().downcast_ref::<CanvasCommands>() else {
            return;
        };

        let cmds = cmds_rc.borrow();
        if cmds.is_empty() {
            return;
        }

        let opacity = style.opacity.unwrap_or(1.0);
        let clip_rect = self.encoder.clip_stack().current().map(|c| c.rect);
        let ox = render_rect.x;
        let oy = render_rect.y;

        for cmd in cmds.iter() {
            self.emit_canvas_command(cmd, ox, oy, opacity, clip_rect);
        }
    }

    fn emit_canvas_command(
        &mut self,
        cmd: &dusty_widgets_canvas::CanvasCommand,
        ox: f32,
        oy: f32,
        opacity: f32,
        clip_rect: Option<Rect>,
    ) {
        use dusty_widgets_canvas::CanvasCommand as CC;

        match cmd {
            CC::Rect {
                x,
                y,
                width,
                height,
                fill: Some(dusty_widgets_canvas::FillStyle::Solid(color)),
                ..
            } => {
                self.commands.push(DrawCommand::Rect(DrawPrimitive {
                    rect: Rect {
                        x: ox + x,
                        y: oy + y,
                        width: *width,
                        height: *height,
                    },
                    radii: [0.0; 4],
                    fill_color: *color,
                    border_color: Color::TRANSPARENT,
                    border_widths: [0.0; 4],
                    opacity,
                    clip_rect,
                    clip_radii: [0.0; 4],
                    flags: PrimitiveFlags::empty(),
                    gradient: None,
                }));
            }
            CC::RoundRect {
                x,
                y,
                width,
                height,
                radius,
                fill: Some(dusty_widgets_canvas::FillStyle::Solid(color)),
                ..
            } => {
                self.commands.push(DrawCommand::Rect(DrawPrimitive {
                    rect: Rect {
                        x: ox + x,
                        y: oy + y,
                        width: *width,
                        height: *height,
                    },
                    radii: [*radius; 4],
                    fill_color: *color,
                    border_color: Color::TRANSPARENT,
                    border_widths: [0.0; 4],
                    opacity,
                    clip_rect,
                    clip_radii: [0.0; 4],
                    flags: PrimitiveFlags::ROUNDED,
                    gradient: None,
                }));
            }
            CC::Circle {
                cx,
                cy,
                radius,
                fill: Some(dusty_widgets_canvas::FillStyle::Solid(color)),
                ..
            } => {
                self.commands.push(DrawCommand::Rect(DrawPrimitive {
                    rect: Rect {
                        x: ox + cx - radius,
                        y: oy + cy - radius,
                        width: radius * 2.0,
                        height: radius * 2.0,
                    },
                    radii: [*radius; 4],
                    fill_color: *color,
                    border_color: Color::TRANSPARENT,
                    border_widths: [0.0; 4],
                    opacity,
                    clip_rect,
                    clip_radii: [0.0; 4],
                    flags: PrimitiveFlags::ROUNDED,
                    gradient: None,
                }));
            }
            // Path commands, transforms, clips, text, image — deferred to later phases
            _ => {}
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn walk_text(
        &mut self,
        text_node: &dusty_core::TextNode,
        layout: &LayoutResult,
        inherited_font: &FontStyle,
        inherited_fg: Color,
    ) {
        let layout_id = self.alloc_id();
        let Some(layout_rect) = layout.get(layout_id) else {
            return;
        };

        let render_rect = to_render_rect(layout_rect);
        let text = text_node.current_text();

        if text.is_empty() {
            return;
        }

        let max_width = if render_rect.width > 0.0 {
            Some(render_rect.width)
        } else {
            None
        };
        let Ok(text_layout) = TextLayout::new(self.text_system, &text, inherited_font, max_width)
        else {
            eprintln!("dusty: FontSystem borrow conflict during text layout");
            return;
        };

        let (tw, th) = text_layout.size();
        if tw == 0.0 && th == 0.0 {
            eprintln!("dusty: text measurement returned (0, 0) for non-empty text, font system may be unavailable");
        }

        let clip_rect = self.encoder.clip_stack().current().map(|c| c.rect);
        let fg_color = [
            inherited_fg.r,
            inherited_fg.g,
            inherited_fg.b,
            inherited_fg.a,
        ];

        let mut glyphs = Vec::new();
        let Ok(mut font_system) = self.text_system.font_system_mut() else {
            eprintln!("dusty: FontSystem borrow conflict during glyph rasterization");
            return;
        };

        for run in text_layout.buffer().layout_runs() {
            for layout_glyph in run.glyphs {
                let physical = layout_glyph.physical((0., 0.), self.scale_factor);

                let cached = self.glyph_cache.get_or_rasterize(
                    physical.cache_key,
                    self.rasterizer,
                    &mut font_system,
                );

                if let Some(cached) = cached {
                    let glyph_x = render_rect.x + physical.x as f32 + cached.offset[0] as f32;
                    let glyph_y =
                        render_rect.y + run.line_top + physical.y as f32 - cached.offset[1] as f32;

                    glyphs.push(TextGlyph {
                        x: glyph_x,
                        y: glyph_y,
                        width: cached.size[0] as f32,
                        height: cached.size[1] as f32,
                        uv: cached.uv,
                        color: fg_color,
                        opacity: 1.0,
                        clip_rect,
                    });
                }
            }
        }

        drop(font_system);

        if !glyphs.is_empty() {
            self.commands
                .push(DrawCommand::Text(TextPrimitive { glyphs }));
        }
    }
}

/// Converts a layout `Rect` to a render `Rect`.
const fn to_render_rect(layout_rect: &dusty_layout::Rect) -> Rect {
    Rect {
        x: layout_rect.x,
        y: layout_rect.y,
        width: layout_rect.width,
        height: layout_rect.height,
    }
}

/// Compute UV coordinates based on sizing mode.
///
/// - `"fill"` — stretch to fill: `[0, 0, 1, 1]`
/// - `"cover"` — crop to fill, center-aligned
/// - `"contain"` — letterbox, center-aligned
fn compute_uv(
    sizing_mode: &str,
    render_rect: &Rect,
    intrinsic_w: f32,
    intrinsic_h: f32,
) -> [f32; 4] {
    if intrinsic_w <= 0.0 || intrinsic_h <= 0.0 {
        return [0.0, 0.0, 1.0, 1.0];
    }

    match sizing_mode {
        "cover" => {
            let aspect_src = intrinsic_w / intrinsic_h;
            let aspect_dst = render_rect.width / render_rect.height;
            if aspect_dst > aspect_src {
                // Wider than image — crop vertically
                let visible_h = intrinsic_w / aspect_dst;
                let offset_v = (intrinsic_h - visible_h) / (2.0 * intrinsic_h);
                let end_v = 1.0 - offset_v;
                [0.0, offset_v, 1.0, end_v]
            } else {
                // Taller than image — crop horizontally
                let visible_w = intrinsic_h * aspect_dst;
                let offset_u = (intrinsic_w - visible_w) / (2.0 * intrinsic_w);
                let end_u = 1.0 - offset_u;
                [offset_u, 0.0, end_u, 1.0]
            }
        }
        // "fill", "contain", and unknown modes all use full UV range.
        // Contain adjusts the dest rect at a higher level, not UVs.
        _ => [0.0, 0.0, 1.0, 1.0],
    }
}

/// Hashes a string to produce a deterministic image ID using `SipHash`.
fn hash_string(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Module alias for canvas command types used in the renderer.
mod dusty_widgets_canvas {
    pub use dusty_widgets::canvas::{CanvasCommand, FillStyle};
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::float_cmp,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
mod tests {
    use super::*;
    use dusty_core::{el, text, ComponentNode};
    use dusty_layout::compute_layout;
    use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};
    use dusty_style::{Color, Edges, Overflow};

    struct MockMeasure;
    impl dusty_layout::TextMeasure for MockMeasure {
        fn measure(&self, text: &str, max_width: Option<f32>, _font: &FontStyle) -> (f32, f32) {
            let char_width = 8.0;
            let line_height = 16.0;
            let text_width = text.len() as f32 * char_width;
            if let Some(max) = max_width {
                if text_width > max {
                    let chars_per_line = (max / char_width).floor() as usize;
                    if chars_per_line == 0 {
                        return (char_width, line_height);
                    }
                    let lines = text.len().div_ceil(chars_per_line);
                    return (max, lines as f32 * line_height);
                }
            }
            (text_width, line_height)
        }
    }

    fn with_scope(f: impl FnOnce(dusty_reactive::Scope)) {
        initialize_runtime();
        create_scope(f);
        dispose_runtime();
    }

    fn walk_with_mock(root: &Node, width: f32, height: f32) -> Vec<DrawCommand> {
        let layout = compute_layout(root, width, height, &MockMeasure).unwrap();
        let text_system = TextSystem::new();
        let mut glyph_cache = GlyphCache::new(256, 256);
        let mut rasterizer = GlyphRasterizer::new();

        walk_tree(
            root,
            &layout,
            &text_system,
            &mut glyph_cache,
            &mut rasterizer,
            1.0,
        )
    }

    #[test]
    fn single_element_emits_rect_commands() {
        with_scope(|cx| {
            let node = el("Box", cx)
                .style(Style {
                    width: Some(100.0),
                    height: Some(50.0),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            assert!(!cmds.is_empty());
            assert!(
                cmds.iter().any(|c| matches!(c, DrawCommand::Rect(_))),
                "should contain Rect command"
            );
        });
    }

    #[test]
    fn text_node_emits_text_command() {
        // Text nodes produce DrawCommand::Text with glyphs
        let node = Node::Text(text("hello"));
        let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
        let text_system = TextSystem::new();
        let mut glyph_cache = GlyphCache::new(512, 512);
        let mut rasterizer = GlyphRasterizer::new();

        let cmds = walk_tree(
            &node,
            &layout,
            &text_system,
            &mut glyph_cache,
            &mut rasterizer,
            1.0,
        );

        let has_text = cmds.iter().any(|c| matches!(c, DrawCommand::Text(_)));
        assert!(has_text, "should contain Text command, got: {cmds:?}");
    }

    #[test]
    fn parent_background_before_child_text() {
        with_scope(|cx| {
            let node = el("Parent", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .child(text("hello"))
                .build_node();

            let layout = compute_layout(&node, 400.0, 300.0, &MockMeasure).unwrap();
            let text_system = TextSystem::new();
            let mut glyph_cache = GlyphCache::new(512, 512);
            let mut rasterizer = GlyphRasterizer::new();

            let cmds = walk_tree(
                &node,
                &layout,
                &text_system,
                &mut glyph_cache,
                &mut rasterizer,
                1.0,
            );

            // Find first Rect and first Text
            let rect_idx = cmds.iter().position(|c| matches!(c, DrawCommand::Rect(_)));
            let text_idx = cmds.iter().position(|c| matches!(c, DrawCommand::Text(_)));

            if let (Some(ri), Some(ti)) = (rect_idx, text_idx) {
                assert!(ri < ti, "Rect should come before Text for z-ordering");
            }
        });
    }

    #[test]
    fn fragment_children_flattened_in_order() {
        with_scope(|cx| {
            let frag = Node::Fragment(vec![
                el("A", cx)
                    .style(Style {
                        width: Some(50.0),
                        height: Some(50.0),
                        background: Some(Color::WHITE),
                        ..Style::default()
                    })
                    .build_node(),
                el("B", cx)
                    .style(Style {
                        width: Some(50.0),
                        height: Some(50.0),
                        background: Some(Color::BLACK),
                        ..Style::default()
                    })
                    .build_node(),
            ]);

            let parent = el("Parent", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .child_node(frag)
                .build_node();

            let cmds = walk_with_mock(&parent, 400.0, 300.0);
            // Should have rects for both A and B
            let rect_count = cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::Rect(_)))
                .count();
            assert!(
                rect_count >= 2,
                "should have at least 2 rects, got {rect_count}"
            );
        });
    }

    #[test]
    fn component_node_transparent() {
        with_scope(|cx| {
            let inner = el("Inner", cx)
                .style(Style {
                    width: Some(80.0),
                    height: Some(40.0),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .build_node();

            let comp = Node::Component(ComponentNode {
                name: "MyComponent",
                child: Box::new(inner),
            });

            let parent = el("Parent", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .child_node(comp)
                .build_node();

            let cmds = walk_with_mock(&parent, 400.0, 300.0);
            // Should have rect for Inner (Component is transparent)
            let rect_count = cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::Rect(_)))
                .count();
            assert!(rect_count >= 1, "should have at least 1 rect for Inner");
        });
    }

    #[test]
    fn nested_overflow_hidden_emits_clips() {
        with_scope(|cx| {
            let node = el("Outer", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(200.0),
                    overflow: Some(Overflow::Hidden),
                    background: Some(Color::WHITE),
                    ..Style::default()
                })
                .child(
                    el("Inner", cx)
                        .style(Style {
                            width: Some(100.0),
                            height: Some(100.0),
                            overflow: Some(Overflow::Hidden),
                            background: Some(Color::BLACK),
                            ..Style::default()
                        })
                        .build_node(),
                )
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);

            let push_count = cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::PushClip(_)))
                .count();
            let pop_count = cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::PopClip))
                .count();

            assert_eq!(push_count, 2, "should have 2 PushClip commands");
            assert_eq!(pop_count, 2, "should have 2 PopClip commands");
            assert_eq!(push_count, pop_count, "push/pop must be balanced");
        });
    }

    #[test]
    fn empty_tree_produces_no_commands() {
        let node = Node::Fragment(vec![]);
        // Empty fragment returns error from compute_layout, so we skip.
        // Instead test with a styled but invisible element.
        with_scope(|cx| {
            let node = el("Empty", cx)
                .style(Style {
                    width: Some(0.0),
                    height: Some(0.0),
                    ..Style::default()
                })
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            // No background = no commands
            assert!(
                cmds.is_empty(),
                "empty/unstyled element should produce no commands"
            );
        });
    }

    #[test]
    fn deeply_nested_tree_correct_order() {
        with_scope(|cx| {
            let node = el("L1", cx)
                .style(Style {
                    width: Some(400.0),
                    height: Some(400.0),
                    background: Some(Color::WHITE),
                    padding: Edges::all(10.0),
                    ..Style::default()
                })
                .child(
                    el("L2", cx)
                        .style(Style {
                            width: Some(300.0),
                            height: Some(300.0),
                            background: Some(Color::hex(0xAAAAAA)),
                            padding: Edges::all(10.0),
                            ..Style::default()
                        })
                        .child(
                            el("L3", cx)
                                .style(Style {
                                    width: Some(200.0),
                                    height: Some(200.0),
                                    background: Some(Color::BLACK),
                                    ..Style::default()
                                })
                                .build_node(),
                        )
                        .build_node(),
                )
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 400.0);

            // Pre-order: L1 rect, L2 rect, L3 rect
            let rects: Vec<&DrawCommand> = cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::Rect(_)))
                .collect();
            assert_eq!(rects.len(), 3, "should have 3 rect commands");
        });
    }

    #[test]
    fn node_id_assignment_matches_layout() {
        with_scope(|cx| {
            // Build the exact same tree used for layout and verify
            // the walker consumes IDs in the same order.
            let node = el("Root", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .child(text("hello"))
                .build_node();

            let layout = compute_layout(&node, 200.0, 100.0, &MockMeasure).unwrap();

            // Layout should have 2 nodes: Root(0) + Text(1)
            assert_eq!(layout.len(), 2);

            let text_system = TextSystem::new();
            let mut glyph_cache = GlyphCache::new(256, 256);
            let mut rasterizer = GlyphRasterizer::new();

            // Should not panic — means IDs are assigned correctly
            let _cmds = walk_tree(
                &node,
                &layout,
                &text_system,
                &mut glyph_cache,
                &mut rasterizer,
                1.0,
            );
        });
    }

    // -- Image element tests --

    #[test]
    fn image_element_emits_image_command() {
        with_scope(|cx| {
            let node = el("Image", cx)
                .attr("src", "photo.png")
                .attr("sizing_mode", "fill")
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            let has_image = cmds.iter().any(|c| matches!(c, DrawCommand::Image(_)));
            assert!(has_image, "should contain Image command, got: {cmds:?}");
        });
    }

    #[test]
    fn sizing_fill_uv() {
        let uv = super::compute_uv(
            "fill",
            &Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 100.0,
            },
            400.0,
            200.0,
        );
        assert_eq!(uv, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn sizing_cover_uv() {
        // Container is 200x100 (aspect 2.0), image is 100x100 (aspect 1.0)
        // Container is wider — crop vertically
        let uv = super::compute_uv(
            "cover",
            &Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 100.0,
            },
            100.0,
            100.0,
        );
        // Visible height = 100/2 = 50, offset_v = (100-50)/(200) = 0.25
        assert!(
            (uv[1] - 0.25).abs() < 0.01,
            "v_start should be ~0.25, got {}",
            uv[1]
        );
        assert!(
            (uv[3] - 0.75).abs() < 0.01,
            "v_end should be ~0.75, got {}",
            uv[3]
        );
    }

    #[test]
    fn sizing_contain_uv() {
        let uv = super::compute_uv(
            "contain",
            &Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 100.0,
            },
            400.0,
            200.0,
        );
        assert_eq!(uv, [0.0, 0.0, 1.0, 1.0]);
    }

    // -- Canvas element tests --

    #[test]
    fn canvas_emits_rect_commands() {
        with_scope(|cx| {
            use dusty_widgets::canvas::{CanvasCommand, FillStyle};

            let commands: Rc<RefCell<Vec<CanvasCommand>>> =
                Rc::new(RefCell::new(vec![CanvasCommand::Rect {
                    x: 10.0,
                    y: 10.0,
                    width: 80.0,
                    height: 40.0,
                    fill: Some(FillStyle::Solid(Color::WHITE)),
                    stroke: None,
                }]));

            let node = el("Canvas", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .data(commands)
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            let rect_count = cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::Rect(_)))
                .count();
            assert!(
                rect_count >= 1,
                "should contain Rect from canvas, got {rect_count}"
            );
        });
    }

    #[test]
    fn commands_offset_by_position() {
        with_scope(|cx| {
            use dusty_widgets::canvas::{CanvasCommand, FillStyle};

            let commands: Rc<RefCell<Vec<CanvasCommand>>> =
                Rc::new(RefCell::new(vec![CanvasCommand::Rect {
                    x: 5.0,
                    y: 5.0,
                    width: 20.0,
                    height: 20.0,
                    fill: Some(FillStyle::Solid(Color::BLACK)),
                    stroke: None,
                }]));

            // Wrap in a parent so the canvas gets a non-zero position
            let canvas_node = el("Canvas", cx)
                .style(Style {
                    width: Some(100.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .data(commands)
                .build_node();

            let parent = el("Root", cx)
                .style(Style {
                    width: Some(400.0),
                    height: Some(300.0),
                    padding: Edges::all(20.0),
                    ..Style::default()
                })
                .child_node(canvas_node)
                .build_node();

            let cmds = walk_with_mock(&parent, 400.0, 300.0);

            // Find the canvas rect (drawn from the canvas element)
            let canvas_rects: Vec<_> = cmds
                .iter()
                .filter_map(|c| {
                    if let DrawCommand::Rect(p) = c {
                        if p.rect.width == 20.0 && p.rect.height == 20.0 {
                            Some(p)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            assert!(!canvas_rects.is_empty(), "should find the canvas rect");
            // Canvas is at (20, 20) due to parent padding, + (5, 5) from command offset
            let r = &canvas_rects[0].rect;
            assert_eq!(r.x, 25.0, "x should be parent_padding + command_x");
            assert_eq!(r.y, 25.0, "y should be parent_padding + command_y");
        });
    }

    #[test]
    fn empty_commands_no_output() {
        with_scope(|cx| {
            use dusty_widgets::canvas::CanvasCommand;

            let commands: Rc<RefCell<Vec<CanvasCommand>>> = Rc::new(RefCell::new(vec![]));

            let node = el("Canvas", cx)
                .style(Style {
                    width: Some(100.0),
                    height: Some(100.0),
                    ..Style::default()
                })
                .data(commands)
                .build_node();

            let cmds = walk_with_mock(&node, 400.0, 300.0);
            // No background and no canvas commands → empty
            assert!(cmds.is_empty(), "empty canvas should produce no commands");
        });
    }

    #[test]
    fn hash_string_known_djb2_collision_produces_distinct_ids() {
        // "Az" and "BY" collide under DJB2 but not under SipHash
        use super::hash_string;
        let h1 = hash_string("Az");
        let h2 = hash_string("BY");
        assert_ne!(
            h1, h2,
            "DJB2 collision pair must produce distinct SipHash values"
        );
    }

    #[test]
    fn inherited_foreground_color_used() {
        with_scope(|cx| {
            let node = el("Parent", cx)
                .style(Style {
                    width: Some(200.0),
                    height: Some(100.0),
                    foreground: Some(Color::rgb(1.0, 0.0, 0.0)), // red
                    ..Style::default()
                })
                .child(text("red text"))
                .build_node();

            let layout = compute_layout(&node, 200.0, 100.0, &MockMeasure).unwrap();
            let text_system = TextSystem::new();
            let mut glyph_cache = GlyphCache::new(512, 512);
            let mut rasterizer = GlyphRasterizer::new();

            let cmds = walk_tree(
                &node,
                &layout,
                &text_system,
                &mut glyph_cache,
                &mut rasterizer,
                1.0,
            );

            // Check that text glyphs have red color
            for cmd in &cmds {
                if let DrawCommand::Text(tp) = cmd {
                    for glyph in &tp.glyphs {
                        assert_eq!(glyph.color[0], 1.0, "text glyph red channel should be 1.0");
                        assert_eq!(
                            glyph.color[1], 0.0,
                            "text glyph green channel should be 0.0"
                        );
                    }
                }
            }
        });
    }
}
