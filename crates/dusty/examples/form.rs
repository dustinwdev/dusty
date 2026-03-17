#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Form -- interactive widgets, validation with Memo, conditional error display.

use dusty::prelude::*;

fn main() {
    dusty::app("Registration Form")
        .width(500.0)
        .height(700.0)
        .root(|cx| {
            let name = create_signal(String::new());
            let email = create_signal(String::new());
            let experience = create_signal("beginner".to_string());
            let notifications = create_signal(false);
            let satisfaction = create_signal(5.0f64);
            let accept_terms = create_signal(false);
            let submitted = create_signal(false);

            let name_valid = create_memo(move || name.with(|s| !s.is_empty()));

            let email_valid =
                create_memo(move || email.with(|s| s.contains('@') && s.contains('.')));

            let name_valid_for_form = name_valid.clone();
            let email_valid_for_form = email_valid.clone();
            let form_valid = create_memo(move || {
                name_valid_for_form.get() && email_valid_for_form.get() && accept_terms.get()
            });

            let is_valid = form_valid.get();

            let form_valid_for_submit = form_valid.clone();
            let on_submit = move |_ctx: &EventContext, _e: &ClickEvent| {
                if form_valid_for_submit.get() {
                    submitted.set(true);
                }
            };

            // Group fields into sub-containers (ViewSeq max arity = 12)
            let name_field = col![cx;
                Text::new("Name").build(cx),
                TextInput::new()
                    .controlled(name)
                    .placeholder("Your name")
                    .build(cx),
                Show::new(move || {
                    let touched = name.with(|s| !s.is_empty());
                    !touched || !name_valid.get()
                })
                .child(|| Node::Text(text("Name is required")))
                .build(cx)
            ];

            let email_field = col![cx;
                Text::new("Email").build(cx),
                TextInput::new()
                    .controlled(email)
                    .placeholder("you@example.com")
                    .build(cx),
                Show::new(move || {
                    let touched = email.with(|s| !s.is_empty());
                    touched && !email_valid.get()
                })
                .child(|| Node::Text(text("Enter a valid email")))
                .build(cx)
            ];

            let radio_group = col![cx;
                Text::new("Experience Level").build(cx),
                Radio::new("beginner".to_string(), experience).label("Beginner").build(cx),
                Radio::new("intermediate".to_string(), experience).label("Intermediate").build(cx),
                Radio::new("advanced".to_string(), experience).label("Advanced").build(cx)
            ];

            let controls = col![cx;
                Toggle::new().controlled(notifications).label("Email notifications").build(cx),
                Text::new("Satisfaction (0-10)").build(cx),
                Slider::new()
                    .controlled(satisfaction)
                    .min(0.0)
                    .max(10.0)
                    .step(1.0)
                    .build(cx),
                Checkbox::new()
                    .controlled(accept_terms)
                    .label("I accept the terms and conditions")
                    .build(cx)
            ];

            col![cx;
                Text::new("Registration Form").build(cx),
                name_field,
                email_field,
                radio_group,
                Divider::horizontal().build(cx),
                controls,
                Divider::horizontal().build(cx),
                Button::new("Submit")
                    .disabled(!is_valid)
                    .on_click(on_submit)
                    .build(cx),
                Show::new(move || submitted.get())
                    .child(|| Node::Text(text("Form submitted successfully!")))
                    .build(cx)
            ]
        })
        .run()
        .unwrap();
}
