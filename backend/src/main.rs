use desktop_email_client_backend::{Database, init_tracing, run_server};
use std::path::PathBuf;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	init_tracing();

	dotenvy::dotenv().expect("failed to load .env file");

	let frontend_dist_dir =
		std::env::var("FRONTEND_DIST_PATH").unwrap_or_else(|_| "../frontend/dist".to_string());
	let port: u16 = std::env::var("PORT")
		.ok()
		.and_then(|port_str| port_str.parse().ok())
		.unwrap_or(8080);

	let url = dotenvy::var("DATABASE_URL")
		.expect("`DATABASE_URL` env var not set, please set it (temporary or in .env file)");

	let database = Database::init_from_url(&url).await;

	if database.get_email_accounts().await.is_empty() {
		database.populate_with_mock_data().await;
	}

	run_server(database, port, PathBuf::from(frontend_dist_dir)).await
}
