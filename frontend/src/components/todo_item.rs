use leptos::prelude::*;

use crate::api::{self, Todo, UpdateTodo};
use crate::state::TodoState;

#[component]
pub fn TodoItem(todo: Todo, state: TodoState) -> impl IntoView {
    let id = todo.id.clone();
    let title = todo.title.clone();
    let completed = todo.completed;

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
        let state = state.clone();
        move |_| {
            state.set_editing_id.set(Some(id.clone()));
        }
    };

    let editing = {
        let id = id.clone();
        let state = state.clone();
        move || state.editing_id.get().as_deref() == Some(id.as_str())
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
        </li>
    }
}
