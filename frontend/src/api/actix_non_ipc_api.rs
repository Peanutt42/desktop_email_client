use desktop_email_client_shared::{
	DATABASE_CHANGED_EVENT_NAME, DEV_NON_IPC_API_ROUTE, DatabaseChangedEvent, Request, Response,
};
use gloo::events::EventListener;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::wasm_bindgen::JsCast;
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
pub fn use_db_listen(callback: impl Fn(DatabaseChangedEvent) + Clone + 'static) {
	let callback_ref = use_memo((), |_| Rc::new(RefCell::new(callback.clone())));
	{
		let callback_ref = callback_ref.clone();
		use_effect(move || {
			*callback_ref.borrow_mut() = callback.clone();
		});
	}
	{
		let callback_ref = callback_ref.clone();
		use_effect_with((), move |_| {
			let active = Rc::new(RefCell::new(true));
			let active_inner = active.clone();

			let api_route = format!("{}/{}", DEV_NON_IPC_API_ROUTE, DATABASE_CHANGED_EVENT_NAME);
			let event_source =
				web_sys::EventSource::new(&api_route).expect("failed to connect to SSE endpoint");

			let listener = EventListener::new(&event_source, "message", move |event| {
				if !*active_inner.borrow() {
					return;
				}
				let event = event.dyn_ref::<web_sys::MessageEvent>().unwrap();
				let data = event.data().as_string().expect("SSE data is not a string");
				match serde_json::from_str::<DatabaseChangedEvent>(&data) {
					Ok(db_event) => (callback_ref.borrow())(db_event),
					Err(e) => tracing::error!("failed to parse SSE event: {}", e),
				}
			});

			move || {
				*active.borrow_mut() = false;
				drop(listener); // cleanup is automatic
				event_source.close();
			}
		});
	}
}
