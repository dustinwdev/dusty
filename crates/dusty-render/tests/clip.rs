//! Clip stack edge-case tests.

use dusty_render::{ClipRegion, ClipStack, Rect};

#[test]
fn deeply_nested_clips() {
    let mut stack = ClipStack::new();

    // Each nested clip shrinks by 10px on each side
    for i in 0..5 {
        #[allow(clippy::cast_precision_loss)]
        let offset = (i * 10) as f32;
        #[allow(clippy::cast_precision_loss)]
        let size = 200.0 - (i * 20) as f32;
        stack.push(ClipRegion {
            rect: Rect {
                x: offset,
                y: offset,
                width: size,
                height: size,
            },
            radii: [0.0; 4],
        });
    }

    assert_eq!(stack.depth(), 5);

    // Innermost clip should be 40,40 -> 120x120
    let current = stack.current().unwrap();
    assert_eq!(current.rect.x, 40.0);
    assert_eq!(current.rect.y, 40.0);
    assert_eq!(current.rect.width, 120.0);
    assert_eq!(current.rect.height, 120.0);

    // Pop all
    for _ in 0..5 {
        stack.pop();
    }
    assert!(stack.is_empty());
}

#[test]
fn completely_overlapping_clips() {
    let mut stack = ClipStack::new();
    let clip = ClipRegion {
        rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        radii: [0.0; 4],
    };

    stack.push(clip);
    stack.push(clip);

    let current = stack.current().unwrap();
    assert_eq!(current.rect.width, 100.0);
    assert_eq!(current.rect.height, 100.0);
}

#[test]
fn partial_overlap_clips() {
    let mut stack = ClipStack::new();

    stack.push(ClipRegion {
        rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        radii: [0.0; 4],
    });

    stack.push(ClipRegion {
        rect: Rect {
            x: 75.0,
            y: 25.0,
            width: 100.0,
            height: 50.0,
        },
        radii: [0.0; 4],
    });

    let current = stack.current().unwrap();
    assert_eq!(current.rect.x, 75.0);
    assert_eq!(current.rect.y, 25.0);
    assert_eq!(current.rect.width, 25.0);
    assert_eq!(current.rect.height, 50.0);
}

#[test]
fn zero_area_clip_from_disjoint() {
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

    // Completely disjoint
    stack.push(ClipRegion {
        rect: Rect {
            x: 200.0,
            y: 200.0,
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
fn pop_restores_to_previous_level() {
    let mut stack = ClipStack::new();

    stack.push(ClipRegion {
        rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 500.0,
            height: 500.0,
        },
        radii: [0.0; 4],
    });

    stack.push(ClipRegion {
        rect: Rect {
            x: 100.0,
            y: 100.0,
            width: 100.0,
            height: 100.0,
        },
        radii: [0.0; 4],
    });

    // Pop back to first level
    stack.pop();
    let current = stack.current().unwrap();
    assert_eq!(current.rect.width, 500.0);

    // Pop back to empty
    stack.pop();
    assert!(stack.current().is_none());
}

#[test]
fn rounded_clip_radii_propagation() {
    let mut stack = ClipStack::new();

    stack.push(ClipRegion {
        rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        },
        radii: [16.0; 4],
    });

    // Child with its own radii
    stack.push(ClipRegion {
        rect: Rect {
            x: 20.0,
            y: 20.0,
            width: 160.0,
            height: 160.0,
        },
        radii: [8.0; 4],
    });

    let current = stack.current().unwrap();
    assert_eq!(current.radii, [8.0; 4]);

    // Pop back
    stack.pop();
    let current = stack.current().unwrap();
    assert_eq!(current.radii, [16.0; 4]);
}
