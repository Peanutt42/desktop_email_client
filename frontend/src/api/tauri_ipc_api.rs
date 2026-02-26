mod tauri_wasm_invoke {
	use web_sys::wasm_bindgen::{self, JsValue, prelude::wasm_bindgen};

	#[wasm_bindgen]
	extern "C" {
		#[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
		pub(crate) async fn invoke(cmd: &str, args: JsValue) -> JsValue;
	}
}

use desktop_email_client_shared::{
	DATABASE_CHANGED_EVENT_NAME, DatabaseChangedEvent, Request, Response,
};
use futures_util::StreamExt;
use yew::prelude::*;

pub async fn invoke_backend_api(request: Request) -> Response {
	let cmd = "tauri_ipc_api_request_handler";

	tracing::debug!("invoking {}", request);

	let args_js_value =
		serde_wasm_bindgen::to_value(&desktop_email_client_shared::TauriCommandArgsWrapper {
			args: request,
		})
		.expect("failed to serialize paramters for tauri command using serde");

	let output_js_value = tauri_wasm_invoke::invoke(cmd, args_js_value).await;

	serde_wasm_bindgen::from_value(output_js_value)
		.expect("failed to deserialize tauri command's output using serde")
}

#[hook]
pub fn use_db_listen(callback: Callback<DatabaseChangedEvent>) {
	use_effect_with((), move |_| {
		wasm_bindgen_futures::spawn_local(async move {
			tracing::debug!(
				"registering event listener for {}",
				DATABASE_CHANGED_EVENT_NAME
			);

			// tauri-sys returns a Stream
			let mut events =
				tauri_sys::event::listen::<DatabaseChangedEvent>(DATABASE_CHANGED_EVENT_NAME)
					.await
					.expect("failed to listen");

			while let Some(event) = events.next().await {
				tracing::debug!("received db changed event: {:?}", event);
				callback.emit(event.payload);
			}
		});
	});
}
