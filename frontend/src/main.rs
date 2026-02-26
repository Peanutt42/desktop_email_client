use tracing_subscriber::{Layer, filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_web::MakeWebConsoleWriter;

fn main() {
	let web_console_layer = tracing_subscriber::fmt::layer()
		.with_ansi(false)
		.without_time()
		.with_writer(MakeWebConsoleWriter::new())
		.with_filter(
			Targets::new()
				.with_target("yew", tracing::Level::DEBUG)
				.with_default(tracing::Level::TRACE),
		);

	tracing_subscriber::registry()
		.with(web_console_layer)
		.init();

	tracing::debug!("Starting desktop_email_client frontend yew application!");

	yew::Renderer::<desktop_email_client_frontend::App>::new().render();
}
