use crate::{
	KeyboardShortcutListener,
	api::{FrontendApiClient, use_db_listen},
	views::{Axis, EmailFolderPane, EmailListPane, ReaderPane, SettingsDialog, SplitPanes},
};
use desktop_email_client_shared::{DatabaseChangedEvent, DatabaseTable};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use yew::prelude::*;

#[component]
pub fn App() -> Html {
	let database_context = use_state(|| DatabaseContext {
		revisions: Rc::new(RefCell::new(HashMap::new())),
		db_revision: 0,
	});

	use_db_listen({
		let database_context = database_context.clone();
		move |event: DatabaseChangedEvent| {
			tracing::debug!("recieved db change on table: {:?}", event.changed_table);
			database_context.set((*database_context).clone().invalidate(event.changed_table));
		}
	});

	html! {
		<main class="font-sans m-0">
			<ContextProvider<UseStateHandle<DatabaseContext>> context={database_context}>
				<AppImpl />
			</ContextProvider<UseStateHandle<DatabaseContext>>>
		</main>
	}
}

/// use_db_listen and use_state should be in different function_components,
/// as when state changes trigger a db change, that db change could be lost due to the db listener
/// being unmounted and remounted
#[component]
pub fn AppImpl() -> Html {
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

#[derive(Debug, Clone)]
pub struct DatabaseContext {
	/// HashMap from table to revision number
	revisions: Rc<RefCell<HashMap<DatabaseTable, usize>>>,
	/// increments on every db update, regargless of the affected changed_table
	/// insures that child components that referenced ´get_revision_for´ reevaluate
	db_revision: usize,
}
impl PartialEq for DatabaseContext {
	fn eq(&self, other: &Self) -> bool {
		self.db_revision.eq(&other.db_revision)
	}
}
impl DatabaseContext {
	pub fn get_revision_for(&self, table: DatabaseTable) -> usize {
		self.revisions.borrow().get(&table).copied().unwrap_or(0)
	}

	pub fn invalidate(self, table: DatabaseTable) -> Self {
		*self.revisions.borrow_mut().entry(table).or_insert(0) += 1;
		Self {
			revisions: self.revisions.clone(),
			db_revision: self.db_revision + 1,
		}
	}
}

#[hook]
pub fn use_db_context() -> DatabaseContext {
	(*use_context::<UseStateHandle<DatabaseContext>>()
		.expect("no database context provided, missing a <ContextProvider /> above this component"))
	.clone()
}
