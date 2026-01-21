use desktop_email_client::EmailAccountSelection;
use gloo::events::EventListener;
use std::cell::RefCell;
use web_sys::{wasm_bindgen::JsCast, window};
use yew::{Component, Context, Html, KeyboardEvent, NodeRef, Properties, UseStateHandle};

use crate::{AppState, views::focus_email_searchbar_input};

fn handle_keyboard_shortcut(
	e: &KeyboardEvent,
	email_searchbar_input_ref: &NodeRef,
	app_state: &UseStateHandle<AppState>,
) {
	if e.key() == "f" && e.ctrl_key() {
		// prevents the default search page ui from showing up, instead of our searchbar focus
		e.prevent_default();

		focus_email_searchbar_input(email_searchbar_input_ref);
	} else if let Ok(num) = e.key().parse::<u8>()
		&& e.ctrl_key()
		&& (1..=9).contains(&num)
	{
		app_state.set(AppState {
			email_account_selection: EmailAccountSelection::from_keyboard_shortcut(
				num,
				app_state.email_accounts.len() as u32,
			),
			..(**app_state).clone()
		});
	}
}

/// invisible component that simply registers itself to the window's keydown event
pub struct KeyboardShortcutListener {
	event_listener: RefCell<Option<EventListener>>,
}
#[derive(Properties, PartialEq)]
pub struct KeyboardShortcutListenerProps {
	pub email_searchbar_input_ref: NodeRef,
	pub app_state: UseStateHandle<AppState>,
}
impl Component for KeyboardShortcutListener {
	type Message = ();
	type Properties = KeyboardShortcutListenerProps;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			event_listener: RefCell::new(None),
		}
	}

	/// reads the event listener every rerender
	fn view(&self, ctx: &Context<Self>) -> Html {
		let email_searchbar_input_ref = ctx.props().email_searchbar_input_ref.clone();
		let app_state = ctx.props().app_state.clone();
		let new_event_listener =
			add_keyboard_shortcut_event_listener(email_searchbar_input_ref, app_state);
		let _ = self.event_listener.replace(Some(new_event_listener));

		Html::default()
	}
}

fn add_keyboard_shortcut_event_listener(
	email_searchbar_input_ref: NodeRef,
	app_state: UseStateHandle<AppState>,
) -> EventListener {
	let window = window().unwrap();
	EventListener::new(&window, "keydown", move |e| {
		if let Some(e) = e.dyn_ref::<KeyboardEvent>() {
			handle_keyboard_shortcut(e, &email_searchbar_input_ref, &app_state);
		}
	})
}
