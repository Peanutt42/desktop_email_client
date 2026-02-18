use crate::{Email, EmailAccount, EmailFolder, api};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmailInfo {
	pub email_id: i64,
	pub email_account_id: i64,
	pub author_name: String,
	pub author_address: String,
	pub subject: String,
	/// limited to 50 chars
	pub body_summary: Option<String>,
	pub sent_time: DateTime<Utc>,
	pub read: bool,
	pub folder_id: i64,
	pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmailFilter {
	Any,
	Folder { folder_id: i64 },
	MatchingSearch { search: String },
}

api! {
	async fn get_email_accounts() -> Vec<EmailAccount>;
	async fn get_email(email_id: i64) -> Option<Email>;
	async fn get_emails(email_filter: EmailFilter) -> Vec<EmailInfo>;
	async fn get_email_folders(email_id: i64) -> Vec<EmailFolder>;
	async fn mark_email_read(email_id: i64) -> ();
}

/// simply used such that all tauri commands just have this one argument of type `ArgsWrapper`
/// such that we dont have to "curry" and "uncurry" between invoking tauri ipc command and handeling the ipc command
#[derive(Debug, Serialize, Deserialize)]
pub struct TauriCommandArgsWrapper<Args> {
	pub args: Args,
}

pub const DEV_NON_IPC_API_ROUTE: &str = "/non_ipc_api";
