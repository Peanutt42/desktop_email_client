use chrono::{DateTime, Local, Utc};
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

	pub fn create_mock() -> Self {
		Self::Fake {
			emails: vec![
				Rc::new(Email {
					subject: "Hello".into(),
					author: "alice@example.com".into(),
					body: EmailBody::text_only("Hello from Alice"),
					sent_time: Local::now().with_timezone(&Utc) - std::time::Duration::from_mins(1),
					tags: vec!["Friends".to_string()],
				}),
				Rc::new(Email {
					subject: "Status update".into(),
					author: "bob@example.com".into(),
					body: EmailBody::text_only("All systems operational."),
					sent_time: Local::now().with_timezone(&Utc)
						- std::time::Duration::from_mins(45),
					tags: vec!["Status system".to_string(), "Not important".to_string()],
				}),
				Rc::new(Email {
					subject: "Looooooooooooooooooooooooooooooooooooooooooooooooooooooooong".into(),
					author: "loooooong@example.com".into(),
					body: EmailBody::html("<h1>Loooooong</h1> <br>".repeat(20)),
					sent_time: Local::now().with_timezone(&Utc)
						- std::time::Duration::from_hours(24 * 7),
					tags: vec![],
				}),
				Rc::new(Email {
					subject:
						"Looooooooooooooooooooooooooooooooooooooooooooooooooooooooong but text only"
							.into(),
					author: "loooooong@example.com".into(),
					body: EmailBody::text_only("Loooooong\n".repeat(20)),
					sent_time: Local::now().with_timezone(&Utc)
						- std::time::Duration::from_hours(24 * 7)
						- std::time::Duration::from_mins(5),
					tags: vec![],
				}),
				Rc::new(Email {
					subject: "Moodle: You have received feedback!".into(),
					author: "noreply@moodle.com".into(),
					body: EmailBody::text_only("Look on the moodle website for feedback"),
					sent_time: Local::now().with_timezone(&Utc)
						- std::time::Duration::from_mins(120),
					tags: vec!["Moodle".to_string()],
				}),
				Rc::new(Email {
					subject: "New Login".into(),
					author: "noreply@bitwarden.com".into(),
					body: EmailBody::text_only(
						"A new device has logged into your bitwarden account!",
					),
					sent_time: Local::now().with_timezone(&Utc)
						- std::time::Duration::from_mins(10),
					tags: vec!["Not important".to_string()],
				}),
				Rc::new(Email {
					subject: "Something important from long ago...".to_string(),
					author: "i dont know :/".to_string(),
					body: EmailBody::text_only("i forgor :("),
					sent_time: Local::now().with_timezone(&Utc) - chrono::TimeDelta::weeks(120),
					tags: vec![],
				}),
			]
			.into_iter()
			.map(|email| (Uuid::new_v4(), email))
			.collect::<HashMap<Uuid, Rc<Email>>>(),
		}
	}
}
