use leptos::prelude::*;

use crate::api::{self, Todo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Filter {
    pub fn matches(&self, todo: &Todo) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !todo.completed,
            Filter::Completed => todo.completed,
        }
    }

    pub fn from_hash(hash: &str) -> Self {
        match hash {
            "#/active" | "/active" => Filter::Active,
            "#/completed" | "/completed" => Filter::Completed,
            _ => Filter::All,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoState {
    pub todos: ReadSignal<Vec<Todo>>,
    pub set_todos: WriteSignal<Vec<Todo>>,
    pub filter: ReadSignal<Filter>,
    pub set_filter: WriteSignal<Filter>,
    pub editing_id: ReadSignal<Option<String>>,
    pub set_editing_id: WriteSignal<Option<String>>,
}

impl TodoState {
    pub fn new() -> Self {
        let (todos, set_todos) = signal(Vec::<Todo>::new());
        let (filter, set_filter) = signal(Filter::All);
        let (editing_id, set_editing_id) = signal(None::<String>);

        TodoState {
            todos,
            set_todos,
            filter,
            set_filter,
            editing_id,
            set_editing_id,
        }
    }

    pub fn filtered_todos(&self) -> Vec<Todo> {
        let filter = self.filter.get();
        self.todos
            .get()
            .into_iter()
            .filter(|t| filter.matches(t))
            .collect()
    }

    pub fn active_count(&self) -> usize {
        self.todos.get().iter().filter(|t| !t.completed).count()
    }

    pub fn completed_count(&self) -> usize {
        self.todos.get().iter().filter(|t| t.completed).count()
    }

    pub fn all_completed(&self) -> bool {
        let todos = self.todos.get();
        !todos.is_empty() && todos.iter().all(|t| t.completed)
    }
}

pub fn load_todos(state: TodoState) {
    leptos::task::spawn_local(async move {
        match api::fetch_todos().await {
            Ok(todos) => state.set_todos.set(todos),
            Err(e) => leptos::logging::error!("failed to load todos: {e}"),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Todo;

    fn make_todo(id: &str, title: &str, completed: bool) -> Todo {
        Todo {
            id: id.to_string(),
            title: title.to_string(),
            completed,
            display_order: 0,
        }
    }

    #[test]
    fn filter_all_matches_everything() {
        let active = make_todo("1", "Active", false);
        let done = make_todo("2", "Done", true);
        assert!(Filter::All.matches(&active));
        assert!(Filter::All.matches(&done));
    }

    #[test]
    fn filter_active_matches_only_incomplete() {
        let active = make_todo("1", "Active", false);
        let done = make_todo("2", "Done", true);
        assert!(Filter::Active.matches(&active));
        assert!(!Filter::Active.matches(&done));
    }

    #[test]
    fn filter_completed_matches_only_completed() {
        let active = make_todo("1", "Active", false);
        let done = make_todo("2", "Done", true);
        assert!(!Filter::Completed.matches(&active));
        assert!(Filter::Completed.matches(&done));
    }

    #[test]
    fn from_hash_parses_all() {
        assert_eq!(Filter::from_hash("#/"), Filter::All);
        assert_eq!(Filter::from_hash(""), Filter::All);
        assert_eq!(Filter::from_hash("garbage"), Filter::All);
    }

    #[test]
    fn from_hash_parses_active() {
        assert_eq!(Filter::from_hash("#/active"), Filter::Active);
        assert_eq!(Filter::from_hash("/active"), Filter::Active);
    }

    #[test]
    fn from_hash_parses_completed() {
        assert_eq!(Filter::from_hash("#/completed"), Filter::Completed);
        assert_eq!(Filter::from_hash("/completed"), Filter::Completed);
    }

    #[test]
    fn filter_applies_to_todo_list() {
        let todos = vec![
            make_todo("1", "Buy milk", false),
            make_todo("2", "Walk dog", true),
            make_todo("3", "Read book", false),
        ];

        let active: Vec<_> = todos.iter().filter(|t| Filter::Active.matches(t)).collect();
        assert_eq!(active.len(), 2);

        let completed: Vec<_> = todos.iter().filter(|t| Filter::Completed.matches(t)).collect();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].title, "Walk dog");

        let all: Vec<_> = todos.iter().filter(|t| Filter::All.matches(t)).collect();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn active_count_logic() {
        let todos = vec![
            make_todo("1", "A", false),
            make_todo("2", "B", true),
            make_todo("3", "C", false),
        ];
        let active = todos.iter().filter(|t| !t.completed).count();
        assert_eq!(active, 2);
    }

    #[test]
    fn completed_count_logic() {
        let todos = vec![
            make_todo("1", "A", false),
            make_todo("2", "B", true),
            make_todo("3", "C", true),
        ];
        let completed = todos.iter().filter(|t| t.completed).count();
        assert_eq!(completed, 2);
    }

    #[test]
    fn all_completed_logic() {
        let empty: Vec<Todo> = vec![];
        assert!(!(!empty.is_empty() && empty.iter().all(|t| t.completed)));

        let all_done = vec![make_todo("1", "A", true), make_todo("2", "B", true)];
        assert!(!all_done.is_empty() && all_done.iter().all(|t| t.completed));

        let mixed = vec![make_todo("1", "A", true), make_todo("2", "B", false)];
        assert!(!(!mixed.is_empty() && mixed.iter().all(|t| t.completed)));
    }

    #[test]
    fn trim_empty_input_handling() {
        let inputs = vec!["  ", "", "\t\n"];
        for input in inputs {
            assert!(input.trim().is_empty());
        }

        assert_eq!("  hello world  ".trim(), "hello world");
        assert_eq!(" single ".trim(), "single");
    }

    #[test]
    fn toggle_all_logic() {
        let mut todos = vec![
            make_todo("1", "A", false),
            make_todo("2", "B", true),
            make_todo("3", "C", false),
        ];

        let all_completed = !todos.is_empty() && todos.iter().all(|t| t.completed);
        let new_state = !all_completed;
        assert!(new_state);

        for t in todos.iter_mut() {
            t.completed = new_state;
        }
        assert!(todos.iter().all(|t| t.completed));

        // Toggle back
        let all_completed = todos.iter().all(|t| t.completed);
        let new_state = !all_completed;
        for t in todos.iter_mut() {
            t.completed = new_state;
        }
        assert!(todos.iter().all(|t| !t.completed));
    }
}

pub fn setup_hash_routing(state: TodoState) {
    use leptos::wasm_bindgen::JsCast;
    use leptos::wasm_bindgen::prelude::Closure;

    let window = leptos::tachys::dom::window();

    // Set initial filter from current hash
    let hash = window.location().hash().unwrap_or_default();
    state.set_filter.set(Filter::from_hash(&hash));

    // Listen for hash changes
    let closure = Closure::<dyn Fn()>::new(move || {
        let hash = leptos::tachys::dom::window()
            .location()
            .hash()
            .unwrap_or_default();
        state.set_filter.set(Filter::from_hash(&hash));
    });

    window
        .add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref())
        .expect("failed to add hashchange listener");

    closure.forget();
}
