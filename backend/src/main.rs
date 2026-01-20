use tauri::Manager;

mod server;
use server::run_server;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();

	let address = "127.0.0.1:8080";
	let server_url = format!("http://{}", address);

	std::thread::spawn(move || {
		actix_rt::System::new().block_on(async move {
			run_server(address)
				.await
				.expect("failed to serve actix server");
		});
	});

	tauri::Builder::default()
		.setup(move |app| {
			let window = app.get_webview_window("main").unwrap();

			tracing::info!("going to initial index.html page of actix server");
			window
				.eval(format!("window.location.replace('{}')", server_url))
				.expect("failed to load initial UI");

			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
