mod tauri_wasm_invoke {
	use web_sys::wasm_bindgen::{self, JsValue, prelude::wasm_bindgen};

	#[wasm_bindgen]
	extern "C" {
		#[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
		pub(crate) async fn invoke(cmd: &str, args: JsValue) -> JsValue;
	}
}

use desktop_email_client_shared::{Request, Response};

pub async fn invoke_backend_api(request: Request) -> Response {
	let cmd = "tauri_ipc_api_request_handler";

	tracing::info!("invoking {}", request);

	let args_js_value =
		serde_wasm_bindgen::to_value(&desktop_email_client_shared::TauriCommandArgsWrapper {
			args: request,
		})
		.expect("failed to serialize paramters for tauri command using serde");

	let output_js_value = tauri_wasm_invoke::invoke(cmd, args_js_value).await;

	serde_wasm_bindgen::from_value(output_js_value)
		.expect("failed to deserialize tauri command's output using serde")
}
