use desktop_email_client::EmailAccountSelection;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::AppState;

#[component]
pub fn EmailFolderPane() -> Html {
	html! {
		<div class="w-full">
			<EmailAccountSelector />
		</div>
	}
}

// TODO: make generic enum UI
#[component]
fn EmailAccountSelector() -> Html {
	let app_state = use_context::<UseStateHandle<AppState>>().unwrap();

	let all_selected = app_state.email_account_selection.all_selected();
	let is_email_account_index_selected = |email_account_index| matches!(&app_state.email_account_selection, EmailAccountSelection::Single{ index } if *index == email_account_index as u32);

	let email_account_select = use_node_ref();
	let on_change_email_selection = {
		let app_state = app_state.clone();
		let email_account_select = email_account_select.clone();
		move |_e| {
			if let Some(email_account_select) = email_account_select
				.cast::<HtmlInputElement>()
				.map(|select| select.value())
			{
				match email_account_select.parse::<i64>() {
					Ok(index) => app_state.set(AppState {
						email_account_selection: match index {
							-1 => EmailAccountSelection::All,
							_ => EmailAccountSelection::Single {
								index: index as u32,
							},
						},
						..(*app_state).clone()
					}),
					Err(e) => {
						tracing::error!("failed to parse email account selection value: {}", e)
					}
				}
			}
		}
	};

	html! {
		<div class="p-[15px]">

			<select
				ref={email_account_select}
				name="email_account_select"
				class="mt-auto p-2 rounded-lg border w-full"
				style="background-color: var(--color-bg);"
				onchange={on_change_email_selection}
			>
				<option value="-1" selected={all_selected}>
					<div class="no-select">{"All accounts"}</div>
				</option>

				for (email_account_index, email_account) in app_state.email_accounts.iter().enumerate() {
					<option
						value={format!("{}", email_account_index)}
						selected={is_email_account_index_selected(email_account_index)}
					>
						<div class="no-select">{ &email_account.name }</div>
					</option>
				}
			</select>

		</div>
	}
}
