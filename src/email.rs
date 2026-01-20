use chrono::{DateTime, Utc};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use std::{collections::HashMap, rc::Rc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email {
	pub subject: String,
	pub author: String,
	pub body: EmailBody,
	pub sent_time: DateTime<Utc>,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAccount {
	pub name: String,
	pub address: String,
	pub provider: EmailProvider,
}
impl EmailAccount {
	pub fn new(name: String, address: String, provider: EmailProvider) -> Self {
		Self {
			name,
			address,
			provider,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailAccountSelection {
	All,
	Single { index: u32 },
}
impl EmailAccountSelection {
	pub fn next(&self, email_count: u32) -> EmailAccountSelection {
		Self::Single {
			index: match self {
				Self::All => 0,
				Self::Single { index } => (index + 1) % email_count,
			},
		}
	}
	pub fn prev(&self, email_count: u32) -> EmailAccountSelection {
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
	/// in htmx keyup syntax
	pub fn get_shortcut_key(&self) -> Option<String> {
		let key = match self {
			Self::All => 1, // Ctrl+1
			Self::Single { index } => {
				let key = *index as usize + 2;
				if key > 9 {
					return None;
				}
				key
			}
		};
		Some(format!("keydown[ctrlKey&&key=='{}']", key))
	}
}

// TOOD: refactor once necessary
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailProvider {
	Fake { emails: HashMap<Uuid, Rc<Email>> },
}
impl EmailProvider {
	pub fn get_emails(&self) -> &HashMap<Uuid, Rc<Email>> {
		match self {
			Self::Fake { emails } => emails,
		}
	}

	pub fn get_emails_mut(&mut self) -> &mut HashMap<Uuid, Rc<Email>> {
		match self {
			Self::Fake { emails } => emails,
		}
	}

	pub fn create_fake() -> Self {
		Self::Fake {
			emails: vec![
				Rc::new(Email {
					subject: "Hello".into(),
					author: "alice@example.com".into(),
					body: EmailBody::text_only("Hello from Alice"),
					sent_time: Utc::now() - std::time::Duration::from_mins(1),
				}),
				Rc::new(Email {
					subject: "Status update".into(),
					author: "bob@example.com".into(),
					body: EmailBody::text_only("All systems operational."),
					sent_time: Utc::now() - std::time::Duration::from_mins(60),
				}),
				Rc::new(Email {
					subject: "Looooooooooooooooooooooooooooooooooooooooooooooooooooooooong".into(),
					author: "loooooong@example.com".into(),
					body: EmailBody::html("<h1>Loooooong</h1> <br>".repeat(20)),
					sent_time: Utc::now() - std::time::Duration::from_hours(24 * 7),
				}),
				Rc::new(Email {
					subject: "Moodle: You have received feedback!".into(),
					author: "noreply@moodle.com".into(),
					body: EmailBody::text_only("Look on the moodle website for feedback"),
					sent_time: Utc::now() - std::time::Duration::from_mins(60),
				}),
				Rc::new(Email {
					subject: "New Login".into(),
					author: "noreply@bitwarden.com".into(),
					body: EmailBody::text_only(
						"A new device has logged into your bitwarden account!",
					),
					sent_time: Utc::now() - std::time::Duration::from_mins(60),
				}),
			]
			.into_iter()
			.map(|email| (Uuid::new_v4(), email))
			.collect::<HashMap<Uuid, Rc<Email>>>(),
		}
	}
}
