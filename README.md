# desktop_email_client

Simple, modern looking desktop email client

### Structure

`frontend` (desktop_email_client_frontend):
Rust WASM frontend application using Yew

`backend` (desktop_email_client_backend):
Rust backend actix server that serves all the routes
(also starts the tauri webview window)

`root` (desktop_email_client):
The root crate has all shared code between `frontend` and `backend`
