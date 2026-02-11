#[macro_export]
macro_rules! generate_invoke_backend_api_function {
	($fn_name:ident, $request_type:ty) => {
		pub async fn $fn_name(
			args: <$request_type as desktop_email_client_shared::ApiRequest>::Args,
		) -> <$request_type as desktop_email_client_shared::ApiRequest>::Output {
			gloo_net::http::Request::post(&format!(
				"/non_ipc_api/{}",
				<$request_type as desktop_email_client_shared::ApiRequest>::NAME
			))
			.credentials(web_sys::RequestCredentials::Include)
			.json(&args)
			.expect("failed to serialize args to backend api")
			.send()
			.await
			.expect("failed to send backend api request")
			.json()
			.await
			.expect("failed to deserialize response of backend api")
		}
	};
}
