#[macro_export]
macro_rules! generate_tauri_command_for_backend_api {
	($name:ident, $request_type:ty) => {
		#[tauri::command(rename_all = "snake_case")]
		pub fn $name(
			backend: tauri::State<'_, $crate::Backend>,
			args: <$request_type as desktop_email_client_shared::ApiRequest>::Args,
		) -> <$request_type as desktop_email_client_shared::ApiRequest>::Output {
			backend.$name(args)
		}
	};
}
#[macro_export]
macro_rules! generate_tauri_command_for_backend_api_no_args {
	($name:ident, $request_type:ty) => {
		#[tauri::command(rename_all = "snake_case")]
		pub fn $name(
			backend: tauri::State<'_, $crate::Backend>,
		) -> <$request_type as desktop_email_client_shared::ApiRequest>::Output {
			backend.$name(())
		}
	};
}
