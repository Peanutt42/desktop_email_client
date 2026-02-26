use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// guaranteed to be max 50 chars
pub struct EmailBodySummary(String);
impl EmailBodySummary {
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
	pub fn take_str(self) -> String {
		self.0
	}
}
impl TryFrom<&str> for EmailBodySummary {
	type Error = &'static str;
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		if value.len() > 50 {
			Err("Summary too long")
		} else {
			Ok(Self(value.to_string()))
		}
	}
}

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
	pub fn is_html(&self) -> bool {
		matches!(self, EmailBody::Html(_))
	}
	pub fn content_str(self) -> String {
		match self {
			EmailBody::TextOnly(content) => content,
			EmailBody::Html(content) => content,
		}
	}
	/// TODO: Add body summary extraction from html body
	// summary is max 50 chars
	pub fn extract_summary(&self) -> Option<EmailBodySummary> {
		match self {
			Self::TextOnly(text) => text
				.chars()
				.take(50)
				.collect::<String>()
				.as_str()
				.try_into()
				.ok(),
			Self::Html(_) => None,
		}
	}
}

#[derive(Debug, Clone)]
pub struct ReceivedEmail {
	pub email_account_id: i64,
	pub envelope: EmailEnvelope,
	pub body: EmailBody,
	pub folder_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Email {
	pub email_account_id: i64,
	#[serde(flatten)]
	pub envelope: EmailEnvelope,
	pub body: EmailBody,
	pub body_summary: Option<String>,
	pub read: bool,
	pub folder_id: i64,
	pub tags: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EmailEnvelope {
	pub author_name: String,
	pub author_address: String,
	pub subject: String,
	pub sent_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EmailRow {
	pub id: i64,
	#[serde(flatten)]
	pub email: Email,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmailInfo {
	pub email_id: i64,
	pub email_account_id: i64,
	#[serde(flatten)]
	pub envelope: EmailEnvelope,
	/// limited to 50 chars
	pub body_summary: Option<String>,
	pub read: bool,
	pub tags: Vec<String>,
}

/// represents one of the user added email accounts (not a general email address)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EmailAccount {
	pub id: i64,
	pub name: String,
	pub address: String,
	pub provider: EmailProvider,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum EmailProvider {
	ManualImapSmtp {
		imap_host: String,
		imap_username: String,
		imap_password: String,
	},
	Mock,
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum EmailProviderType {
	ManualImapSmtp,
	Mock,
}
impl EmailProviderType {
	pub const ALL: &'static [EmailProviderType] =
		&[EmailProviderType::ManualImapSmtp, EmailProviderType::Mock];

	pub fn as_str(&self) -> &'static str {
		match self {
			Self::ManualImapSmtp => "manual_imap_smtp",
			Self::Mock => "mock",
		}
	}

	pub fn pretty_name(&self) -> &'static str {
		match self {
			Self::ManualImapSmtp => "Manual (IMAP/SMTP)",
			Self::Mock => "Mock",
		}
	}
}
impl FromStr for EmailProviderType {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"manual_imap_smtp" => Ok(Self::ManualImapSmtp),
			"mock" => Ok(Self::Mock),
			_ => Err(()),
		}
	}
}
impl EmailProvider {
	pub fn get_type(&self) -> EmailProviderType {
		match self {
			Self::ManualImapSmtp { .. } => EmailProviderType::ManualImapSmtp,
			Self::Mock => EmailProviderType::Mock,
		}
	}
}

// TODO: should we include email_account_id? leaning no
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EmailFolder {
	pub id: i64,
	pub name: String,
	pub subfolders: Vec<EmailFolder>,
}
