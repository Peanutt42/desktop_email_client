use actix_cors::Cors;
use actix_web::{
	App, HttpServer,
	http::header::{AUTHORIZATION, CONTENT_TYPE},
	web::Data,
};
use desktop_email_client_backend::{
	Backend,
	api::{
		DEV_NON_IPC_BACKEND_PORT, DEV_NON_IPC_FRONTEND_PORT, configure_actix_backend_api_routes,
	},
	init_db_from_url,
};
use desktop_email_client_shared::Api;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	tracing_subscriber::fmt::fmt().without_time().init();

	dotenvy::dotenv().expect("failed to load .env file");
	let url = dotenvy::var("DATABASE_URL")
		.expect("`DATABASE_URL` env var not set, please set it (temporary or in .env file)");

	let db_pool = init_db_from_url(&url).await;

	let backend = Arc::new(Backend::new(db_pool));

	if backend.get_email_accounts().await.is_empty() {
		backend.populate_with_mock_data().await;
	}

	HttpServer::new(move || {
		let cors = Cors::default()
			.allowed_origin(&format!("http://localhost:{}", DEV_NON_IPC_FRONTEND_PORT))
			.allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
			.allowed_headers(vec![CONTENT_TYPE, AUTHORIZATION])
			.supports_credentials();

		App::new()
			.wrap(cors)
			.app_data(Data::new(backend.clone()))
			.configure(configure_actix_backend_api_routes)
	})
	.bind(("localhost", DEV_NON_IPC_BACKEND_PORT))?
	.run()
	.await
}
