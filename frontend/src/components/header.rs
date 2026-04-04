use leptos::prelude::*;
use leptos::ev;
use leptos::wasm_bindgen::JsCast;

use crate::api;
use crate::state::TodoState;

#[component]
pub fn Header(state: TodoState) -> impl IntoView {
    let (input_value, set_input_value) = signal(String::new());

    let on_keydown = {
        let state = state.clone();
        move |ev: ev::KeyboardEvent| {
            if ev.key() == "Enter" {
                let title = input_value.get().trim().to_string();
                if title.is_empty() {
                    return;
                }
                set_input_value.set(String::new());
                let state = state.clone();
                leptos::task::spawn_local(async move {
                    match api::create_todo(&title).await {
                        Ok(todo) => {
                            state.set_todos.update(|todos| todos.push(todo));
                        }
                        Err(e) => leptos::logging::error!("failed to create todo: {e}"),
                    }
                });
            }
        }
    };

    view! {
        <header class="header">
            <h1>"todos"</h1>
            <input
                class="new-todo"
                placeholder="What needs to be done?"
                autofocus=true
                prop:value=input_value
                on:input=move |ev| {
                    let target = ev.target().expect("input target");
                    let input: leptos::web_sys::HtmlInputElement = target.unchecked_into();
                    set_input_value.set(input.value());
                }
                on:keydown=on_keydown
            />
        </header>
    }
}
