#[cfg(feature = "non_ipc_backend")]
mod actix_non_ipc_api;
#[cfg(feature = "non_ipc_backend")]
use actix_non_ipc_api::invoke_backend_api;

#[cfg(not(feature = "non_ipc_backend"))]
mod tauri_ipc_api;
use desktop_email_client_shared::{ApiClient, Request, Response};
#[cfg(not(feature = "non_ipc_backend"))]
use tauri_ipc_api::invoke_backend_api;
#[cfg(not(feature = "non_ipc_backend"))]
pub use tauri_ipc_api::use_db_listen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrontendApiClient;

#[async_trait::async_trait(?Send)]
impl ApiClient for FrontendApiClient {
	async fn dispatch(self, request: Request) -> Response {
		invoke_backend_api(request).await
	}
}
