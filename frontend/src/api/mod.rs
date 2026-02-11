#[cfg(feature = "non_ipc_backend")]
mod actix_non_ipc_api;

#[cfg(not(feature = "non_ipc_backend"))]
mod tauri_ipc_api;
#[cfg(not(feature = "non_ipc_backend"))]
pub(crate) use tauri_ipc_api::tauri_wasm_invoke;

use crate::generate_invoke_backend_api_function;

use desktop_email_client_shared::{
	GetEmail, GetEmailAccounts, GetEmailFolders, GetEmailsInFolder, GetEmailsMatchingSearch,
	MarkEmailRead,
};

generate_invoke_backend_api_function!(get_email, GetEmail);
generate_invoke_backend_api_function!(get_email_accounts, GetEmailAccounts);
generate_invoke_backend_api_function!(get_email_folders, GetEmailFolders);
generate_invoke_backend_api_function!(get_emails_in_folder, GetEmailsInFolder);
generate_invoke_backend_api_function!(get_emails_matching_search, GetEmailsMatchingSearch);
generate_invoke_backend_api_function!(mark_email_read, MarkEmailRead);
