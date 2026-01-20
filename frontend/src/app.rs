use std::{collections::HashSet, rc::Rc};

use crate::{
	KeyboardShortcutListener,
	views::{Axis, EmailListPane, ReaderPane, SplitPanes},
};
use desktop_email_client::{EmailAccount, EmailAccountSelection, EmailProvider};
use uuid::Uuid;
use yew::prelude::*;

#[component]
pub fn App() -> Html {
	let app_state = use_state(AppState::create_mock);

	let email_searchbar_input_ref = NodeRef::default();

	let email_list_pane = html! {
		<EmailListPane
			email_searchbar_input_ref={&email_searchbar_input_ref}
		/>
	};

	let reader_pane = html! {
		<ReaderPane />
	};

	html! {
		<main class="font-sans m-0 text-white bg-black">
			<ContextProvider<UseStateHandle<AppState>> context={app_state}>
				<SplitPanes
					axis={Axis::Vertical}
					height={"100vh"}
					left={email_list_pane}
					right={reader_pane}
				/>
				<KeyboardShortcutListener email_searchbar_input_ref={email_searchbar_input_ref} />
			</ContextProvider<UseStateHandle<AppState>>>
		</main>
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
	pub email_accounts: Vec<Rc<EmailAccount>>,
	pub email_account_selection: EmailAccountSelection,
	pub read_email_uuids: HashSet<Uuid>,
	pub selected_email_uuid: Option<Uuid>,
	pub email_search_input: Option<AttrValue>,
}
impl AppState {
	pub fn create_mock() -> Self {
		Self {
			email_accounts: vec![
				Rc::new(EmailAccount::new(
					"Account 1".to_string(),
					"account1@example.com".to_string(),
					EmailProvider::create_fake(),
				)),
				Rc::new(EmailAccount::new(
					"Account 2".to_string(),
					"account2@example.com".to_string(),
					EmailProvider::create_fake(),
				)),
				Rc::new(EmailAccount::new(
					"Account 3".to_string(),
					"account3@example.com".to_string(),
					EmailProvider::create_fake(),
				)),
				Rc::new(EmailAccount::new(
					"Account 4".to_string(),
					"account4@example.com".to_string(),
					EmailProvider::create_fake(),
				)),
			],
			email_account_selection: EmailAccountSelection::All,
			read_email_uuids: HashSet::new(),
			selected_email_uuid: None,
			email_search_input: None,
		}
	}
}
