use desktop_email_client_shared::{
	EmailAccountSelection, EmailFolder, get_email_accounts, get_email_folders,
};
use uuid::Uuid;
use web_sys::HtmlInputElement;
use yew::{
	prelude::*,
	suspense::{use_future, use_future_with},
};
use yew_autoprops::autoprops;

use crate::AppState;

#[component]
pub fn EmailFolderPane() -> HtmlResult {
	let app_state = use_context::<UseStateHandle<AppState>>().unwrap();

	let email_accounts = use_future(get_email_accounts)?;

	let email_account_name_styling =
		"text-wrap-mode: nowrap; white-space-collapse: preserve; user-select: none;";

	Ok(html! {
		<div class="w-full p-10px flex flex-col" style="height: inherit">
			<EmailAccountSelector />

			<div class="flex flex-col overflow-y-auto grow p-[15px] gap-1">
				<EmailFolderView
					folder_name="All"
					on_select_folder_uuid={None}
					count=420
					app_state={app_state.clone()}
				/>

				for (i, email_account) in email_accounts.iter().enumerate() {
					if app_state.email_account_selection.is_selected(i as u32) {
						if app_state.email_account_selection.all_selected() {
							<div class="w-full h-px mt-[5px] mb-[5px] bg-[#555]" />

							<div class="pt-5px pb-5px flex flex-row">
								<div style={email_account_name_styling}>{format!("{} (", email_account.name)}</div>
								<div style={email_account_name_styling} class="truncate">
									{&email_account.address}
								</div>
								<div style={email_account_name_styling}>{")"}</div>
							</div>
						}

						<EmailFolders email_account_index={i} />
					}
				}
			</div>
		</div>
	})
}

#[autoprops]
#[component]
fn EmailFolders(email_account_index: usize) -> HtmlResult {
	let app_state = use_context::<UseStateHandle<AppState>>().unwrap();
	let folders = use_future_with(email_account_index, |email_account_index| {
		get_email_folders(*email_account_index)
	})?;

	Ok(html! {
		<div>
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

// TODO: make generic enum UI
#[component]
fn EmailAccountSelector() -> HtmlResult {
	let app_state = use_context::<UseStateHandle<AppState>>().unwrap();

	let email_accounts = use_future(get_email_accounts)?;

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

	Ok(html! {
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

				for (email_account_index, email_account) in email_accounts.iter().enumerate() {
					<option
						value={format!("{}", email_account_index)}
						selected={is_email_account_index_selected(email_account_index)}
					>
						<div class="no-select">{ &email_account.name }</div>
					</option>
				}
			</select>

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
	let background_class = if selected {
		"bg-blue-500 hover:bg-blue-400"
	} else {
		""
	};

	html! {
		<div class="w-full">
			<button
				class={format!("{} select-none inline-flex items-center w-full p-[5px] gap-2 whitespace-nowrap text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 [&amp;_svg]:pointer-events-none [&amp;_svg]:size-4 [&amp;_svg]:shrink-0 hover:bg-[#27272a] hover:text-accent-foreground h-9 rounded-md px-3 justify-start", background_class)}
				onclick={on_click}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24px"
					min-width="24px"
					height="24px"
					min-height="24px"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="lucide lucide-file mr-2 h-4 w-4 shrink-0 gap-1"
				>
					<image href="/static/inbox.svg" />
				</svg>
				<div class="truncate">{folder_name}</div>
				<span class="ml-auto">{format!("{}", count)}</span>
			</button>
		</div>
	}
}
