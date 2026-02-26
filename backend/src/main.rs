use desktop_email_client_backend::{Backend, Database, init_tracing};
use desktop_email_client_shared::{DATABASE_CHANGED_EVENT_NAME, DatabaseChangedEvent};
use tauri::{App, Emitter, Manager};

fn main() {
	init_tracing();

	let builder = tauri::Builder::default();

	#[cfg(not(feature = "non_ipc_backend"))]
	let builder = builder.invoke_handler(tauri::generate_handler![
		desktop_email_client_backend::api::tauri_ipc_api_request_handler
	]);

	builder
		.setup(|app: &mut App| {
			let app_data_dir = app
				.path()
				.app_local_data_dir()
				.expect("failed to get local data directory");

			let sqlite_db_filepath = app_data_dir.join("db.sqlite");

			let database =
				tauri::async_runtime::block_on(Database::init_from_file(&sqlite_db_filepath));

			let app_handle = app.handle().clone();

			let on_database_update = move |event: DatabaseChangedEvent| {
				tracing::debug!("emit db update: {:?}", event);
				app_handle.emit(DATABASE_CHANGED_EVENT_NAME, event).unwrap();
			};

			let backend = tauri::async_runtime::block_on(Backend::new(
				database,
				Box::new(on_database_update),
			));

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
			&& xdg_current_desktop != "Hyprland"
	} else {
		true
	}
}
