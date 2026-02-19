use desktop_email_client_shared::ApiClient;
use yew::{prelude::*, suspense::use_future_with};
use yew_autoprops::autoprops;

use crate::{use_app_state, views::EmailView};

#[component]
pub fn ReaderPane() -> Html {
	let state = use_app_state();

	let content = match &state.selected_email_id {
		Some(email_id) => html! {
			<Suspense>
				<ReaderPaneContent email_id={*email_id} />
			</Suspense>
		},
		None => html! {
			<p class="p-3 w-full h-full flex items-center justify-center select-none">{ "Select an email" }</p>
		},
	};

	html! {
		<div class="w-full h-full">
			{ content }
		</div>
	}
}

#[autoprops]
#[component]
fn ReaderPaneContent(email_id: &i64) -> HtmlResult {
	let state = use_app_state();
	let api_client = &state.api_client;
	let email = use_future_with(*email_id, |email_id| api_client.get_email(*email_id))?;

	Ok(match email.as_ref() {
		Some(email) => html! {
			<EmailView email={email.clone()} />
		},
		None => html! {
			<p class="p-3 w-full h-full flex items-center justify-center select-none">{ "Email not found" }</p>
		},
	})
}
