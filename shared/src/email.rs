use chrono::{DateTime, Utc};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
	pub subject: String,
	pub author: EmailAccount,
	pub body: EmailBody,
	pub sent_time: DateTime<Utc>,
	pub read: bool,
	pub folder_uuid: Uuid,
	pub tags: Vec<String>,
}
impl Email {
	// TODO: search email by author and body too, not just subject
	/// returns match scope, None if not matching at all
	/// the higher, the better of a match
	pub fn match_search_pattern(&self, search_pattern: &str) -> Option<i64> {
		SkimMatcherV2::default().fuzzy_match(&self.subject, search_pattern)
	}
}

/// represents one of the user added email accounts (not a general email address)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EmailAccount {
	pub name: String,
	pub address: String,
}
impl EmailAccount {
	pub fn new(name: String, address: String) -> Self {
		Self { name, address }
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EmailFolder {
	pub uuid: Uuid,
	pub name: String,
	pub subfolders: Vec<EmailFolder>,
}
impl EmailFolder {
	pub fn new(name: impl Into<String>, subfolders: Vec<EmailFolder>) -> Self {
		Self {
			uuid: Uuid::new_v4(),
			name: name.into(),
			subfolders,
		}
	}

	pub fn single(name: impl Into<String>) -> Self {
		Self {
			uuid: Uuid::new_v4(),
			name: name.into(),
			subfolders: vec![],
		}
	}
}
/// recursively searches folders and their subfolders for folder with given uuid
pub fn get_email_folder_by_uuid(folders: &[EmailFolder], uuid: Uuid) -> Option<&EmailFolder> {
	for folder in folders {
		if folder.uuid == uuid {
			return Some(folder);
		}
		if let Some(folder) = get_email_folder_by_uuid(&folder.subfolders, uuid) {
			return Some(folder);
		}
	}

	None
}
