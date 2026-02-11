pub(crate) mod tauri_wasm_invoke {
	use web_sys::wasm_bindgen::{self, JsValue, prelude::wasm_bindgen};

	#[wasm_bindgen]
	extern "C" {
		#[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
		pub(crate) async fn invoke(cmd: &str, args: JsValue) -> JsValue;

		#[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
		pub(crate) async fn invoke_without_args(cmd: &str) -> JsValue;
	}
}

#[macro_export]
macro_rules! generate_invoke_backend_api_function {
	($fn_name:ident, $request_type:ty) => {
		pub async fn $fn_name(
			args: <$request_type as desktop_email_client_shared::ApiRequest>::Args,
		) -> <$request_type as desktop_email_client_shared::ApiRequest>::Output {
			let cmd = <$request_type as desktop_email_client_shared::ApiRequest>::NAME;

			let unit_args = std::any::TypeId::of::<
				<$request_type as desktop_email_client_shared::ApiRequest>::Args,
			>() == std::any::TypeId::of::<()>();

			let output_js_value = if unit_args {
				tracing::info!("{}()", cmd);

				$crate::api::tauri_wasm_invoke::invoke_without_args(cmd).await
			} else {
				tracing::info!("{}({:?})", cmd, args);

				let args_js_value = serde_wasm_bindgen::to_value(
					&desktop_email_client_shared::TauriCommandArgsWrapper { args },
				)
				.expect("failed to serialize paramters for tauri command using serde");

				$crate::api::tauri_wasm_invoke::invoke(cmd, args_js_value).await
			};

			serde_wasm_bindgen::from_value(output_js_value)
				.expect("failed to deserialize tauri command's output using serde")
		}
	};
}
