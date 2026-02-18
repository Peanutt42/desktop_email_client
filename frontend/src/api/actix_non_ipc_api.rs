use desktop_email_client_shared::{DEV_NON_IPC_API_ROUTE, Request, Response};

pub async fn invoke_backend_api(request: Request) -> Response {
	tracing::info!("invoking {}", request);

	gloo_net::http::Request::post(DEV_NON_IPC_API_ROUTE)
		.credentials(web_sys::RequestCredentials::Include)
		.json(&request)
		.expect("failed to serialize args to backend api")
		.send()
		.await
		.expect("failed to send backend api request")
		.json()
		.await
		.expect("failed to deserialize response of backend api")
}
