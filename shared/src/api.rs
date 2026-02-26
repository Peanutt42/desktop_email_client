use crate::{EmailAccount, EmailFolder, EmailInfo, EmailProvider, EmailRow, api};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmailFilter {
	Any,
	Folder { folder_id: i64 },
	MatchingSearch { search: String },
}

pub const DATABASE_CHANGED_EVENT_NAME: &str = "database_changed";

/// events that get emitted by backend
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseChangedEvent {
	EmailAccountsChanged,
	EmailFolderChanged {
		email_account_id: i64,
		email_folder_id: i64,
	},
	EmailsChanged {
		email_account_id: i64,
		affected_email_folder_id: i64,
	},
}

api! {
	async fn insert_email_account(name: String, address: String, provider: EmailProvider) -> i64;
	async fn remove_email_account(email_account_id: i64) -> ();
	async fn get_email_accounts() -> Vec<EmailAccount>;
	async fn get_email(email_id: i64) -> Option<EmailRow>;
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
