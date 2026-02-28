use yew::prelude::*;
use yew_autoprops::autoprops;

use crate::{api::use_email_row, use_app_state, views::EmailView};

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
fn ReaderPaneContent(email_id: &i64) -> Html {
	let email_row = use_email_row(*email_id);

	match email_row.as_ref() {
		Some(email_row) => html! {
			<EmailView email={email_row.email.clone()} />
		},
		None => html! {
			<p class="p-3 w-full h-full flex items-center justify-center select-none">{ "Email not found" }</p>
		},
	}
}
