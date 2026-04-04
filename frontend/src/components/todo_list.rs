use leptos::prelude::*;

use crate::api;
use crate::components::todo_item::TodoItem;
use crate::state::TodoState;

#[component]
pub fn TodoList(state: TodoState) -> impl IntoView {
    let on_toggle_all = {
        let state = state.clone();
        move |_| {
            let state = state.clone();
            let new_completed = !state.all_completed();
            leptos::task::spawn_local(async move {
                match api::toggle_all(new_completed).await {
                    Ok(()) => {
                        state.set_todos.update(|todos| {
                            for todo in todos.iter_mut() {
                                todo.completed = new_completed;
                            }
                        });
                    }
                    Err(e) => leptos::logging::error!("toggle all failed: {e}"),
                }
            });
        }
    };

    let has_todos = {
        let state = state.clone();
        move || !state.todos.get().is_empty()
    };

    let all_completed = {
        let state = state.clone();
        move || state.all_completed()
    };

    let filtered = {
        let state = state.clone();
        move || state.filtered_todos()
    };

    view! {
        <section class="main" style:display=move || if has_todos() { "" } else { "none" }>
            <input
                id="toggle-all"
                class="toggle-all"
                type="checkbox"
                prop:checked=all_completed
                on:change=on_toggle_all
            />
            <label for="toggle-all">"Mark all as complete"</label>
            <ul class="todo-list">
                <For
                    each=filtered
                    key=|todo| todo.id.clone()
                    let:todo
                >
                    <TodoItem todo=todo state=state.clone() />
                </For>
            </ul>
        </section>
    }
}
