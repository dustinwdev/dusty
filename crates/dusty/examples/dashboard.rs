#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Dashboard -- Resource/Suspense, ErrorBoundary, ScrollView, Canvas, complex layout.

use dusty::prelude::*;
use dusty::widgets::canvas::FillStyle;

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
    time: String,
}

fn stat_card(cx: Scope, label: &str, value: &str) -> Node {
    col![cx;
        Text::new(value).build(cx),
        Text::new(label).style(Style {
            foreground: Some(Palette::SLATE.get(500).unwrap()),
            ..Style::default()
        }).build(cx)
    ]
}

fn status_row(cx: Scope, name: &str, ok: bool) -> Node {
    row![cx;
        Text::new(name).build(cx),
        Text::new(if ok { "OK" } else { "DOWN" }).build(cx)
    ]
}

fn main() {
    dusty::app("Dashboard")
        .width(1024.0)
        .height(768.0)
        .root(|cx| {
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
                            description: "New user signed up".into(),
                            time: "2m ago".into(),
                        },
                        Activity {
                            id: 2,
                            description: "Order #1234 placed".into(),
                            time: "5m ago".into(),
                        },
                        Activity {
                            id: 3,
                            description: "Payment received".into(),
                            time: "12m ago".into(),
                        },
                        Activity {
                            id: 4,
                            description: "User updated profile".into(),
                            time: "15m ago".into(),
                        },
                        Activity {
                            id: 5,
                            description: "New comment posted".into(),
                            time: "20m ago".into(),
                        },
                    ]);
                },
            );

            // Build stat cards from resource data
            let stats = stats_resource.get();
            let (users, revenue, orders, active) = stats.map_or(
                ("---".into(), "---".into(), "---".into(), "---".into()),
                |s| {
                    (
                        format!("{}", s.users),
                        format!("${}", s.revenue),
                        format!("{}", s.orders),
                        format!("{}", s.active),
                    )
                },
            );

            let stat_cards = row![cx;
                stat_card(cx, "Users", &users),
                stat_card(cx, "Revenue", &revenue),
                stat_card(cx, "Orders", &orders),
                stat_card(cx, "Active Now", &active)
            ];

            // Activity feed with Suspense
            let activity_for_ready = activity_resource.clone();
            let activity_for_child = activity_resource.clone();

            let activity_feed = Suspense::new(move || activity_for_ready.get().is_some())
                .child(move || {
                    let activities = activity_for_child.get().unwrap_or_default();
                    ScrollView::new()
                        .child(
                            For::<Activity, u32, Node>::new(move || activities.clone())
                                .key(|a: &Activity| a.id)
                                .view(|a: Activity| {
                                    Node::Text(text(format!("[{}] {}", a.time, a.description)))
                                }),
                        )
                        .build(cx)
                })
                .fallback(|| Node::Text(text("Loading activity...")))
                .build(cx);

            // Bar chart via Canvas in an ErrorBoundary
            let chart = ErrorBoundary::new()
                .child(|cx: Scope| {
                    Canvas::new(|frame| {
                        let values = [40.0_f32, 65.0, 30.0, 80.0, 55.0];
                        let colors = [
                            Color::hex(0x3b82f6),
                            Color::hex(0x10b981),
                            Color::hex(0xf59e0b),
                            Color::hex(0xef4444),
                            Color::hex(0x8b5cf6),
                        ];
                        let bar_width = 30.0_f32;
                        let gap = 10.0_f32;
                        for (i, (&val, &color)) in values.iter().zip(colors.iter()).enumerate() {
                            #[allow(clippy::cast_precision_loss)]
                            let x = i as f32 * (bar_width + gap);
                            let height = val;
                            let y = 100.0_f32 - height;
                            frame.rect(
                                x,
                                y,
                                bar_width,
                                height,
                                Some(FillStyle::Solid(color)),
                                None,
                            );
                        }
                    })
                    .style(Style {
                        width: Some(200.0),
                        height: Some(100.0),
                        ..Style::default()
                    })
                    .build(cx)
                })
                .fallback(|msg: String| Node::Text(text(format!("Chart error: {msg}"))))
                .build(cx);

            // Status panel
            let status_panel = col![cx;
                Text::new("System Status").build(cx),
                status_row(cx, "API Server", true),
                status_row(cx, "Database", true),
                status_row(cx, "Cache", true),
                status_row(cx, "Queue", false)
            ];

            // Main layout
            col![cx;
                row![cx;
                    Text::new("Dashboard").build(cx),
                    Spacer::new().build(cx),
                    Button::new("Refresh").on_click(move |_ctx: &EventContext, _e: &ClickEvent| {
                        refresh_trigger.update(|n| *n += 1);
                    }).build(cx)
                ],
                Divider::horizontal().build(cx),
                stat_cards,
                Divider::horizontal().build(cx),
                row![cx;
                    activity_feed,
                    col![cx; chart, status_panel]
                ]
            ]
        })
        .run()
        .unwrap();
}
