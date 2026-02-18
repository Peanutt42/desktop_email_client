use desktop_email_client_backend::{Backend, init_db};
use tauri::{App, Manager};

fn main() {
	tracing_subscriber::fmt::fmt().without_time().init();

	let builder = tauri::Builder::default();

	#[cfg(not(feature = "non_ipc_backend"))]
	let builder = builder.invoke_handler(tauri::generate_handler![
		desktop_email_client_backend::api::tauri_ipc_api_request_handler
	]);

	builder
		.setup(|app: &mut App| {
			let sqlite_db_filepath = app
				.path()
				.app_local_data_dir()
				.expect("failed to get local data directory")
				.join("db.sqlite");

			let db_pool = tauri::async_runtime::block_on(init_db(&sqlite_db_filepath));

			let backend = Backend::new(db_pool);

			app.manage(backend);

			let window = app.get_webview_window("main").unwrap();
			window.set_decorations(should_have_window_decorations())?;
			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

fn should_have_window_decorations() -> bool {
	if cfg!(target_os = "linux") {
		let xdg_current_desktop = std::env::var("XDG_CURRENT_DESKTOP")
			.unwrap_or_default()
			.to_lowercase();

		// TODO: incomplete list of tiling window managers
		xdg_current_desktop != "niri"
			&& xdg_current_desktop != "sway"
			&& xdg_current_desktop != "i3"
	} else {
		true
	}
}
