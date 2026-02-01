mod email;
pub use email::{
	Email, EmailAccount, EmailAccountSelection, EmailBody, EmailFolder, EmailProvider,
	get_email_folder_by_uuid,
};
use yew_router::Routable;

#[derive(Debug, Clone, Routable, PartialEq, Eq)]
pub enum Route {
	#[at("/index.html")]
	Index,
}
