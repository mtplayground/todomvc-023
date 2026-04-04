# TodoMVC - Rust (Leptos + Axum)

A full-stack [TodoMVC](https://todomvc.com) implementation using Rust, with a [Leptos](https://leptos.dev) CSR frontend and an [Axum](https://github.com/tokio-rs/axum) backend backed by SQLite.

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

## Project Structure

```
Cargo.toml                  # Workspace: frontend + server
rust-toolchain.toml         # Stable toolchain + wasm32 target
frontend/
  Cargo.toml
  Trunk.toml                # Trunk build config
  index.html                # HTML entry point with TodoMVC CSS
  src/
    main.rs                 # App component, mounts to DOM
    api.rs                  # HTTP client for REST API
    state.rs                # Reactive signals and filter logic
    components/
      header.rs             # New todo input
      todo_item.rs          # Individual todo (view + edit)
      todo_list.rs          # Filtered list + toggle-all
      footer.rs             # Item count, filters, clear completed
server/
  Cargo.toml
  src/
    main.rs                 # Axum server entry point
    lib.rs                  # Library exports for tests
    db.rs                   # SQLite pool and migrations
    models.rs               # Todo struct and API types
    routes.rs               # REST API handlers
  tests/
    api_tests.rs            # Integration tests
migrations/
  20240101000000_create_todos.sql
```

## Development

### Build and run the frontend (dev mode)

```bash
trunk serve frontend/index.html
```

This starts a dev server at `http://localhost:8080` with hot-reload.

### Build and run the server

```bash
# Build the frontend for production
trunk build frontend/index.html --release

# Run the server (serves frontend + API)
cargo run -p server
```

The server listens on `http://0.0.0.0:8080`.

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite:todos.db` | SQLite database path |
| `FRONTEND_DIST` | `frontend/dist` | Path to built frontend assets |
| `RUST_LOG` | (none) | Logging level (e.g. `info`, `debug`) |

## API Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/health` | Health check |
| `GET` | `/api/todos` | List all todos |
| `POST` | `/api/todos` | Create a todo |
| `PATCH` | `/api/todos` | Toggle all todos (bulk) |
| `PATCH` | `/api/todos/:id` | Update a todo |
| `DELETE` | `/api/todos/:id` | Delete a todo |
| `DELETE` | `/api/todos/completed` | Clear completed todos |

## Testing

```bash
# Run server integration tests
cargo test -p server

# Run frontend unit tests
cargo test -p frontend
```

## License

MIT
