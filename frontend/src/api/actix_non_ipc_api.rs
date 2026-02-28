use desktop_email_client_shared::{DEV_NON_IPC_API_ROUTE, DatabaseChangedEvent, Request, Response};
use yew::prelude::*;

pub async fn invoke_backend_api(request: Request) -> Response {
	tracing::debug!("invoking {}", request);

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

#[hook]
pub fn use_db_listen(_callback: impl Fn(DatabaseChangedEvent) + Clone + 'static) {
	// TODO
}
