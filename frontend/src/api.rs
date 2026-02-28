use crate::{app::use_db_context, use_app_state};
use desktop_email_client_shared::{
	API_ROUTE, ApiClient, DATABASE_CHANGED_EVENT_NAME, DatabaseChangedEvent, DatabaseTable,
	EmailAccount, EmailFilter, EmailFolder, EmailInfo, EmailRow, Request, Response,
};
use gloo::events::EventListener;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::wasm_bindgen::JsCast;
use yew::prelude::*;

pub async fn invoke_backend_api(request: Request) -> Response {
	tracing::debug!("invoking {}", request);

	gloo_net::http::Request::post(API_ROUTE)
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

			let api_route = format!("{}/{}", API_ROUTE, DATABASE_CHANGED_EVENT_NAME);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrontendApiClient;

#[async_trait::async_trait(?Send)]
impl ApiClient for FrontendApiClient {
	async fn dispatch(self, request: Request) -> Response {
		invoke_backend_api(request).await
	}
}

#[hook]
pub fn use_db_resource<D, R, F, Fut>(
	dep: D,
	fetch_resource: F,
	initial_resource: R,
	table: DatabaseTable,
) -> UseStateHandle<R>
where
	D: 'static + Clone + PartialEq,
	R: 'static + PartialEq,
	F: 'static + Clone + Fn(D, &FrontendApiClient) -> Fut,
	Fut: Future<Output = R>,
{
	let resource = use_state(move || initial_resource);
	let app_state = use_app_state();

	let db_context = use_db_context();
	let revision = db_context.get_revision_for(table);
	use_effect_with((revision, dep.clone()), {
		let resource = resource.clone();
		let fetch_resource = fetch_resource.clone();
		let api_client = app_state.api_client;

		move |(_, dep)| {
			let resource = resource.clone();
			let api_client = api_client;
			let dep = dep.clone();
			wasm_bindgen_futures::spawn_local(async move {
				resource.set(fetch_resource(dep, &api_client).await);
			});
		}
	});

	resource
}

#[hook]
pub fn use_email_accounts() -> UseStateHandle<Vec<EmailAccount>> {
	use_db_resource(
		(),
		move |_, api_client: &FrontendApiClient| api_client.get_email_accounts(),
		Vec::new(),
		DatabaseTable::EmailAccounts,
	)
}

#[hook]
pub fn use_email_folders(email_account_id: i64) -> UseStateHandle<Vec<EmailFolder>> {
	use_db_resource(
		email_account_id,
		move |email_account_id, api_client: &FrontendApiClient| {
			api_client.get_email_folders(email_account_id)
		},
		Vec::new(),
		DatabaseTable::EmailFolders,
	)
}

#[hook]
pub fn use_email_row(email_id: i64) -> UseStateHandle<Option<EmailRow>> {
	use_db_resource(
		email_id,
		move |email_id, api_client: &FrontendApiClient| api_client.get_email(email_id),
		None,
		DatabaseTable::Emails,
	)
}

#[hook]
pub fn use_emails(email_filter: EmailFilter) -> UseStateHandle<Vec<EmailInfo>> {
	use_db_resource(
		email_filter.clone(),
		move |email_filter, api_client: &FrontendApiClient| api_client.get_emails(email_filter),
		Vec::new(),
		DatabaseTable::Emails,
	)
}
