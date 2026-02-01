mod commands;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();

	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![commands::greet])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
