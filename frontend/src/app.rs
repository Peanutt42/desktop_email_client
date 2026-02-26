use crate::{
	KeyboardShortcutListener,
	api::FrontendApiClient,
	views::{Axis, EmailFolderPane, EmailListPane, ReaderPane, SettingsDialog, SplitPanes},
};
use yew::prelude::*;

#[component]
pub fn App() -> Html {
	let app_state = use_state(AppState::default);

	let email_folder_pane = html! {
		<Suspense>
			<EmailFolderPane />
		</Suspense>
	};

	let email_list_pane = html! {
		<EmailListPane />
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
				<SettingsDialog node_ref={&app_state.settings_dialog_node_ref} />
				<KeyboardShortcutListener app_state={app_state} />
			</ContextProvider<UseStateHandle<AppState>>>
		</main>
	}
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AppState {
	pub api_client: FrontendApiClient,
	pub selected_email_id: Option<i64>,
	/// None is implicit "All emails" folders
	pub selected_email_folder_id: Option<i64>,
	pub email_search_input: Option<AttrValue>,
	pub show_email_searchbar: bool,
	pub settings_dialog_node_ref: NodeRef,
}
impl AppState {
	pub fn set_show_email_searchbar(self, show_email_searchbar: bool) -> Self {
		Self {
			show_email_searchbar,
			..self
		}
	}
}

#[hook]
pub fn use_app_state() -> UseStateHandle<AppState> {
	use_context::<UseStateHandle<AppState>>()
		.expect("no AppState context provided, missing a <ContextProvider /> above this component")
}
