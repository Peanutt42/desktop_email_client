use actix_cors::Cors;
use actix_files::Files;
use actix_web::{
	App, HttpServer,
	http::header::{AUTHORIZATION, CONTENT_TYPE},
	web::Data,
};
use desktop_email_client_backend::{
	Backend, Database,
	api::{DatabaseChangedSseState, configure_actix_backend_api_routes},
	init_tracing,
};
use desktop_email_client_shared::DatabaseChangedEvent;
use std::sync::Arc;
use tokio::sync::broadcast;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	init_tracing();

	dotenvy::dotenv().expect("failed to load .env file");

	let dist_path = std::env::var("DIST_PATH").unwrap_or_else(|_| "../frontend/dist".to_string());
	let port: u16 = std::env::var("PORT")
		.unwrap_or_else(|_| "8080".to_string())
		.parse()
		.unwrap_or(8080);

	let url = dotenvy::var("DATABASE_URL")
		.expect("`DATABASE_URL` env var not set, please set it (temporary or in .env file)");

	let database = Database::init_from_url(&url).await;

	if database.get_email_accounts().await.is_empty() {
		database.populate_with_mock_data().await;
	}

	let (database_changed_tx, database_changed_rx) = broadcast::channel(10);
	let database_changed_sse_state = DatabaseChangedSseState {
		database_changed_rx,
	};
	let on_database_update = move |event: DatabaseChangedEvent| {
		tracing::debug!("emit db update: {:?}", event);

		if let Err(e) = database_changed_tx.send(event) {
			tracing::error!("failed to broadcast database changed event to SSE: {}", e);
		}
	};

	let backend = Arc::new(Backend::new(database, Box::new(on_database_update)).await);

	HttpServer::new(move || {
		let cors = Cors::default()
			.allowed_origin(&format!("http://localhost:{}", port))
			.allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
			.allowed_headers(vec![CONTENT_TYPE, AUTHORIZATION])
			.supports_credentials();

		App::new()
			.wrap(cors)
			.app_data(Data::new(backend.clone()))
			.app_data(Data::new(database_changed_sse_state.clone()))
			.configure(configure_actix_backend_api_routes)
			.service(Files::new("/", &dist_path).index_file("index.html"))
	})
	.bind(("localhost", port))?
	.run()
	.await
}
