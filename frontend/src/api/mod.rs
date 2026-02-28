#[cfg(feature = "non_ipc_backend")]
mod actix_non_ipc_api;
#[cfg(feature = "non_ipc_backend")]
use actix_non_ipc_api::invoke_backend_api;
#[cfg(feature = "non_ipc_backend")]
pub use actix_non_ipc_api::use_db_listen;

#[cfg(not(feature = "non_ipc_backend"))]
mod tauri_ipc_api;
#[cfg(not(feature = "non_ipc_backend"))]
use tauri_ipc_api::invoke_backend_api;
#[cfg(not(feature = "non_ipc_backend"))]
pub use tauri_ipc_api::use_db_listen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrontendApiClient;

#[async_trait::async_trait(?Send)]
impl ApiClient for FrontendApiClient {
	async fn dispatch(self, request: Request) -> Response {
		invoke_backend_api(request).await
	}
}

use crate::{app::use_db_context, use_app_state};
use desktop_email_client_shared::{
	ApiClient, DatabaseTable, EmailAccount, EmailFilter, EmailFolder, EmailInfo, EmailRow, Request,
	Response,
};
use yew::prelude::*;

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
