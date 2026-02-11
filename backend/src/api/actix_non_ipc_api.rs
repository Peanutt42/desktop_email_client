use desktop_email_client_shared::{
	GetEmail, GetEmailAccountCount, GetEmailAccounts, GetEmailFolders, GetEmailsInFolder,
	GetEmailsMatchingSearch, MarkEmailRead,
};

macro_rules! route_backend_api {
	($request_type:ty, $service_cfg:expr, $backend_fn_name:ident) => {{
		let route_name = format!(
			"/non_ipc_api/{}",
			<$request_type as desktop_email_client_shared::ApiRequest>::NAME
		);

		$service_cfg.route(
			&route_name,
			actix_web::web::post().to(
				async |backend: actix_web::web::Data<std::sync::Arc<crate::Backend>>,
				       args: actix_web::web::Json<
					<$request_type as desktop_email_client_shared::ApiRequest>::Args,
				>|
				       -> actix_web::web::Json<
					<$request_type as desktop_email_client_shared::ApiRequest>::Output,
				> { actix_web::web::Json(backend.$backend_fn_name(args.0)) },
			),
		);
	}};
}

pub fn configure_backend(cfg: &mut actix_web::web::ServiceConfig) {
	route_backend_api!(GetEmailAccountCount, cfg, get_email_account_count);
	route_backend_api!(GetEmailAccounts, cfg, get_email_accounts);
	route_backend_api!(GetEmail, cfg, get_email);
	route_backend_api!(GetEmailFolders, cfg, get_email_folders);
	route_backend_api!(GetEmailsInFolder, cfg, get_emails_in_folder);
	route_backend_api!(GetEmailsMatchingSearch, cfg, get_emails_matching_search);
	route_backend_api!(MarkEmailRead, cfg, mark_email_read);
}
