use desktop_email_client_shared::{
	DATABASE_CHANGED_EVENT_NAME, DatabaseChangedEvent, Request, Response, TauriCommandArgsWrapper,
};
use futures_util::{
	StreamExt,
	future::{AbortHandle, Abortable},
};
use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;

pub async fn invoke_backend_api(request: Request) -> Response {
	let cmd = "tauri_ipc_api_request_handler";

	tracing::debug!("invoking {}", request);

	tauri_sys::core::invoke(cmd, TauriCommandArgsWrapper { args: request }).await
}

/// Important: this listens to db changes, the listener does not get remounted on rerender.
#[hook]
pub fn use_db_listen(callback: impl Fn(DatabaseChangedEvent) + Clone + 'static) {
	// reference to the callback used by the listener
	let callback_ref = use_memo((), |_| Rc::new(RefCell::new(callback.clone())));

	// sets the new callback for the listener
	{
		let callback_ref = callback_ref.clone();
		use_effect(move || {
			*callback_ref.borrow_mut() = callback.clone();
		});
	}

	// this effect will only run once, not again on remount, as doing so would mean that reacting
	// to db changes results in the db listener having to restart, missing important db change
	// events
	{
		let callback_ref = callback_ref.clone();
		use_effect_with((), move |_| {
			let active = Rc::new(RefCell::new(true));
			let active_inner = active.clone();
			let (abort_handle, abort_registration) = AbortHandle::new_pair();

			wasm_bindgen_futures::spawn_local(async move {
				let fut = async move {
					let mut events = tauri_sys::event::listen::<DatabaseChangedEvent>(
						DATABASE_CHANGED_EVENT_NAME,
					)
					.await
					.expect("failed to listen");

					while let Some(event) = events.next().await {
						if !*active_inner.borrow() {
							break;
						}
						(callback_ref.borrow())(event.payload);
					}
				};
				let _ = Abortable::new(fut, abort_registration).await;
			});

			move || {
				*active.borrow_mut() = false;
				abort_handle.abort();
			}
		});
	}
}
