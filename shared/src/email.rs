use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmailBody {
	TextOnly(String),
	Html(String),
}
impl EmailBody {
	pub fn text_only(text: impl Into<String>) -> Self {
		Self::TextOnly(text.into())
	}
	pub fn html(html: impl Into<String>) -> Self {
		Self::Html(html.into())
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Email {
	pub id: i64,
	pub email_account_id: i64,
	pub author_name: String,
	pub author_address: String,
	pub subject: String,
	pub body: EmailBody,
	pub sent_time: DateTime<Utc>,
	pub read: bool,
	pub folder_id: i64,
	pub tags: Vec<String>,
}

/// represents one of the user added email accounts (not a general email address)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct EmailAccount {
	pub id: i64,
	pub name: String,
	pub address: String,
}

// TODO: should we include email_account_id? leaning no
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EmailFolder {
	pub id: i64,
	pub name: String,
	pub subfolders: Vec<EmailFolder>,
}
