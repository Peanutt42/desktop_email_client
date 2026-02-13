use desktop_email_client_shared::{Email, EmailBody};
use yew::prelude::*;
use yew_autoprops::autoprops;

#[autoprops]
#[component]
pub fn EmailView(email: &Email) -> Html {
	html! {
		<div class="flex flex-col w-full h-full">
			<div class="border-b border-accent flex flex-row">
				<EmailAvatar big=true name={email.author.name.clone()} />

				<div class="p-3">
					<div class="text-xl font-bold break-normal">
						{&email.subject}
					</div>

					<div class="flex flex-row gap-1.5">
						<div class="select-none">{ "From: " }</div>

						<div class="text-nowrap">{ format!("{} ({})", email.author.name, email.author.address) }</div>
					</div>
				</div>
			</div>

			<div class="p-2 h-full">
				<EmailBodyView email={email.clone()} />
			</div>
		</div>
	}
}

#[autoprops]
#[component]
pub fn EmailBodyView(email: &Email) -> Html {
	match email.body.clone() {
		EmailBody::TextOnly(text_body) => html! {
			<div class="whitespace-break-spaces">{ text_body }</div>
		},
		EmailBody::Html(html_body) => html! {
			<iframe
				class="bg-white w-full h-full border-0 rounded-[7px] box-border overflow-y-auto"
				sandbox=""
				srcdoc={html_body}
			/>
		},
	}
}

#[autoprops]
#[component]
pub fn EmailAvatar(name: AttrValue, #[prop_or_default] big: bool) -> Html {
	let base_classes = classes!("shrink-0", "rounded-full", "bg-neutral");
	let class = if big {
		classes!("w-14", "h-14", "min-w-14", "min-h-14", base_classes)
	} else {
		classes!("w-8", "h-8", "min-w-8", "min-h-8", base_classes)
	};

	let mut initials = String::new();
	let mut prev_space = true;
	for c in name.as_str().chars() {
		if prev_space && !c.is_ascii_whitespace() {
			initials.push(c.to_ascii_uppercase());
			prev_space = false;
		} else {
			prev_space = c.is_ascii_whitespace();
		}
	}

	html! {
		<div class={classes!("avatar", "avatar-placeholder", if big { Some("p-2") } else { None })}>
			<div class={class}>
				<span class={if big { "text-3xl" } else { "text-xs" }}>{initials}</span>
			</div>
		</div>
	}
}
