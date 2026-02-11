use desktop_email_client_backend::Backend;
use tauri::Manager;

fn main() {
	tracing_subscriber::fmt::fmt().without_time().init();

	let backend = Backend::create_mock();

	let builder = tauri::Builder::default().manage(backend);

	#[cfg(not(feature = "non_ipc_backend"))]
	let builder = builder.invoke_handler(tauri::generate_handler![
		tauri_commands::get_email_account_count,
		tauri_commands::get_email_accounts,
		tauri_commands::get_email,
		tauri_commands::get_email_folders,
		tauri_commands::get_emails_in_folder,
		tauri_commands::get_emails_matching_search,
		tauri_commands::mark_email_read
	]);

	builder
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

#[cfg(not(feature = "non_ipc_backend"))]
mod tauri_commands {
	use desktop_email_client_backend::{
		generate_tauri_command_for_backend_api, generate_tauri_command_for_backend_api_no_args,
	};
	use desktop_email_client_shared::{
		GetEmail, GetEmailAccountCount, GetEmailAccounts, GetEmailFolders, GetEmailsInFolder,
		GetEmailsMatchingSearch, MarkEmailRead,
	};

	generate_tauri_command_for_backend_api!(get_email_account_count, GetEmailAccountCount);
	generate_tauri_command_for_backend_api_no_args!(get_email_accounts, GetEmailAccounts);
	generate_tauri_command_for_backend_api!(get_email, GetEmail);
	generate_tauri_command_for_backend_api!(get_email_folders, GetEmailFolders);
	generate_tauri_command_for_backend_api!(get_emails_in_folder, GetEmailsInFolder);
	generate_tauri_command_for_backend_api!(get_emails_matching_search, GetEmailsMatchingSearch);
	generate_tauri_command_for_backend_api!(mark_email_read, MarkEmailRead);
}
