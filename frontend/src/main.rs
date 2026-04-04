pub mod api;
pub mod components;
pub mod state;

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use components::footer::Footer;
use components::header::Header;
use components::todo_list::TodoList;
use state::{load_todos, setup_hash_routing, TodoState};

#[component]
fn App() -> impl IntoView {
    let state = TodoState::new();

    load_todos(state.clone());
    setup_hash_routing(state.clone());

    view! {
        <Header state=state.clone() />
        <TodoList state=state.clone() />
        <Footer state=state.clone() />
    }
}

fn main() {
    console_error_panic_hook::set_once();
    let root = leptos::tachys::dom::document()
        .query_selector(".todoapp")
        .expect("failed to query .todoapp")
        .expect(".todoapp element not found")
        .unchecked_into::<leptos::web_sys::HtmlElement>();
    leptos::mount::mount_to(root, App).forget();
}
