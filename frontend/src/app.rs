use crate::{
	KeyboardShortcutListener,
	views::{Axis, EmailFolderPane, EmailListPane, ReaderPane, SplitPanes},
};
use desktop_email_client_shared::EmailAccountSelection;
use uuid::Uuid;
use yew::prelude::*;

#[component]
pub fn App() -> Html {
	let app_state = use_state(AppState::default);

	let email_searchbar_input_ref = NodeRef::default();

	let email_folder_pane = html! {
		<Suspense>
			<EmailFolderPane />
		</Suspense>
	};

	let email_list_pane = html! {
		<EmailListPane
			email_searchbar_input_ref={&email_searchbar_input_ref}
		/>
	};

	let reader_pane = html! {
		<ReaderPane />
	};

	let main_pane = html! {
		<SplitPanes
			name="folder_sidebar"
			axis={Axis::Vertical}
			height={"100vh"}
			left={email_list_pane}
			right={reader_pane}
		/>
	};

	html! {
		<main class="font-sans m-0">
			<ContextProvider<UseStateHandle<AppState>> context={app_state.clone()}>
				<SplitPanes
					name="list_reader_split"
					axis={Axis::Vertical}
					height={"100vh"}
					starting_width=500
					left={email_folder_pane}
					right={main_pane}
				/>
				<KeyboardShortcutListener email_searchbar_input_ref={email_searchbar_input_ref} app_state={app_state} />
			</ContextProvider<UseStateHandle<AppState>>>
		</main>
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
	pub email_account_selection: EmailAccountSelection,
	pub selected_email_uuid: Option<Uuid>,
	/// None is implicit "All emails" folders
	pub selected_email_folder_uuid: Option<Uuid>,
	pub email_search_input: Option<AttrValue>,
}
impl Default for AppState {
	fn default() -> Self {
		Self {
			email_account_selection: EmailAccountSelection::All,
			selected_email_uuid: None,
			selected_email_folder_uuid: None,
			email_search_input: None,
		}
	}
}
