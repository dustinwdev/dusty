#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Counter — minimal signal usage with reactive text and button handlers.

use dusty::prelude::*;

fn main() {
    dusty::app("Counter")
        .width(400.0)
        .height(200.0)
        .root(|cx| {
            let count = create_signal(0i32).unwrap();

            let decrement = {
                let count = count;
                move |_ctx: &EventContext, _e: &ClickEvent| {
                    count.update(|n| *n -= 1).unwrap();
                }
            };
            let reset = {
                let count = count;
                move |_ctx: &EventContext, _e: &ClickEvent| {
                    count.set(0).unwrap();
                }
            };
            let increment = {
                let count = count;
                move |_ctx: &EventContext, _e: &ClickEvent| {
                    count.update(|n| *n += 1).unwrap();
                }
            };

            col![cx;
                Text::dynamic(move || format!("Count: {}", count.get().unwrap())).build(cx),
                row![cx;
                    Button::new("-1").on_click(decrement).build(cx),
                    Button::new("Reset")
                        .variant(ButtonVariant::Secondary)
                        .on_click(reset)
                        .build(cx),
                    Button::new("+1").on_click(increment).build(cx)
                ]
            ]
        })
        .run()
        .unwrap();
}
