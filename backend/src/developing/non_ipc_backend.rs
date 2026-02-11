use actix_cors::Cors;
use actix_web::{
	App, HttpServer,
	http::header::{AUTHORIZATION, CONTENT_TYPE},
	web::Data,
};
use desktop_email_client_backend::{Backend, configure_backend};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	tracing_subscriber::fmt::fmt().without_time().init();

	HttpServer::new(move || {
		let backend = Arc::new(Backend::create_mock());

		let cors = Cors::default()
			.allowed_origin("http://localhost:8080")
			.allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
			.allowed_headers(vec![CONTENT_TYPE, AUTHORIZATION])
			.supports_credentials();

		App::new()
			.wrap(cors)
			.app_data(Data::new(backend.clone()))
			.configure(configure_backend)
	})
	.bind(("localhost", 4242))?
	.run()
	.await
}
