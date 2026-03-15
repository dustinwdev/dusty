//! Integration tests for the command encoder.

use dusty_render::{ClipRegion, CommandEncoder, DrawCommand, PrimitiveFlags, Rect};
use dusty_style::{
    BoxShadow, Color, ColorStop, Corners, Edges, GradientDirection, LinearGradient, Overflow, Style,
};

#[test]
fn card_style_produces_shadow_then_rect() {
    let mut encoder = CommandEncoder::new();
    let style = Style::default()
        .bg_white()
        .rounded_lg()
        .shadow_md()
        .border(1.0, Color::hex(0xE5E7EB));

    let rect = Rect {
        x: 20.0,
        y: 20.0,
        width: 300.0,
        height: 200.0,
    };

    let cmds = encoder.encode_element(&style, &rect);

    // shadow_md produces 2 shadows, then the rect
    let shadow_count = cmds
        .iter()
        .filter(|c| matches!(c, DrawCommand::Shadow(_)))
        .count();
    let rect_count = cmds
        .iter()
        .filter(|c| matches!(c, DrawCommand::Rect(_)))
        .count();
    assert_eq!(shadow_count, 2);
    assert_eq!(rect_count, 1);

    // Rect should have rounded + bordered flags
    if let DrawCommand::Rect(prim) = cmds.last().unwrap() {
        assert!(prim.flags.contains(PrimitiveFlags::ROUNDED));
        assert!(prim.flags.contains(PrimitiveFlags::BORDERED));
        assert_eq!(prim.radii, [8.0; 4]); // rounded_lg = 8px
    } else {
        panic!("last command should be Rect");
    }
}

#[test]
fn gradient_button_encodes_correctly() {
    let mut encoder = CommandEncoder::new();
    let style = Style {
        background_gradient: Some(LinearGradient {
            direction: GradientDirection::ToRight,
            stops: vec![
                ColorStop {
                    color: Color::hex(0x3B82F6),
                    position: 0.0,
                },
                ColorStop {
                    color: Color::hex(0x8B5CF6),
                    position: 1.0,
                },
            ],
        }),
        border_radius: Corners::all(6.0),
        ..Style::default()
    };

    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 120.0,
        height: 40.0,
    };

    let cmds = encoder.encode_element(&style, &rect);
    assert_eq!(cmds.len(), 1);
    if let DrawCommand::Rect(prim) = &cmds[0] {
        assert!(prim.flags.contains(PrimitiveFlags::GRADIENT));
        assert!(prim.flags.contains(PrimitiveFlags::ROUNDED));
        let gd = prim.gradient.as_ref().unwrap();
        assert_eq!(gd.stops.len(), 2);
    } else {
        panic!("expected Rect command");
    }
}

#[test]
fn nested_overflow_hidden_clips_children() {
    let mut encoder = CommandEncoder::new();
    let mut all_commands = Vec::new();

    // Parent with overflow: hidden
    let parent_style = Style {
        background: Some(Color::WHITE),
        overflow: Some(Overflow::Hidden),
        ..Style::default()
    };
    let parent_rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 200.0,
    };

    if let Some(clip_cmd) = encoder.maybe_push_clip(&parent_style, &parent_rect) {
        all_commands.push(clip_cmd);
    }
    all_commands.extend(encoder.encode_element(&parent_style, &parent_rect));

    // Child that extends beyond parent
    let child_style = Style {
        background: Some(Color::hex(0xFF0000)),
        ..Style::default()
    };
    let child_rect = Rect {
        x: 150.0,
        y: 150.0,
        width: 200.0,
        height: 200.0,
    };
    all_commands.extend(encoder.encode_element(&child_style, &child_rect));

    // Pop the clip
    all_commands.push(encoder.pop_clip());

    // Should have: PushClip, Rect(parent), Rect(child with clip), PopClip
    assert!(matches!(all_commands[0], DrawCommand::PushClip(_)));
    assert!(matches!(all_commands.last().unwrap(), DrawCommand::PopClip));

    // The child rect should have a clip_rect set
    let child_cmd = &all_commands[2];
    if let DrawCommand::Rect(prim) = child_cmd {
        assert!(prim.clip_rect.is_some());
    } else {
        panic!("expected Rect command for child");
    }
}

#[test]
fn empty_style_no_commands() {
    let mut encoder = CommandEncoder::new();
    let style = Style::default();
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    let cmds = encoder.encode_element(&style, &rect);
    assert!(cmds.is_empty());
}

#[test]
fn border_only_no_background() {
    let mut encoder = CommandEncoder::new();
    let style = Style {
        border_width: Edges::all(2.0),
        border_color: Some(Color::BLACK),
        ..Style::default()
    };
    let rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
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

#[test]
fn shadow_with_spread_and_offset() {
    let mut encoder = CommandEncoder::new();
    let style = Style {
        background: Some(Color::WHITE),
        shadow: Some(vec![BoxShadow {
            offset_x: 10.0,
            offset_y: 20.0,
            blur_radius: 15.0,
            spread_radius: 5.0,
            color: Color::rgba(0.0, 0.0, 0.0, 0.3),
            inset: false,
        }]),
        ..Style::default()
    };
    let rect = Rect {
        x: 50.0,
        y: 50.0,
        width: 200.0,
        height: 100.0,
    };
    let cmds = encoder.encode_element(&style, &rect);
    assert_eq!(cmds.len(), 2);
    if let DrawCommand::Shadow(shadow) = &cmds[0] {
        assert_eq!(shadow.rect.x, 50.0 + 10.0 - 5.0);
        assert_eq!(shadow.rect.y, 50.0 + 20.0 - 5.0);
        assert_eq!(shadow.rect.width, 200.0 + 10.0);
        assert_eq!(shadow.rect.height, 100.0 + 10.0);
        assert_eq!(shadow.blur_radius, 15.0);
    } else {
        panic!("expected Shadow command");
    }
}

#[test]
fn multiple_elements_encode_independently() {
    let mut encoder = CommandEncoder::new();
    let style1 = Style {
        background: Some(Color::WHITE),
        ..Style::default()
    };
    let style2 = Style {
        background: Some(Color::BLACK),
        border_radius: Corners::all(4.0),
        ..Style::default()
    };

    let rect1 = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
    };
    let rect2 = Rect {
        x: 100.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
    };

    let cmds1 = encoder.encode_element(&style1, &rect1);
    let cmds2 = encoder.encode_element(&style2, &rect2);

    assert_eq!(cmds1.len(), 1);
    assert_eq!(cmds2.len(), 1);

    if let DrawCommand::Rect(p1) = &cmds1[0] {
        assert!(!p1.flags.contains(PrimitiveFlags::ROUNDED));
        assert_eq!(p1.fill_color, Color::WHITE);
    } else {
        panic!("expected Rect command");
    }

    if let DrawCommand::Rect(p2) = &cmds2[0] {
        assert!(p2.flags.contains(PrimitiveFlags::ROUNDED));
        assert_eq!(p2.fill_color, Color::BLACK);
    } else {
        panic!("expected Rect command");
    }
}
