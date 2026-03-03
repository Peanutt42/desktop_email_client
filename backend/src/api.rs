use crate::Backend;
use actix_web::web::{Data, Json, ServiceConfig, get, post};
use actix_web::{HttpResponse, Responder};
use actix_web_lab::sse::{self, Sse};
use desktop_email_client_shared::{
	API_ROUTE, Api, DATABASE_CHANGED_EVENT_NAME, DatabaseChangedEvent, Request, Response,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

pub fn configure_api_routes(service_cfg: &mut ServiceConfig) {
	service_cfg.route(
		API_ROUTE,
		post().to(
			async |backend: Data<Arc<Backend>>, request: Json<Request>| -> Json<Response> {
				let request = request.into_inner();
				tracing::debug!("{}", request);
				let response = backend.get_ref().dispatch(request).await;
				Json(response)
			},
		),
	);

	service_cfg.route(
		&format!("{}/{}", API_ROUTE, DATABASE_CHANGED_EVENT_NAME),
		get().to(database_changed_sse),
	);

	service_cfg.route("/api/health", get().to(HttpResponse::Ok));
}

pub struct DatabaseChangedSseState {
	pub database_changed_rx: broadcast::Receiver<DatabaseChangedEvent>,
}
impl Clone for DatabaseChangedSseState {
	fn clone(&self) -> Self {
		Self {
			database_changed_rx: self.database_changed_rx.resubscribe(),
		}
	}
}

async fn database_changed_sse(state: Data<DatabaseChangedSseState>) -> impl Responder {
	let mut rx = state.database_changed_rx.resubscribe();

	let stream = async_stream::stream! {
		loop {
			match rx.recv().await {
				Ok(event) => {
					let data = serde_json::to_string(&event).unwrap();
					yield Ok::<_, actix_web::Error>(
						sse::Event::Data(sse::Data::new(data))
					);
				}
				Err(broadcast::error::RecvError::Lagged(_)) => continue,
				Err(broadcast::error::RecvError::Closed) => break,
			}
		}
	};

	Sse::from_stream(stream).with_keep_alive(Duration::from_secs(15))
}
