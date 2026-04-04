use leptos::prelude::*;
use leptos::ev;
use leptos::wasm_bindgen::JsCast;

use crate::api::{self, Todo, UpdateTodo};
use crate::state::TodoState;

#[component]
pub fn TodoItem(todo: Todo, state: TodoState) -> impl IntoView {
    let id = todo.id.clone();
    let title = todo.title.clone();
    let completed = todo.completed;

    let (edit_text, set_edit_text) = signal(String::new());

    let on_toggle = {
        let id = id.clone();
        let state = state.clone();
        move |_| {
            let id = id.clone();
            let new_completed = !completed;
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::update_todo(
                    &id,
                    &UpdateTodo {
                        completed: Some(new_completed),
                        title: None,
                        display_order: None,
                    },
                )
                .await
                {
                    Ok(updated) => {
                        state.set_todos.update(|todos| {
                            if let Some(t) = todos.iter_mut().find(|t| t.id == updated.id) {
                                *t = updated;
                            }
                        });
                    }
                    Err(e) => leptos::logging::error!("toggle failed: {e}"),
                }
            });
        }
    };

    let on_destroy = {
        let id = id.clone();
        let state = state.clone();
        move |_| {
            let id = id.clone();
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::delete_todo(&id).await {
                    Ok(()) => {
                        state.set_todos.update(|todos| {
                            todos.retain(|t| t.id != id);
                        });
                    }
                    Err(e) => leptos::logging::error!("delete failed: {e}"),
                }
            });
        }
    };

    let on_dblclick = {
        let id = id.clone();
        let title = title.clone();
        let state = state.clone();
        move |_| {
            set_edit_text.set(title.clone());
            state.set_editing_id.set(Some(id.clone()));
        }
    };

    let editing = {
        let id = id.clone();
        let state = state.clone();
        move || state.editing_id.get().as_deref() == Some(id.as_str())
    };

    let save_edit = {
        let id = id.clone();
        let state = state.clone();
        move || {
            let trimmed = edit_text.get().trim().to_string();
            let id = id.clone();
            let state = state.clone();
            state.set_editing_id.set(None);
            if trimmed.is_empty() {
                leptos::task::spawn_local(async move {
                    match api::delete_todo(&id).await {
                        Ok(()) => {
                            state.set_todos.update(|todos| {
                                todos.retain(|t| t.id != id);
                            });
                        }
                        Err(e) => leptos::logging::error!("delete failed: {e}"),
                    }
                });
            } else {
                leptos::task::spawn_local(async move {
                    match api::update_todo(
                        &id,
                        &UpdateTodo {
                            title: Some(trimmed),
                            completed: None,
                            display_order: None,
                        },
                    )
                    .await
                    {
                        Ok(updated) => {
                            state.set_todos.update(|todos| {
                                if let Some(t) = todos.iter_mut().find(|t| t.id == updated.id) {
                                    *t = updated;
                                }
                            });
                        }
                        Err(e) => leptos::logging::error!("edit failed: {e}"),
                    }
                });
            }
        }
    };

    let on_edit_keydown = {
        let save_edit = save_edit.clone();
        let state = state.clone();
        move |ev: ev::KeyboardEvent| {
            match ev.key().as_str() {
                "Enter" => save_edit(),
                "Escape" => {
                    state.set_editing_id.set(None);
                }
                _ => {}
            }
        }
    };

    let on_edit_blur = {
        let editing = editing.clone();
        move |_| {
            if editing() {
                save_edit();
            }
        }
    };

    let on_edit_input = move |ev: ev::Event| {
        let target = ev.target().expect("input target");
        let input: leptos::web_sys::HtmlInputElement = target.unchecked_into();
        set_edit_text.set(input.value());
    };

    let li_class = {
        let editing = editing.clone();
        move || {
            let mut classes = Vec::new();
            if completed {
                classes.push("completed");
            }
            if editing() {
                classes.push("editing");
            }
            classes.join(" ")
        }
    };

    view! {
        <li class=li_class>
            <div class="view">
                <input
                    class="toggle"
                    type="checkbox"
                    prop:checked=completed
                    on:change=on_toggle
                />
                <label on:dblclick=on_dblclick>{title}</label>
                <button class="destroy" on:click=on_destroy></button>
            </div>
            <input
                class="edit"
                prop:value=edit_text
                on:input=on_edit_input
                on:blur=on_edit_blur
                on:keydown=on_edit_keydown
            />
        </li>
    }
}
