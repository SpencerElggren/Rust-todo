use seed::prelude::*;
use seed::*;
use wasm_bindgen::__rt::std::collections::BTreeMap;
use ulid::Ulid;
use std::iter::Iterator;
use seed::virtual_dom::UpdateEl;
use wasm_bindgen::__rt::std::convert::TryFrom;

const ENTER_KEY: &str = "Enter";
const ESCAPE_KEY: &str = "Escape";



#[derive(Default)]
struct Model {
    new_title: String,
    todos_list: BTreeMap<Ulid, Todo>,
    selected_item: Option<Selected>,
}

struct Todo {
    id: Ulid,
    title: String,
    complete: bool
}

struct Selected {
    id: Ulid,
    title: String,
    input_element: ElRef<web_sys::HtmlInputElement>
}

enum Msg {
    NewItemTitle(String),
    CreateItem,
    ClearAll,
    ToggleItem(Ulid),
    RemoveItem(Ulid),
    SelectItem(Option<Ulid>),
    SelectTitleChange(String),
    SaveSelectedTodo,
}

fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    use Msg::*;

    match msg {
        NewItemTitle(title) => model.new_title = title,
        CreateItem => {
            let title = model.new_title.trim();
            if not(title.is_empty()) {
                let id = Ulid::new();
                model.todos_list.insert(
                    id,
                    Todo {
                        id,
                        title: title.to_owned(),
                        complete: false
                    }
                );
                model.new_title.clear();
            }
        },
        ToggleItem(id) => {
            if let Some(todo) = model.todos_list.get_mut(&id) {
                todo.complete = not(todo.complete)
            }
        },
        RemoveItem(id) => {
            model.todos_list.remove(&id);
        },
        ClearAll => model.todos_list.clear(),
        SelectItem(Some(id)) => {
            if let Some(todo) = model.todos_list.get(&id) {
                let input_element = ElRef::new();
                model.selected_item = Some(Selected {
                    id,
                    title: todo.title.clone(),
                    input_element: input_element.clone()
                });
                let title_length = u32::try_from(todo.title.len()).expect("title length as u32");
                _orders.after_next_render(move |_| {
                    let input_element = input_element.get().expect("input element");
                    input_element.focus().expect("focus input element");
                    input_element.set_selection_range(title_length, title_length).expect("Cursor to end")

                });
            }
        }
        SelectItem(None) => model.selected_item = None,
        SelectTitleChange(title) => {
            if let Some(selected_item) = &mut model.selected_item {
                selected_item.title = title
            }
        }
        SaveSelectedTodo => {
            if let Some(selected_item) = model.selected_item.take() {
                if let Some(todo) = model.todos_list.get_mut(&selected_item.id) {
                    todo.title = selected_item.title;
                }
            }
        }
    }
}

fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![
        view_head(&model.new_title),
        IF!(not(&model.todos_list.is_empty()) => vec![
            view_list(&model.todos_list, model.selected_item.as_ref())
        ])
    ]
}

fn view_head(new_title: &str) -> Node<Msg> {
    header![
        C!["header"],
        h1!["todos"],
        input![
            C!["new-todo"],
            attrs! {
                At::Placeholder => "What needs to be done?",
                At::AutoFocus => AtValue::None,
                At::Value => new_title,
            },
            input_ev(Ev::Input, Msg::NewItemTitle),
            keyboard_ev(Ev::KeyDown, |keyboard_event| {
                IF!(keyboard_event.key() == ENTER_KEY => Msg::CreateItem)
            }),
        ],
        button![C!["clear"], "Clear All",
            ev(Ev::Click, move |_| Msg::ClearAll)
        ]
    ]
}

fn view_list(todos: &BTreeMap<Ulid, Todo>, selected: Option<&Selected>) -> Node<Msg> {
    let todos = todos.values();
    div![C!["todo-section"],
        ul![C!["todo-list"],
            todos.map(|todo| {
                let id = todo.id;
                let is_selected = Some(id) == selected.map(|selected| selected.id);
                li![C![IF!(todo.complete => "completed"), IF!(is_selected => "editing")],
                el_key(&todo.id),
                div![C!["view"],
                    input![C!["toggle"],
                        attrs!{At::Type => "checkbox", At::Checked => todo.complete.as_at_value()},
                        ev(Ev::Change, move |_| Msg::ToggleItem(id)),
                    ],
                    label![
                        &todo.title,
                        ev(Ev::DblClick, move |_| Msg::SelectItem(Some(id))),
                    ],
                    button![C!["destroy"], "X",
                        ev(Ev::Click, move |_| Msg::RemoveItem(id))
                    ],
                ],
                IF!(is_selected => {
                    let selected_todo = selected.unwrap();
                    input![C!["edit"],
                        el_ref(&selected_todo.input_element),
                        attrs!{At::Value => selected_todo.title},
                        input_ev(Ev::Input, Msg::SelectTitleChange),
                        keyboard_ev(Ev::KeyDown, |keyboard_event| {
                            Some(match keyboard_event.key().as_str() {
                                ESCAPE_KEY => Msg::SelectItem(None),
                                ENTER_KEY => Msg::SaveSelectedTodo,
                                _ => return None
                            })
                        }),
                        ev(Ev::Blur, |_| Msg::SaveSelectedTodo),
                    ]
                }),
            ]
            })
        ]
    ]
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model::default()
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
