use leptos::prelude::*;

use crate::api;
use crate::state::{Filter, TodoState};

#[component]
pub fn Footer(state: TodoState) -> impl IntoView {
    let has_todos = {
        let state = state.clone();
        move || !state.todos.get().is_empty()
    };

    let active_count = {
        let state = state.clone();
        move || state.active_count()
    };

    let completed_count = {
        let state = state.clone();
        move || state.completed_count()
    };

    let current_filter = {
        let state = state.clone();
        move || state.filter.get()
    };

    let on_clear_completed = {
        let state = state.clone();
        move |_| {
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::clear_completed().await {
                    Ok(()) => {
                        state.set_todos.update(|todos| {
                            todos.retain(|t| !t.completed);
                        });
                    }
                    Err(e) => leptos::logging::error!("clear completed failed: {e}"),
                }
            });
        }
    };

    let items_left_text = move || {
        let count = active_count();
        if count == 1 {
            format!("{count} item left")
        } else {
            format!("{count} items left")
        }
    };

    view! {
        <footer class="footer" style:display=move || if has_todos() { "" } else { "none" }>
            <span class="todo-count">
                {items_left_text}
            </span>
            <ul class="filters">
                <li>
                    <a
                        href="#/"
                        class:selected=move || current_filter() == Filter::All
                    >"All"</a>
                </li>
                <li>
                    <a
                        href="#/active"
                        class:selected=move || current_filter() == Filter::Active
                    >"Active"</a>
                </li>
                <li>
                    <a
                        href="#/completed"
                        class:selected=move || current_filter() == Filter::Completed
                    >"Completed"</a>
                </li>
            </ul>
            <button
                class="clear-completed"
                style:display=move || { if completed_count() > 0 { "" } else { "none" } }
                on:click=on_clear_completed
            >
                "Clear completed"
            </button>
        </footer>
    }
}
