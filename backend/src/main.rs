use desktop_email_client_backend::{Database, init_tracing, run_server};
use std::path::PathBuf;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	init_tracing();

	let _ = dotenvy::dotenv();

	let frontend_dist_dir =
		std::env::var("FRONTEND_DIST_PATH").unwrap_or_else(|_| "../frontend/dist".to_string());

	let url = std::env::var("DATABASE_URL")
		.expect("`DATABASE_URL` env var not set, please set it (temporary or in .env file)");

	let database = Database::init_from_url(&url).await;

	if database.get_email_accounts().await.is_empty() {
		database.populate_with_mock_data().await;
	}

	run_server(database, PathBuf::from(frontend_dist_dir)).await
}
