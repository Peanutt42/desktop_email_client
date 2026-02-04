mod backend;
use backend::Backend;

use desktop_email_client_shared::{
	BackendApi, Email, EmailAccount, EmailAccountSelection, EmailFolder, EmailSearchResult,
	register_tauri_commands,
};
use tauri::Manager;
use uuid::Uuid;

fn main() {
	tracing_subscriber::fmt::fmt().without_time().init();

	let backend = Backend::create_mock();

	tauri::Builder::default()
		.manage(Box::new(backend) as Box<dyn BackendApi>)
		.invoke_handler(register_tauri_commands!(Backend))
		.setup(|app| {
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
