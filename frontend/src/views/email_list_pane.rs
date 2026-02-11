use crate::{
	AppState,
	api::{get_emails_in_folder, get_emails_matching_search, mark_email_read},
	pretty_format_date_time, use_app_state,
};
use desktop_email_client_shared::{Email, EmailBody};
use uuid::Uuid;
use web_sys::HtmlInputElement;
use yew::{prelude::*, suspense::use_future_with};
use yew_autoprops::autoprops;

#[component]
pub fn EmailListPane() -> Html {
	html! {
		<ul class="w-full flex flex-col select-none" style="height: inherit">
			<EmailSearchbar />

			<EmailListItems />
		</ul>
	}
}

#[component]
fn EmailSearchbar() -> Html {
	let app_state = use_app_state();

	let email_searchbar_input_ref = use_node_ref();

	let on_searchbar_input = {
		let state = use_app_state();
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

	let display_class = classes!(if app_state.show_email_searchbar {
		"block"
	} else {
		"hidden"
	});

	// autofocus when showing the searchbar
	{
		let email_searchbar_input_ref = email_searchbar_input_ref.clone();
		use_effect_with(app_state.show_email_searchbar, move |_| {
			if let Some(input) = email_searchbar_input_ref.cast::<HtmlInputElement>() {
				let _ = input.focus();
			}
			|| ()
		});
	}

	html! {
		<div class={classes!("bg-background/95", "p-4", display_class)}>
			<div class="relative">
				<svg class="lucide lucide-search absolute left-2 top-2.5 h-4 w-4 text-muted-foreground"
					width="24"
					height="24"
					stroke="white"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<image href="/static/icons/search.svg" />
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
	let state = use_app_state();

	let items = match &state.email_search_input {
		Some(search) if !search.is_empty() => html! {
			<EmailMatchingSearchResultList search={search} />
		},
		_ => html! {
			<EmailListOfSelectedFolder />
		},
	};

	html! {
		<Suspense>
			<div class="flex flex-col overflow-y-auto gap-1.5 p-2 grow w-full">
				{ items }
			</div>
		</Suspense>
	}
}

#[autoprops]
#[component]
fn EmailMatchingSearchResultList(search: AttrValue) -> HtmlResult {
	let state = use_app_state();
	let search_results = use_future_with(search, |search| {
		get_emails_matching_search(search.to_string())
	})?;

	Ok(html! {
		for search_result in search_results.iter() {
			<EmailListItem
				email={search_result.email.clone()}
				email_uuid={search_result.email_uuid}
				selected={state.selected_email_uuid == Some(search_result.email_uuid)}
			/>
		}
	})
}

#[component]
fn EmailListOfSelectedFolder() -> HtmlResult {
	let state = use_app_state();
	let search_results = use_future_with(
		state.selected_email_folder_uuid,
		|selected_email_folder_uuid| get_emails_in_folder(*selected_email_folder_uuid),
	)?;

	Ok(html! {
		for search_result in search_results.iter() {
			<EmailListItem
				email={search_result.email.clone()}
				email_uuid={search_result.email_uuid}
				selected={state.selected_email_uuid == Some(search_result.email_uuid)}
			/>
		}
	})
}

#[autoprops]
#[component]
fn EmailListItem(email: &Email, email_uuid: &Uuid, selected: bool) -> HtmlResult {
	let classes = "p-[7px] rounded-[7px] border-2";
	let selected_classes = format!("{} bg-[#555] border-transparent", classes);
	let not_selected_classes = format!("{} border-[#222] hover:bg-[#222] cursor-pointer", classes);

	let app_state = use_app_state();
	let email_uuid = *email_uuid;
	let on_click = move |_e| {
		// TODO: wait a bit
		wasm_bindgen_futures::spawn_local(mark_email_read(email_uuid));

		app_state.set(AppState {
			selected_email_uuid: Some(email_uuid),
			..(*app_state).clone()
		});
	};
	let time_sent_ago_formatted = pretty_format_date_time(&email.sent_time);

	Ok(html! {
		<div
			key={format!("{}", email_uuid)}
			class={if selected { selected_classes } else { not_selected_classes }}
			onclick={on_click}
		>
			<div class="flex flex-row items-center gap-2 w-full">
				<EmailAvatar />
				<div class="font-semibold truncate">{ &email.subject }</div>
			</div>
			if let EmailBody::TextOnly(body_text) = &email.body {
				<small class="truncate nowrap block">{ body_text.clone() }</small>
			}
			<div class="flex flex-row items-center gap-1">
				for tag_name in email.tags.iter() {
					<EmailTagBadge name={tag_name.clone()} />
				}
				<div class="flex-grow" />
				<small class="ml-auto text-xs text-gray-100 font-thin" style="margin: 0; text-wrap: nowrap;">
					{time_sent_ago_formatted}
				</small>
				<div class={format!("{} w-2 h-2 m-[10px] rounded-full shrink-0", if email.read {""} else {"bg-blue-500"})}></div>
			</div>
		</div>
	})
}

#[autoprops]
#[component]
fn EmailTagBadge(name: &AttrValue) -> Html {
	html! {
		<div class="inline-flex items-center text-nowrap truncate rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-white text-black foreground shadow hover:bg-primary/80">
			{name}
		</div>
	}
}

#[component]
fn EmailAvatar() -> Html {
	html! {
		<img src="/static/icons/person-circle.svg" class="w-[35px] min-w-[35px] h-[35px] min-h-[35px] shrink-0 align-middle rounded-full" />
	}
}
