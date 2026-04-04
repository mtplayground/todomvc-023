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
