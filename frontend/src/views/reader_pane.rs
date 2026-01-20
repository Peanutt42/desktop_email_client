use yew::prelude::*;
use yew_autoprops::autoprops;

use crate::{AppState, views::EmailView};

#[autoprops]
#[component]
pub fn ReaderPane() -> Html {
	html! {
		<div class="w-full h-full">
			<ReaderPaneContent />
		</div>
	}
}

#[component]
fn ReaderPaneContent() -> Html {
	let state = use_context::<UseStateHandle<AppState>>().unwrap();

	match &state.selected_email_uuid {
		Some(email_uuid) => {
			for email_account in state.email_accounts.iter() {
				if let Some(email) = email_account.provider.get_emails().get(email_uuid) {
					return html! {
						<EmailView email={email.clone()} />
					};
				}
			}

			html! {
				<p class="p-3">{ "Email not found" }</p>
			}
		}
		None => html! { <p class="p-3">{ "Select an email" }</p> },
	}
}
