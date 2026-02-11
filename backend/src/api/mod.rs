#[cfg(feature = "non_ipc_backend")]
mod actix_non_ipc_api;
#[cfg(feature = "non_ipc_backend")]
pub use actix_non_ipc_api::configure_backend;

#[cfg(not(feature = "non_ipc_backend"))]
mod tauri_ipc_api;
