use crate::{Email, EmailAccount, EmailAccountSelection, EmailFolder, tauri_backend_api};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSearchResult {
	pub email_uuid: Uuid,
	pub email: Email,
}

tauri_backend_api! {
	fn get_email_account_count() -> usize;

	fn get_email_accounts() -> Vec<EmailAccount>;

	fn get_email_folders(email_account_index: usize) -> Vec<EmailFolder>;

	fn get_emails_in_folder(email_account_selection: EmailAccountSelection, email_folder_uuid: Option<Uuid>) -> Vec<EmailSearchResult>;

	fn get_emails_matching_search(email_account_selection: EmailAccountSelection, search: String) -> Vec<EmailSearchResult>;

	fn get_email(email_uuid: Uuid) -> Option<Email>;

	fn mark_email_read(email_uuid: Uuid) -> ();
}
