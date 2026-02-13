use desktop_email_client_shared::EmailFolder;
use uuid::Uuid;
use yew::{
	prelude::*,
	suspense::{use_future, use_future_with},
};
use yew_autoprops::autoprops;

use crate::{
	AppState,
	api::{get_email_accounts, get_email_folders},
	use_app_state,
};

#[component]
pub fn EmailFolderPane() -> HtmlResult {
	let app_state = use_app_state();

	let email_accounts = use_future(|| get_email_accounts(()))?;

	let email_account_name_styling = "text-wrap-mode: nowrap; white-space-collapse: preserve; user-select: none; font-size: var(--font-base);";

	Ok(html! {
		<div class="w-full p-10px flex flex-col" style="height: inherit">
			<div class="flex flex-col overflow-y-auto grow p-[15px] gap-1">
				<div class="flex flex-row gap-2">
					<EmailFolderView
						folder_name="All"
						on_select_folder_uuid={None}
						count=420
						app_state={app_state.clone()}
					/>

					<ToggleEmailSearchbarButton />
				</div>

				for (i, email_account) in email_accounts.iter().enumerate() {
					<details open=true class="collapse collapse-arrow shrink-0">
						<summary class="collapse-title text-base pt-[5px] pb-[5px] flex flex-row">
							<div style={email_account_name_styling}>{format!("{} (", email_account.name)}</div>
							<div style={email_account_name_styling} class="truncate">
								{&email_account.address}
							</div>
							<div style={email_account_name_styling}>{")"}</div>
						</summary>
						<div class="collapse-content">
							<EmailFolders email_account_index={i} />
						</div>
					</details>
				}
			</div>
		</div>
	})
}

#[component]
fn ToggleEmailSearchbarButton() -> Html {
	let app_state = use_app_state();

	let on_toggle_searchbar = {
		let app_state = app_state.clone();
		move |_| {
			app_state.set(
				(*app_state)
					.clone()
					.set_show_email_searchbar(!app_state.show_email_searchbar),
			)
		}
	};

	let bg_color_class = if app_state.show_email_searchbar {
		Some(classes!("bg-accent"))
	} else {
		None
	};

	html! {
		<div class="tooltip tooltip-bottom" data-tip="Search">
			<button
				onclick={on_toggle_searchbar}
				class={classes!("btn", "btn-square", "btn-ghost", "hover:bg-accent", bg_color_class)}
			>
				<img src="/static/icons/search.svg" width="16px" height="16px" />
			</button>
		</div>
	}
}

#[autoprops]
#[component]
fn EmailFolders(email_account_index: usize) -> HtmlResult {
	let app_state = use_app_state();
	let folders = use_future_with(email_account_index, |email_account_index| {
		get_email_folders(*email_account_index as u64)
	})?;

	Ok(html! {
		<div class="flex flex-col gap-1">
			for folder in folders.iter() {
				// TODO: email_count!
				<FullEmailFolderView
					folder={folder.clone()}
					count=42
					app_state={app_state.clone()}
				/>
			}
		</div>
	})
}

#[autoprops]
#[component]
fn FullEmailFolderView(
	folder: &EmailFolder,
	count: usize,
	app_state: &UseStateHandle<AppState>,
) -> Html {
	html! {
		<div class="flex flex-col gap-1">
			<EmailFolderView
				folder_name={folder.name.clone()}
				on_select_folder_uuid={Some(folder.uuid)}
				count={count}
				app_state={app_state.clone()}
			/>

			if !folder.subfolders.is_empty() {
				// TODO: make collapsable and add vertical line to guide eye with indentation like in code editors
				<div class="flex flex-col pl-[30px] gap-1">
					for subfolder in folder.subfolders.iter() {
						// TODO: folder email count
						<FullEmailFolderView
							folder={subfolder.clone()}
							count={42}
							app_state={app_state.clone()}
						/>
					}
				</div>
			}
		</div>
	}
}

/// `on_select_folder_uuid`: None means All, see `AppState::selected_email_folder_uuid`
#[autoprops]
#[component]
fn EmailFolderView(
	folder_name: &String,
	on_select_folder_uuid: &Option<Uuid>,
	count: usize,
	app_state: &UseStateHandle<AppState>,
) -> Html {
	let on_select_folder_uuid = *on_select_folder_uuid;
	let on_click = {
		let app_state = app_state.clone();
		move |_| {
			app_state.set(AppState {
				selected_email_folder_uuid: on_select_folder_uuid,
				..((*app_state).clone())
			});
		}
	};

	let selected = app_state.selected_email_folder_uuid == on_select_folder_uuid;

	html! {
		<div class="w-full">
			<button
				class={classes!("btn", "btn-ghost", if selected { "bg-accent" } else { "bg-transparent" }, "hover:bg-accent", "w-full")}
				onclick={on_click}
			>
				<img src="/static/icons/inbox.svg" width="16px" height="16px" class="w-4 h-4 shrink-0 gap-1" />
				<div class="truncate">{folder_name}</div>
				<small class="ml-auto text-xs text-gray-100 font-thin">{format!("{}", count)}</small>
			</button>
		</div>
	}
}
