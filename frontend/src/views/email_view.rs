use std::rc::Rc;

use desktop_email_client::{Email, EmailBody};
use yew::prelude::*;
use yew_autoprops::autoprops;

#[autoprops]
#[component]
pub fn EmailView(email: &Rc<Email>) -> Html {
	html! {
		<div class="flex flex-col w-full h-full">
			<div class="border-b border-gray-300 p-2">
				<div class="text-xl font-bold truncate">
					{&email.subject}
				</div>

				<div class="flex flex-row gap-1.5">
					<div class="select-none">{ "From: " }</div>

					<div class="select-all">{ &email.author }</div>
				</div>
			</div>

			<div class="p-2 h-full">
				<EmailBodyView email={email} />
			</div>
		</div>
	}
}

#[autoprops]
#[component]
pub fn EmailBodyView(email: &Rc<Email>) -> Html {
	match email.body.clone() {
		EmailBody::TextOnly(text_body) => html! {
			<div>{ text_body }</div>
		},
		EmailBody::Html(html_body) => html! {
			<iframe
				class="bg-white w-full h-full border-0 rounded-[7px] box-border overflow-y-auto"
				sandbox="true"
				srcdoc={html_body}
			/>
		},
	}
}
