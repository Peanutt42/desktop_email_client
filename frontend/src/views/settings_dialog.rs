#[cfg(not(feature = "non_ipc_backend"))]
use crate::api::use_db_listen;
use desktop_email_client_shared::{ApiClient, DatabaseChangedEvent, EmailAccount};
use yew::{prelude::*, suspense::use_future};
use yew_autoprops::autoprops;

use crate::{use_app_state, views::AddEmailAccountScreen};

pub fn show_settings_dialog(dialog_node_ref: &NodeRef) {
	if let Some(dialog) = dialog_node_ref.cast::<web_sys::HtmlDialogElement>() {
		let _ = dialog.show_modal();
	}
}

#[autoprops]
#[component]
pub fn SettingsDialog(node_ref: &NodeRef) -> Html {
	enum SettingsTab {
		EmailAccounts,
	}

	enum State {
		Settings(SettingsTab),
		AddEmailAccount,
	}

	let state = use_state(|| State::Settings(SettingsTab::EmailAccounts));

	let dialog_content = match *state {
		State::Settings(SettingsTab::EmailAccounts) => {
			let state = state.clone();
			let add_email_account_callback = move |_| {
				state.set(State::AddEmailAccount);
			};

			html! {
				<div class="flex flex-row h-full pr-6">
					<div class="overflow-y-auto w-3xs">
						<button class="btn btn-primary w-full">{"Email Accounts"}</button>
					</div>

					<div class="divider divider-horizontal"></div>

					<Suspense>
						<EmailAccountsSettings add_email_account_callback={add_email_account_callback} />
					</Suspense>
				</div>
			}
		}
		State::AddEmailAccount => {
			let state = state.clone();
			html! {
				<AddEmailAccountScreen
					close_callback={move |_| state.set(State::Settings(SettingsTab::EmailAccounts))}
				/>
			}
		}
	};

	let on_close_btn_pressed = {
		let node_ref = node_ref.clone();
		move |_| {
			if let Some(dialog) = node_ref.cast::<web_sys::HtmlDialogElement>() {
				dialog.close();
			}
		}
	};

	html! {
		<dialog ref={node_ref} class="modal">
			<div class="modal-box w-full h-full max-w-4xl max-h-(--container-2xl) bg-base-200">
				{dialog_content}

				<button class="btn btn-sm btn-circle btn-ghost text-lg absolute right-2 top-2" onclick={on_close_btn_pressed}>{"✕"}</button>
			</div>
			<form method="dialog" class="modal-backdrop">
				<button>{"close"}</button>
			</form>
		</dialog>
	}
}

#[autoprops]
#[component]
fn EmailAccountsSettings(add_email_account_callback: Callback<()>) -> HtmlResult {
	let app_state = use_app_state();
	let api_client = app_state.api_client;

	// let email_accounts = use_future(|| app_state.api_client.get_email_accounts())?;

	let email_accounts = use_state(|| Vec::new());

	use_effect_with((), {
		let email_accounts = email_accounts.clone();
		let api_client = api_client.clone();
		move |_| {
			wasm_bindgen_futures::spawn_local(async move {
				email_accounts.set(api_client.get_email_accounts().await);
			});
		}
	});

	#[cfg(not(feature = "non_ipc_backend"))]
	use_db_listen(Callback::from({
		let email_accounts = email_accounts.clone();
		let api_client = api_client.clone();
		move |event| {
			if matches!(event, DatabaseChangedEvent::EmailAccountsChanged) {
				let email_accounts = email_accounts.clone();
				let api_client = api_client.clone();
				wasm_bindgen_futures::spawn_local(async move {
					email_accounts.set(api_client.get_email_accounts().await);
				});
			}
		}
	}));

	Ok(html! {
		<div class="flex flex-col w-full gap-4">
			<div class="overflow-y-auto flex flex-col gap-4">
				for email_account in email_accounts.iter() {
					<EmailAccountSettings email_account={email_account.clone()} />
				}
			</div>

			<button class="btn btn-primary" onclick={move |_| add_email_account_callback.emit(())}>{"Add Email Account"}</button>
		</div>
	})
}

#[autoprops]
#[component]
fn EmailAccountSettings(email_account: &EmailAccount) -> Html {
	let app_state = use_app_state();

	let confirm_dialog_node_ref = use_node_ref();

	let on_delete_btn_clicked = {
		let confirm_dialog_node_ref = confirm_dialog_node_ref.clone();
		move |_: MouseEvent| {
			if let Some(dialog) = confirm_dialog_node_ref.cast::<web_sys::HtmlDialogElement>() {
				let _ = dialog.show_modal();
			}
		}
	};

	let close_confirm_dialog = {
		let confirm_dialog_node_ref = confirm_dialog_node_ref.clone();
		move || {
			if let Some(dialog) = confirm_dialog_node_ref.cast::<web_sys::HtmlDialogElement>() {
				dialog.close();
			}
		}
	};

	let delete_email_account = {
		let email_account_id = email_account.id;
		let close_confirm_dialog = close_confirm_dialog.clone();
		move || {
			close_confirm_dialog();
			wasm_bindgen_futures::spawn_local(
				app_state.api_client.remove_email_account(email_account_id),
			);
		}
	};

	let name = email_account.name.clone();
	let address = email_account.address.clone();

	html! {
		<div class="card card-border w-full bg-base-300 shadow-xl">
			<div class="card-body">
				<h2 class="card-title">{name}</h2>
				<p>{address}</p>
				<div class="card-actions">
					<button class="btn btn-sm btn-primary" disabled=true>
						<img src="/static/icons/pencil-square.svg" />
						{"Edit"}
					</button>
					<button onclick={on_delete_btn_clicked} class="btn btn-sm btn-error">
						<img src="/static/icons/trash.svg" />
						{ "Delete" }
					</button>

					<DeleteEmailAccountConfirmDialog
						dialog_node_ref={confirm_dialog_node_ref.clone()}
						email_account_name={email_account.name.clone()}
						on_delete={move |_| delete_email_account()}
						on_cancel={move |_| close_confirm_dialog()}
					/>
				</div>
			</div>
		</div>
	}
}

#[autoprops]
#[component]
fn DeleteEmailAccountConfirmDialog(
	dialog_node_ref: NodeRef,
	email_account_name: AttrValue,
	on_delete: Callback<()>,
	on_cancel: Callback<()>,
) -> Html {
	html! {
		<dialog ref={dialog_node_ref} class="modal">
			<div class="modal-box bg-base-200 w-xl">
				<div>
					<h2 class="text-lg pb-4">{format!("Are you sure you want to delete email account \"{}\"?", email_account_name)}</h2>

					<div class="flex flex-row gap-2 justify-end">
						<button onclick={move |_| on_delete.emit(())} class="btn btn-error">{"Delete"}</button>
						<button onclick={move |_| on_cancel.emit(())} class="btn btn-primary">{"Cancel"}</button>
					</div>
				</div>
			</div>
			<form method="dialog" class="modal-backdrop">
				<button>{"close"}</button>
			</form>
		</dialog>
	}
}
