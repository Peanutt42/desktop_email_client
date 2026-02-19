use crate::{AppState, pretty_format_date_time, use_app_state, views::EmailAvatar};
use desktop_email_client_shared::{ApiClient, EmailFilter, EmailInfo};
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
		<div class={classes!("p-4", display_class)}>
			<label class="input w-full">
				<img width="16px" height="16px" src="/static/icons/search.svg" />

				<input
					ref={email_searchbar_input_ref}
					type="search"
					placeholder="Search"
					class="grow"
					oninput={on_searchbar_input}
				/>

				// Keyboard shortcut
				<kbd class="kbd kbd-xs">{"Ctrl"}</kbd>
				{"+"}
				<kbd class="kbd kbd-xs">{"F"}</kbd>
			</label>
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
	let api_client = &state.api_client;
	let search_results = use_future_with(search, |search| {
		api_client.get_emails(EmailFilter::MatchingSearch {
			search: search.to_string(),
		})
	})?;

	Ok(html! {
		for email_info in search_results.iter() {
			<EmailListItem
				email_info={email_info.clone()}
				selected={state.selected_email_id == Some(email_info.email_id)}
			/>
		}
	})
}

#[component]
fn EmailListOfSelectedFolder() -> HtmlResult {
	let state = use_app_state();
	let api_client = &state.api_client;
	let search_results =
		use_future_with(state.selected_email_folder_id, |selected_email_folder_id| {
			api_client.get_emails(match *selected_email_folder_id {
				Some(folder_id) => EmailFilter::Folder { folder_id },
				None => EmailFilter::Any,
			})
		})?;

	Ok(html! {
		for email_info in search_results.iter() {
			<EmailListItem
				email_info={email_info.clone()}
				selected={state.selected_email_id == Some(email_info.email_id)}
			/>
		}
	})
}

#[autoprops]
#[component]
fn EmailListItem(email_info: &EmailInfo, selected: bool) -> HtmlResult {
	let classes = "p-[7px] rounded-box border-2 border-accent";
	let selected_classes = format!("{} bg-accent border-transparent", classes);
	let not_selected_classes = format!("{} hover:bg-accent cursor-pointer", classes);

	let app_state = use_app_state();
	let api_client = app_state.api_client;
	let email_id = email_info.email_id;
	let on_click = move |_e| {
		// TODO: wait a bit
		wasm_bindgen_futures::spawn_local(api_client.mark_email_read(email_id));

		app_state.set(AppState {
			selected_email_id: Some(email_id),
			..(*app_state).clone()
		});
	};
	let time_sent_ago_formatted = pretty_format_date_time(&email_info.sent_time);

	Ok(html! {
		<button
			key={format!("{}", email_id)}
			class={if selected { selected_classes } else { not_selected_classes }}
			onclick={on_click}
		>
			<div class="flex flex-row items-center gap-2 w-full">
				<EmailAvatar name={email_info.author_name.clone()} />
				<div class="font-semibold truncate">{ &email_info.subject }</div>
			</div>
			if let Some(body_summary) = &email_info.body_summary {
				<small class="truncate nowrap block text-start">{body_summary}</small>
			}
			<div class="flex flex-row items-center gap-1">
				for tag_name in email_info.tags.iter() {
					<EmailTagBadge name={tag_name.clone()} />
				}
				<div class="flex-grow" />
				<small class="ml-auto text-xs text-gray-100 font-thin" style="margin: 0; text-wrap: nowrap;">
					{time_sent_ago_formatted}
				</small>
				<div class={format!("{} w-2 h-2 m-[10px] rounded-full shrink-0", if email_info.read {""} else {"bg-blue-500"})}></div>
			</div>
		</button>
	})
}

#[autoprops]
#[component]
fn EmailTagBadge(name: &AttrValue) -> Html {
	html! {
		<span class="badge badge-sm badge-accent whitespace-nowrap truncate block">{name}</span>
	}
}
