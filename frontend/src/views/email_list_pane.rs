use crate::{AppState, pretty_format_date_time};
use desktop_email_client::{Email, EmailBody};
use std::rc::Rc;
use uuid::Uuid;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_autoprops::autoprops;

#[autoprops]
#[component]
pub fn EmailListPane(email_searchbar_input_ref: &NodeRef) -> Html {
	html! {
		<ul class="w-full flex flex-col select-none" style="height: inherit">
			<EmailSearchbar email_searchbar_input_ref={email_searchbar_input_ref} />

			<EmailListItems />
		</ul>
	}
}

pub fn focus_email_searchbar_input(email_searchbar_input_ref: &NodeRef) {
	if let Some(input) = email_searchbar_input_ref.cast::<HtmlInputElement>() {
		input.focus().unwrap();
	}
}

#[autoprops]
#[component]
fn EmailSearchbar(email_searchbar_input_ref: &NodeRef) -> Html {
	let on_searchbar_input = {
		let state = use_context::<UseStateHandle<AppState>>().unwrap();
		let searchbar_element = email_searchbar_input_ref.clone();

		move |_e| {
			if let Some(input) = searchbar_element
				.cast::<HtmlInputElement>()
				.map(|select| select.value())
			{
				state.set(AppState {
					email_search_input: Some(input.into()),
					..(*state).clone()
				});
			}
		}
	};

	html! {
		<div class="bg-background/95 p-4">
			<div class="relative">
				<svg class="lucide lucide-search absolute left-2 top-2.5 h-4 w-4 text-muted-foreground"
					width="24"
					height="24"
					stroke="white"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<image href="/static/search.svg" />
				</svg>

				<input
					ref={email_searchbar_input_ref}
					class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 md:text-sm pl-8"
					placeholder="Search"
					type="search"
					name="search"
					 oninput={on_searchbar_input}
				/>
			</div>
		</div>
	}
}

#[component]
fn EmailListItems() -> Html {
	let state = use_context::<UseStateHandle<AppState>>().unwrap();

	let items = match &state.email_search_input {
		Some(search) => {
			let mut search_results = vec![];

			for (email_account_index, email_account) in state.email_accounts.iter().enumerate() {
				if state
					.email_account_selection
					.is_selected(email_account_index as u32)
				{
					for (email_uuid, email) in email_account.provider.get_emails() {
						if let Some(score) = email.match_search_pattern(search) {
							search_results.push((score, *email_uuid, email.clone()));
						}
					}
				}
			}

			// order by score descending
			search_results.sort_by(|(a_score, _, _), (b_score, _, _)| b_score.cmp(a_score));

			html! {
				for (_score, email_uuid, email) in search_results {
					<EmailListItem
						email={email}
						email_uuid={email_uuid}
						selected={state.selected_email_uuid == Some(email_uuid)}
					/>
				}
			}
		}
		None => {
			let mut search_results = vec![];

			for (email_account_index, email_account) in state.email_accounts.iter().enumerate() {
				if state
					.email_account_selection
					.is_selected(email_account_index as u32)
				{
					for (email_uuid, email) in email_account.provider.get_emails() {
						search_results.push((*email_uuid, email.clone()));
					}
				}
			}

			// order by time sent descending (latest to oldest)
			search_results
				.sort_by(|(_, email_a), (_, email_b)| email_b.sent_time.cmp(&email_a.sent_time));

			html! {
				for (email_uuid, email) in search_results {
					<EmailListItem
						email={email}
						email_uuid={email_uuid}
						selected={state.selected_email_uuid == Some(email_uuid)}
					/>
				}
			}
		}
	};

	html! {
		<div class="flex flex-col overflow-y-auto gap-1.5 p-2 grow w-full">
			{ items }
		</div>
	}
}

#[autoprops]
#[component]
fn EmailListItem(email: Rc<Email>, email_uuid: &Uuid, selected: bool) -> Html {
	let classes = "p-[7px] rounded-[7px] border-2";
	let selected_classes = format!("{} bg-[#555] border-transparent", classes);
	let not_selected_classes = format!("{} border-[#222] hover:bg-[#222] cursor-pointer", classes);

	let app_state = use_context::<UseStateHandle<AppState>>().unwrap();
	let email_uuid = *email_uuid;
	let is_email_read = app_state.read_email_uuids.contains(&email_uuid);
	let on_click = move |_e| {
		// TODO: wait a bit
		let mut read_email_uuids = app_state.read_email_uuids.clone();
		read_email_uuids.insert(email_uuid);

		app_state.set(AppState {
			selected_email_uuid: Some(email_uuid),
			read_email_uuids,
			..(*app_state).clone()
		});
	};
	let time_sent_ago_formatted = pretty_format_date_time(&email.sent_time);

	html! {
		<div
			key={format!("{}", email_uuid)}
			class={if selected { selected_classes } else { not_selected_classes }}
			onclick={on_click}
		>
			<div class="flex flex-row items-center gap-2 w-full">
				<EmailAvatar />
				<div class="font-semibold truncate">{ &email.subject }</div>
				if !is_email_read {
					<div class="bg-blue-500 w-2 h-2 rounded-full shrink-0"></div>
				}
				<div class="ml-auto text-xs text-muted-foreground" style="text-wrap: nowrap;">
					{time_sent_ago_formatted}
				</div>
			</div>
			if let EmailBody::TextOnly(body_text) = &email.body {
				<small class="truncate nowrap block">{ body_text.clone() }</small>
			}
			<div class="flex flex-row gap-1">
				for tag_name in email.tags.iter() {
					<EmailTagBadge name={tag_name.clone()} />
				}
			</div>
		</div>
	}
}

#[autoprops]
#[component]
fn EmailTagBadge(name: &AttrValue) -> Html {
	html! {
		<div class="inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-white text-black foreground shadow hover:bg-primary/80">
			{name}
		</div>
	}
}

#[component]
fn EmailAvatar() -> Html {
	html! {
		<img src="https://www.w3schools.com/howto/img_avatar.png" class="w-[50px] h-[50px] align-middle rounded-full" />
	}
}
