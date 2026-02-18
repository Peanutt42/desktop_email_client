use desktop_email_client_shared::{Api, Request, Response};

#[tauri::command(rename_all = "snake_case")]
pub async fn tauri_ipc_api_request_handler(
	backend: tauri::State<'_, crate::Backend>,
	args: Request,
) -> Result<Response, ()> {
	tracing::info!("{}", args);
	let response = backend.dispatch(args).await;
	Ok(response)
}
