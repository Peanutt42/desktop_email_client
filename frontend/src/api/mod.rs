#[cfg(feature = "non_ipc_backend")]
mod actix_non_ipc_api;
#[cfg(feature = "non_ipc_backend")]
use actix_non_ipc_api::invoke_backend_api;

#[cfg(not(feature = "non_ipc_backend"))]
mod tauri_ipc_api;
#[cfg(not(feature = "non_ipc_backend"))]
use tauri_ipc_api::invoke_backend_api;

use desktop_email_client_shared::{
	Email, EmailAccount, EmailFilter, EmailFolder, EmailInfo, Request, Response,
};

macro_rules! generate_invoke_backend_api_function {
	($request:ident, ( $($arg:ident : $arg_ty:ty),* ) -> $resp:ty) => {
		pub async fn $request($($arg: $arg_ty,)*) -> $resp {
			let response = invoke_backend_api(Request::$request{ $($arg,)* }).await;
			match response {
				Response::$request(res) => res,
				_ => unreachable!("Request and Response type dont match!"),
			}
		}
	};
}

generate_invoke_backend_api_function!(get_email, (email_id: i64) -> Option<Email>);
generate_invoke_backend_api_function!(get_email_accounts, () -> Vec<EmailAccount>);
generate_invoke_backend_api_function!(get_email_folders, (email_id: i64) -> Vec<EmailFolder>);
generate_invoke_backend_api_function!(get_emails, (email_filter: EmailFilter) -> Vec<EmailInfo>);
generate_invoke_backend_api_function!(mark_email_read, (email_id: i64) -> ());
