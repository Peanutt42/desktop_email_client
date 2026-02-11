use gloo::events::{EventListener, EventListenerOptions};
use std::cell::RefCell;
use web_sys::{wasm_bindgen::JsCast, window};
use yew::{Component, Context, Html, KeyboardEvent, Properties, UseStateHandle};

use crate::AppState;

fn handle_keyboard_shortcut(e: &KeyboardEvent, app_state: &UseStateHandle<AppState>) {
	let key = e.key();

	if key == "f" && e.ctrl_key() {
		// prevents the default search page ui from showing up, instead of our searchbar focus
		e.prevent_default();

		app_state.set((**app_state).clone().set_show_email_searchbar(true));
	} else if key == "Escape" {
		app_state.set((**app_state).clone().set_show_email_searchbar(false));
	}
}

/// invisible component that simply registers itself to the window's keydown event
pub struct KeyboardShortcutListener {
	event_listener: RefCell<Option<EventListener>>,
}
#[derive(Properties, PartialEq)]
pub struct KeyboardShortcutListenerProps {
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
		let app_state = ctx.props().app_state.clone();
		let new_event_listener = add_keyboard_shortcut_event_listener(app_state);
		let _ = self.event_listener.replace(Some(new_event_listener));

		Html::default()
	}
}

fn add_keyboard_shortcut_event_listener(app_state: UseStateHandle<AppState>) -> EventListener {
	let window = window().unwrap();
	EventListener::new_with_options(
		&window,
		"keydown",
		EventListenerOptions::enable_prevent_default(),
		move |e| {
			if let Some(e) = e.dyn_ref::<KeyboardEvent>() {
				handle_keyboard_shortcut(e, &app_state);
			}
		},
	)
}
