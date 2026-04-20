//! Integration tests that verify each example's root-building logic
//! produces a valid node tree.

use dusty::prelude::*;
use dusty_reactive::{create_scope, dispose_runtime, initialize_runtime};

fn with_scope(f: impl FnOnce(Scope)) {
    initialize_runtime();
    create_scope(|cx| f(cx));
    dispose_runtime();
}

fn assert_is_col(node: &Node) {
    match node {
        Node::Element(el) => {
            assert_eq!(el.name(), "Col");
        }
        _ => panic!("expected Element(Col) node"),
    }
}

// ---------------------------------------------------------------------------
// Counter
// ---------------------------------------------------------------------------

#[test]
fn counter_example_builds_valid_tree() {
    with_scope(|cx| {
        let count = create_signal(0i32);

        let decrement = {
            let count = count;
            move |_ctx: &EventContext, _e: &ClickEvent| {
                count.update(|n| *n -= 1);
            }
        };
        let reset = {
            let count = count;
            move |_ctx: &EventContext, _e: &ClickEvent| {
                count.set(0);
            }
        };
        let increment = {
            let count = count;
            move |_ctx: &EventContext, _e: &ClickEvent| {
                count.update(|n| *n += 1);
            }
        };

        let root = col![cx;
            Text::dynamic(move || format!("Count: {}", count.get())).build(cx),
            row![cx;
                Button::new("-1").on_click(decrement).build(cx),
                Button::new("Reset").variant(ButtonVariant::Secondary).on_click(reset).build(cx),
                Button::new("+1").on_click(increment).build(cx)
            ]
        ];

        assert_is_col(&root);
        let children = root.children();
        assert_eq!(children.len(), 2);
        assert!(children[0].is_component());
        assert!(children[1].is_element());
    });
}

// ---------------------------------------------------------------------------
// Todo
// ---------------------------------------------------------------------------

#[test]
fn todo_example_builds_valid_tree() {
    with_scope(|cx| {
        #[derive(Clone)]
        struct Todo {
            id: u32,
            title: String,
            completed: Signal<bool>,
        }

        let todos = create_signal::<Vec<Todo>>(vec![]);
        let input_text = create_signal(String::new());
        let next_id = create_signal(1u32);

        let active_count =
            create_memo(move || todos.with(|ts| ts.iter().filter(|t| !t.completed.get()).count()));

        let add = {
            let input_text = input_text;
            let todos = todos;
            let next_id = next_id;
            move |_text: &str| {
                let title = input_text.get();
                if title.is_empty() {
                    return;
                }
                let id = next_id.get();
                let completed = create_signal(false);
                todos.update(|ts| {
                    ts.push(Todo {
                        id,
                        title: title.clone(),
                        completed,
                    });
                });
                next_id.update(|n| *n += 1);
                input_text.set(String::new());
            }
        };

        let root = col![cx;
            Text::new("Todos").build(cx),
            row![cx;
                TextInput::new().controlled(input_text).on_submit(add).build(cx),
                Button::new("Add").build(cx)
            ],
            Divider::horizontal().build(cx),
            Show::new(move || todos.with(|ts| !ts.is_empty()))
                .child(move || {
                    For::<Todo, u32, Node>::new(move || todos.get())
                        .key(|t: &Todo| t.id)
                        .view(|t: Todo| Node::Text(text(t.title)))
                        .build(cx)
                })
                .fallback(|| Node::Text(text("No todos yet")))
                .build(cx),
            Divider::horizontal().build(cx),
            row![cx;
                Text::dynamic(move || format!("{} active", active_count.get())).build(cx),
                Button::new("Clear completed").build(cx)
            ]
        ];

        assert_is_col(&root);
        let children = root.children();
        assert_eq!(children.len(), 6);
    });
}

// ---------------------------------------------------------------------------
// Form
// ---------------------------------------------------------------------------

#[test]
fn form_example_builds_valid_tree() {
    with_scope(|cx| {
        let name = create_signal(String::new());
        let email = create_signal(String::new());
        let experience = create_signal("beginner".to_string());
        let notifications = create_signal(false);
        let satisfaction = create_signal(5.0);
        let accept_terms = create_signal(false);
        let submitted = create_signal(false);

        let form_valid = create_memo(move || {
            let n = name.with(|s| !s.is_empty());
            let e = email.with(|s| s.contains('@'));
            let a = accept_terms.get();
            n && e && a
        });

        // Group fields into sub-containers to stay within tuple arity limit
        let name_field = col![cx;
            Text::new("Name").build(cx),
            TextInput::new().controlled(name).placeholder("Your name").build(cx)
        ];

        let email_field = col![cx;
            Text::new("Email").build(cx),
            TextInput::new().controlled(email).placeholder("you@example.com").build(cx)
        ];

        let radio_group = col![cx;
            Text::new("Experience").build(cx),
            Radio::new("beginner".to_string(), experience).label("Beginner").build(cx),
            Radio::new("intermediate".to_string(), experience).label("Intermediate").build(cx),
            Radio::new("advanced".to_string(), experience).label("Advanced").build(cx)
        ];

        let controls = col![cx;
            Toggle::new().controlled(notifications).label("Email notifications").build(cx),
            Text::new("Satisfaction").build(cx),
            Slider::new().controlled(satisfaction).min(0.0).max(10.0).step(1.0).build(cx),
            Checkbox::new().controlled(accept_terms).label("I accept the terms").build(cx)
        ];

        let root = col![cx;
            Text::new("Registration Form").build(cx),
            name_field,
            email_field,
            radio_group,
            controls,
            Button::new("Submit").disabled(!form_valid.get()).build(cx),
            Show::new(move || submitted.get())
                .child(|| Node::Text(text("Form submitted successfully!")))
                .build(cx)
        ];

        assert_is_col(&root);
        let children = root.children();
        assert_eq!(children.len(), 7);
    });
}

// ---------------------------------------------------------------------------
// Theme Showcase
// ---------------------------------------------------------------------------

#[test]
fn theme_showcase_example_builds_valid_tree() {
    with_scope(|cx| {
        let dark_mode = create_signal(false);

        let _theme = if dark_mode.get() {
            Theme::dark()
        } else {
            Theme::light()
        };

        let swatch = |color: Color| -> Node {
            el("Swatch", cx)
                .style(Style {
                    background: Some(color),
                    width: Some(Length::Px(32.0)),
                    height: Some(Length::Px(32.0)),
                    border_radius: Corners::all(4.0),
                    ..Style::default()
                })
                .build_node()
        };

        let root = col![cx;
            row![cx;
                Text::new("Theme Showcase").build(cx),
                Toggle::new().controlled(dark_mode).label("Dark Mode").build(cx)
            ],
            Divider::horizontal().build(cx),
            Text::new("Color Palette").build(cx),
            row![cx;
                swatch(Palette::BLUE.get(500).unwrap_or(Color::BLACK)),
                swatch(Palette::RED.get(500).unwrap_or(Color::BLACK)),
                swatch(Palette::GREEN.get(500).unwrap_or(Color::BLACK)),
                swatch(Palette::AMBER.get(500).unwrap_or(Color::BLACK)),
                swatch(Palette::VIOLET.get(500).unwrap_or(Color::BLACK))
            ],
            Text::new("Button Variants").build(cx),
            row![cx;
                Button::new("Primary").build(cx),
                Button::new("Secondary").variant(ButtonVariant::Secondary).build(cx),
                Button::new("Outline").variant(ButtonVariant::Outline).build(cx),
                Button::new("Ghost").variant(ButtonVariant::Ghost).build(cx),
                Button::new("Danger").variant(ButtonVariant::Danger).build(cx),
                Button::new("Disabled").disabled(true).build(cx)
            ],
            Text::new("Typography").build(cx),
            Text::new("Spacing Tokens").build(cx)
        ];

        assert_is_col(&root);
        let children = root.children();
        assert_eq!(children.len(), 8);
    });
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

#[test]
fn dashboard_example_builds_valid_tree() {
    with_scope(|cx| {
        #[derive(Clone, PartialEq)]
        struct DashboardStats {
            users: u32,
            revenue: u32,
            orders: u32,
            active: u32,
        }

        #[derive(Clone, PartialEq)]
        struct Activity {
            id: u32,
            description: String,
        }

        let refresh_trigger = create_signal(0u32);

        let stats_resource = create_resource(
            move || refresh_trigger.get(),
            |_trigger: u32, resolver: ResourceResolver<DashboardStats>| {
                resolver.resolve(DashboardStats {
                    users: 1234,
                    revenue: 56789,
                    orders: 890,
                    active: 42,
                });
            },
        );

        let activity_resource: Resource<Vec<Activity>> = create_resource(
            move || refresh_trigger.get(),
            |_trigger: u32, resolver: ResourceResolver<Vec<Activity>>| {
                resolver.resolve(vec![
                    Activity {
                        id: 1,
                        description: "User signed up".to_string(),
                    },
                    Activity {
                        id: 2,
                        description: "Order placed".to_string(),
                    },
                    Activity {
                        id: 3,
                        description: "Payment received".to_string(),
                    },
                ]);
            },
        );

        let stat_card = |label: &str, value: &str| -> Node {
            col![cx;
                Text::new(value).build(cx),
                Text::new(label).build(cx)
            ]
        };

        let stats = stats_resource.get();
        let (users, revenue, orders, active) = stats.map_or(
            (
                "---".to_string(),
                "---".to_string(),
                "---".to_string(),
                "---".to_string(),
            ),
            |s| {
                (
                    format!("{}", s.users),
                    format!("${}", s.revenue),
                    format!("{}", s.orders),
                    format!("{}", s.active),
                )
            },
        );

        let activity_for_ready = activity_resource.clone();
        let activity_for_child = activity_resource.clone();

        let root = col![cx;
            row![cx;
                Text::new("Dashboard").build(cx),
                Button::new("Refresh").on_click(move |_ctx: &EventContext, _e: &ClickEvent| {
                    refresh_trigger.update(|n| *n += 1);
                }).build(cx)
            ],
            Divider::horizontal().build(cx),
            row![cx;
                stat_card("Users", &users),
                stat_card("Revenue", &revenue),
                stat_card("Orders", &orders),
                stat_card("Active", &active)
            ],
            Suspense::new(move || activity_for_ready.get().is_some())
                .child(move || {
                    let activities = activity_for_child.get().unwrap_or_default();
                    ScrollView::new()
                        .child(
                            For::<Activity, u32, Node>::new(move || activities.clone())
                                .key(|a: &Activity| a.id)
                                .view(|a: Activity| Node::Text(text(a.description)))
                        )
                        .build(cx)
                })
                .fallback(|| Node::Text(text("Loading...")))
                .build(cx),
            ErrorBoundary::new()
                .child(|cx: Scope| {
                    Canvas::new(|frame| {
                        frame.rect(
                            0.0, 0.0, 100.0, 50.0,
                            Some(dusty::widgets::canvas::FillStyle::Solid(Color::hex(0x3b82f6))),
                            None,
                        );
                    })
                    .style(Style {
                        width: Some(Length::Px(200.0)),
                        height: Some(Length::Px(100.0)),
                        ..Style::default()
                    })
                    .build(cx)
                })
                .fallback(|msg: String| Node::Text(text(msg)))
                .build(cx)
        ];

        assert_is_col(&root);
        let children = root.children();
        assert_eq!(children.len(), 5);
    });
}
