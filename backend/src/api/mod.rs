#[cfg(feature = "non_ipc_backend")]
mod actix_non_ipc_api;
#[cfg(feature = "non_ipc_backend")]
pub use actix_non_ipc_api::configure_actix_backend_api_routes;

#[cfg(not(feature = "non_ipc_backend"))]
mod tauri_ipc_api;
#[cfg(not(feature = "non_ipc_backend"))]
pub use tauri_ipc_api::{__cmd__tauri_ipc_api_request_handler, tauri_ipc_api_request_handler};

pub const DEV_NON_IPC_BACKEND_PORT: u16 = 4242;
pub const DEV_NON_IPC_FRONTEND_PORT: u16 = 8080;
