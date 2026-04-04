pub mod api;
pub mod components;
pub mod state;

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

#[component]
fn App() -> impl IntoView {
    view! {
        <p>"Hello, TodoMVC!"</p>
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
