#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Todo app — list management, input, state, For/Show containers, Memo.

use dusty::prelude::*;

#[derive(Clone)]
struct Todo {
    id: u32,
    title: String,
    completed: Signal<bool>,
}

fn main() {
    dusty::app("Todos")
        .width(600.0)
        .height(500.0)
        .root(|cx| {
            let todos = create_signal::<Vec<Todo>>(vec![]).unwrap();
            let input_text = create_signal(String::new()).unwrap();
            let next_id = create_signal(1u32).unwrap();

            let active_count = create_memo(move || {
                todos
                    .with(|ts| {
                        ts.iter()
                            .filter(|t| !t.completed.get().unwrap_or(false))
                            .count()
                    })
                    .unwrap_or(0)
            })
            .unwrap();

            let add = {
                let input_text = input_text;
                let todos = todos;
                let next_id = next_id;
                move |_submitted_text: &str| {
                    let title = input_text.get().unwrap();
                    if title.is_empty() {
                        return;
                    }
                    let id = next_id.get().unwrap();
                    let completed = create_signal(false).unwrap();
                    todos
                        .update(|ts| {
                            ts.push(Todo {
                                id,
                                title: title.clone(),
                                completed,
                            });
                        })
                        .unwrap();
                    next_id.update(|n| *n += 1).unwrap();
                    input_text.set(String::new()).unwrap();
                }
            };

            let clear = {
                let todos = todos;
                move |_ctx: &EventContext, _e: &ClickEvent| {
                    todos
                        .update(|ts| ts.retain(|t| !t.completed.get().unwrap_or(false)))
                        .unwrap();
                }
            };

            col![cx;
                Text::new("Todos").build(cx),
                row![cx;
                    TextInput::new()
                        .controlled(input_text)
                        .placeholder("What needs to be done?")
                        .on_submit(add)
                        .build(cx),
                    Button::new("Add").build(cx)
                ],
                Divider::horizontal().build(cx),
                Show::new(move || todos.with(|ts| !ts.is_empty()).unwrap_or(false))
                    .child(move || {
                        For::<Todo, u32, Node>::new(move || todos.get().unwrap())
                            .key(|t: &Todo| t.id)
                            .view(move |t: Todo| {
                                let completed = t.completed;
                                Node::Component(ComponentNode {
                                    name: "TodoItem",
                                    child: Box::new(
                                        row![cx;
                                            Checkbox::new()
                                                .controlled(completed)
                                                .build(cx),
                                            Text::new(t.title).build(cx)
                                        ],
                                    ),
                                })
                            })
                            .build(cx)
                    })
                    .fallback(|| Node::Text(text("No todos yet")))
                    .build(cx),
                Divider::horizontal().build(cx),
                row![cx;
                    Text::dynamic(move || {
                        format!("{} item(s) active", active_count.get().unwrap_or(0))
                    }).build(cx),
                    Button::new("Clear completed").on_click(clear).build(cx)
                ]
            ]
        })
        .run()
        .unwrap();
}
