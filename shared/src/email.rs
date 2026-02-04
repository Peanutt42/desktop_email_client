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
	pub author: String,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmailAccountSelection {
	All,
	Single { index: u32 },
}
impl EmailAccountSelection {
	pub fn next(&self, email_count: u32) -> Self {
		Self::Single {
			index: match self {
				Self::All => 0,
				Self::Single { index } => (index + 1) % email_count,
			},
		}
	}
	pub fn prev(&self, email_count: u32) -> Self {
		Self::Single {
			index: match self {
				Self::All => email_count - 1,
				Self::Single { index } => (index - 1) % email_count,
			},
		}
	}
	pub fn all_selected(&self) -> bool {
		matches!(self, Self::All)
	}
	/// returns true if all email accounts are selected, or just this one
	pub fn is_selected(&self, email_account_index: u32) -> bool {
		match self {
			Self::All => true,
			Self::Single { index } => *index == email_account_index,
		}
	}
	pub fn as_i64(&self) -> i64 {
		match self {
			Self::All => -1,
			Self::Single { index } => *index as i64,
		}
	}
	/// Ctrl+'key_digit'
	pub fn from_keyboard_shortcut(key_digit: u8, total_email_accounts_count: u32) -> Self {
		match key_digit {
			1 => EmailAccountSelection::All,
			_ => {
				let index = key_digit as u32 - 2;
				if (index) < total_email_accounts_count {
					EmailAccountSelection::Single { index }
				} else {
					EmailAccountSelection::All
				}
			}
		}
	}
}

/*	pub fn create_mock() -> Self {
	let inbox_folder = EmailFolder::single("Inbox");
	let friends_folder = EmailFolder::single("Friends");
	let moodle_folder = EmailFolder::single("Moodle");
	let important_folder = EmailFolder::new(
		"Important",
		vec![friends_folder.clone(), moodle_folder.clone()],
	);
	let not_important_folder = EmailFolder::single("Not important");
	let status_system_folder = EmailFolder::single("Status system");

	Self::Fake {
		email_folders: vec![
			inbox_folder.clone(),
			important_folder,
			not_important_folder.clone(),
			status_system_folder.clone()
		],
		emails: [
			Rc::new(Email {
				subject: "Hello".into(),
				author: "alice@example.com".into(),
				body: EmailBody::text_only("Hello from Alice"),
				sent_time: Local::now().with_timezone(&Utc) - std::time::Duration::from_mins(1),
				folder_tags: [friends_folder.uuid].into(),
			}),
			Rc::new(Email {
				subject: "Status update".into(),
				author: "bob@example.com".into(),
				body: EmailBody::text_only("All systems operational."),
				sent_time: Local::now().with_timezone(&Utc)
					- std::time::Duration::from_mins(45),
				folder_tags: [status_system_folder.uuid, not_important_folder.uuid].into(),
			}),
			Rc::new(Email {
				subject: "Very looooooooooong subject title, so long that i dont know what to write anymore...".into(),
				author: "loooooong@example.com".into(),
				body: EmailBody::html("<h1>Loooooong</h1> <br>".repeat(20)),
				sent_time: Local::now().with_timezone(&Utc)
					- std::time::Duration::from_hours(24 * 7),
				folder_tags: [inbox_folder.uuid].into(),
			}),
			Rc::new(Email {
				subject: "Very looooooooooong subject title, so long that i dont know what to write anymore... (text only)".into(),
				author: "loooooong@example.com".into(),
				body: EmailBody::text_only("Loooooong\n".repeat(20)),
				sent_time: Local::now().with_timezone(&Utc)
					- std::time::Duration::from_hours(24 * 7)
					- std::time::Duration::from_mins(5),
				folder_tags: [inbox_folder.uuid].into(),
			}),
			Rc::new(Email {
				subject: "Moodle: You have received feedback!".into(),
				author: "noreply@moodle.com".into(),
				body: EmailBody::text_only("Look on the moodle website for feedback"),
				sent_time: Local::now().with_timezone(&Utc)
					- std::time::Duration::from_mins(120),
				folder_tags: [moodle_folder.uuid].into(),
			}),
			Rc::new(Email {
				subject: "New Login".into(),
				author: "noreply@bitwarden.com".into(),
				body: EmailBody::text_only(
					"A new device has logged into your bitwarden account!",
				),
				sent_time: Local::now().with_timezone(&Utc)
					- std::time::Duration::from_mins(10),
				folder_tags: [not_important_folder.uuid].into(),
			}),
			Rc::new(Email {
				subject: "Something important from long ago...".to_string(),
				author: "i dont know :/".to_string(),
				body: EmailBody::text_only("i forgor :("),
				sent_time: Local::now().with_timezone(&Utc) - chrono::TimeDelta::weeks(120),
				folder_tags: HashSet::new(),
			}),
		]
		.into_iter()
		.map(|email| (Uuid::new_v4(), email))
		.collect::<HashMap<Uuid, Rc<Email>>>(),
	}
}*/
