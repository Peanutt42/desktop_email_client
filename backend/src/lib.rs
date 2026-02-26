mod backend;
pub use backend::Backend;

pub mod api;

mod imap;
pub use imap::{ImapListener, ImapListenerError};

mod db;
pub use db::Database;

mod email_account_provider_store;
pub use email_account_provider_store::{read_email_account_provider, save_email_account_provider};

pub fn init_tracing() {
	use tracing_subscriber::EnvFilter;

	tracing_subscriber::fmt::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.without_time()
		.init();
}
