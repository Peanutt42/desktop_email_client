use crate::Backend;
use actix_web::web::{Data, Json, ServiceConfig, post};
use desktop_email_client_shared::{Api, DEV_NON_IPC_API_ROUTE, Request, Response};
use std::sync::Arc;

pub fn configure_actix_backend_api_routes(service_cfg: &mut ServiceConfig) {
	service_cfg.route(
		DEV_NON_IPC_API_ROUTE,
		post().to(
			async |backend: Data<Arc<Backend>>, request: Json<Request>| -> Json<Response> {
				let request = request.into_inner();
				tracing::debug!("{}", request);
				let response = backend.get_ref().dispatch(request).await;
				Json(response)
			},
		),
	);
}
