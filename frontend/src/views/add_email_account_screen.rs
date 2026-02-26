use desktop_email_client_shared::{ApiClient, EmailProvider, EmailProviderType};
use gloo::utils::document;
use web_sys::{HtmlInputElement, wasm_bindgen::JsCast};
use yew::prelude::*;
use yew_autoprops::autoprops;

use crate::use_app_state;

#[autoprops]
#[component]
pub fn AddEmailAccountScreen(close_callback: Callback<()>) -> Html {
	let app_state = use_app_state();

	let step = use_state(|| AddEmailAccountStep::SelectProvider {
		provider_type: EmailProviderType::ManualImapSmtp,
	});

	let view = match &*step {
		AddEmailAccountStep::SelectProvider {
			provider_type: selected_provider_type,
		} => {
			let step = step.clone();
			html! {
				<div>
					<h1>{"Pick an email provider:"}</h1>

					<div class="divider" />

					<div class="flex flex-col gap-4">
						for provider_type in EmailProviderType::ALL.iter() {
							<EmailProviderCard
								provider_type={*provider_type}
								selected={selected_provider_type == provider_type}
								on_click={
									let step = step.clone();
									move |_| step.set(AddEmailAccountStep::SelectProvider { provider_type: *provider_type })
								}
							/>
						}
					</div>
				</div>
			}
		}
		AddEmailAccountStep::Authenticate { provider_type } => {
			let provider_type = *provider_type;

			let provider_specific_fields = match provider_type {
				EmailProviderType::ManualImapSmtp => html! {
					<>
						<label class="label">{"Host"}</label>
						<label class="input">
							// <img src="/static/icons/person.svg" class="h-[1em] opacity-50" />
							<input id="add_email_account_imap_smtp_host" type="text" />
						</label>

						<label class="label">{"Username"}</label>
						<label class="input">
							<img src="/static/icons/person.svg" class="h-[1em] opacity-50" />
							<input id="add_email_account_imap_smtp_username" type="text" />
						</label>
						<p class="label pb-2">{"Can be the email address if you don't have a username"}</p>

						<label class="label">{"Password"}</label>
						<label class="input">
							<img src="/static/icons/key.svg" class="h-[1em] opacity-50" />
							<input id="add_email_account_imap_smtp_password" type="password" />
						</label>
					</>
				},
				EmailProviderType::Mock => Html::default(),
			};

			html! {
				<form class="fieldset bg-base-200 border-base-300 rounded-box w-xs border p-4" style="width: fit-content;">

					<label class="label">{"Name"}</label>
					<input id="add_email_account_name" class="input" type="text" />
					<p class="label pb-2">{"Only used to display instead of the full address"}</p>

					<label class="label">{"Email address"}</label>
					<label class="input validator">
						<img src="/static/icons/envelope.svg" class="h-[1em] opacity-50" />
						<input id="add_email_account_address" type="email" />
					</label>

					<div class="divider">{provider_type.pretty_name()}</div>

					{provider_specific_fields}

				</form>
			}
		}
	};

	let finish_adding_email_account = {
		let step = step.clone();
		let close_callback = close_callback.clone();
		move || {
			let get_input_value_by_id = |id: &str| -> String {
				document()
					.get_element_by_id(id)
					.unwrap()
					.dyn_into::<HtmlInputElement>()
					.unwrap()
					.value()
			};

			let name = get_input_value_by_id("add_email_account_name");
			let address = get_input_value_by_id("add_email_account_address");
			let email_provider = match *step {
				AddEmailAccountStep::Authenticate {
					provider_type: EmailProviderType::Mock,
				} => Some(EmailProvider::Mock),
				AddEmailAccountStep::Authenticate {
					provider_type: EmailProviderType::ManualImapSmtp,
				} => {
					let imap_host = get_input_value_by_id("add_email_account_imap_smtp_host");
					let imap_username =
						get_input_value_by_id("add_email_account_imap_smtp_username");
					let imap_password =
						get_input_value_by_id("add_email_account_imap_smtp_password");

					Some(EmailProvider::ManualImapSmtp {
						imap_host,
						imap_username,
						imap_password,
					})
				}
				_ => None,
			};
			if let Some(email_provider) = email_provider {
				let app_state = app_state.clone();
				wasm_bindgen_futures::spawn_local(async move {
					let _ = app_state
						.api_client
						.insert_email_account(name, address, email_provider)
						.await;
				});
				close_callback.emit(());
			}
		}
	};

	let on_back = {
		let step = step.clone();
		let close_callback = close_callback.clone();
		move || {
			if step.is_first() {
				close_callback.emit(())
			} else {
				step.set(step.prev())
			}
		}
	};

	let on_continue = {
		let step = step.clone();
		move || {
			if step.is_last() {
				finish_adding_email_account();
			} else {
				step.set(step.next())
			}
		}
	};

	html! {
		<div class="w-full h-full p-4 flex flex-col gap-4">
			<div class="m-auto">{view}</div>

			<div class="flex flex-row gap-4 mt-auto">
				<button
					class="btn btn-primary w-24"
					onclick={move |_| on_back()}
				>
					{"Back"}
				</button>

				<ul class="steps ml-auto mr-auto">
					<li class={classes!("step", "text-sm", "step-success")}>
						{"Select Provider"}
					</li>
					<li class={classes!("step", "text-sm", if matches!(*step, AddEmailAccountStep::Authenticate { .. }) { "step-success" } else { "" })}>
						{"Authenticate"}
					</li>
				</ul>

				<button
					class="btn btn-primary w-24"
					onclick={move |_| on_continue()}
				>
					{if step.is_last() {
						"Finish"
					} else {
						"Continue"
					}}
				</button>
			</div>
		</div>
	}
}

#[derive(Debug, Clone, Copy)]
enum AddEmailAccountStep {
	SelectProvider { provider_type: EmailProviderType },
	Authenticate { provider_type: EmailProviderType },
}
impl AddEmailAccountStep {
	fn is_first(&self) -> bool {
		matches!(self, AddEmailAccountStep::SelectProvider { .. })
	}

	fn is_last(&self) -> bool {
		matches!(self, AddEmailAccountStep::Authenticate { .. })
	}

	fn prev(&self) -> Self {
		match self {
			AddEmailAccountStep::SelectProvider { .. } => *self,
			AddEmailAccountStep::Authenticate { provider_type } => {
				AddEmailAccountStep::SelectProvider {
					provider_type: *provider_type,
				}
			}
		}
	}

	fn next(&self) -> Self {
		match self {
			AddEmailAccountStep::SelectProvider { provider_type } => {
				AddEmailAccountStep::Authenticate {
					provider_type: *provider_type,
				}
			}
			AddEmailAccountStep::Authenticate { .. } => *self,
		}
	}
}

#[autoprops]
#[component]
fn EmailProviderCard(
	provider_type: &EmailProviderType,
	selected: bool,
	on_click: Callback<()>,
) -> Html {
	html! {
		<button
			class={classes!("btn", "btn-xl", "w-90", "border-2", "bg-primary", "cursor-pointer", "border-solid", "border-3", "transition-all", if selected { "border-neutral-200" } else { "border-transparent" })}
			onclick={move |_| on_click.emit(())}
			role="radio"
			aria-checked={if selected { "true" } else { "false" }}
		>
			<div class="card-body">
				<h2 class="card-title">{provider_type.pretty_name()}</h2>
			</div>
		</button>
	}
}
