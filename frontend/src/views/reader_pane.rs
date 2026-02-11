use uuid::Uuid;
use yew::{prelude::*, suspense::use_future_with};
use yew_autoprops::autoprops;

use crate::{api::get_email, use_app_state, views::EmailView};

#[component]
pub fn ReaderPane() -> Html {
	let state = use_app_state();

	let content = match &state.selected_email_uuid {
		Some(email_uuid) => html! {
			<Suspense>
				<ReaderPaneContent email_uuid={*email_uuid} />
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
fn ReaderPaneContent(email_uuid: &Uuid) -> HtmlResult {
	let email = use_future_with(*email_uuid, |email_uuid| get_email(*email_uuid))?;

	Ok(match email.as_ref() {
		Some(email) => html! {
			<EmailView email={email.clone()} />
		},
		None => html! {
			<p class="p-3 w-full h-full flex items-center justify-center select-none">{ "Email not found" }</p>
		},
	})
}
