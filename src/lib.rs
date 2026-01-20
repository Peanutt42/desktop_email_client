mod email;
pub use email::{Email, EmailAccount, EmailAccountSelection, EmailBody, EmailProvider};
use yew_router::Routable;

#[derive(Debug, Clone, Routable, PartialEq, Eq)]
pub enum Route {
	#[at("/index.html")]
	Index,
}
