use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <p>"Hello, TodoMVC!"</p>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
