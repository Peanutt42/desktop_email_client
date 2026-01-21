use std::sync::Arc;

use actix_files::Files;
use actix_web::{HttpServer, middleware::Logger};
use tokio::sync::Mutex;

pub async fn run_server(bind_address: &str) -> std::io::Result<()> {
	tracing::info!("starting actix server");

	let _state = ();
	let _shared_state = Arc::new(Mutex::new(_state));

	HttpServer::new(move || {
		let _shared_state = _shared_state.clone();

		actix_web::App::new()
			.wrap(Logger::default())
			.configure(move |cfg| {
				// TODO: configure_routes(cfg, shared_state);
				// cfg.service(Files::new("/", "../frontend/dist/").index_file("index.html"));
				cfg.service(
					Files::new(
						"/",
						"/home/peter/github/desktop_email_client/frontend/dist/",
					)
					.index_file("index.html"),
				);
			})
	})
	.bind(bind_address)?
	.run()
	.await
}
