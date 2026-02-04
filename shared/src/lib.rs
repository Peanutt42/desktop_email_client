mod email;
pub use email::{
	Email, EmailAccount, EmailAccountSelection, EmailBody, EmailFolder, get_email_folder_by_uuid,
};

mod api_macros;

mod api;

#[cfg(feature = "backend")]
pub use api::BackendApi;

pub use api::EmailSearchResult;

#[cfg(feature = "frontend")]
pub use api::{
	get_email, get_email_account_count, get_email_accounts, get_email_folders,
	get_emails_in_folder, get_emails_matching_search, mark_email_read,
};
