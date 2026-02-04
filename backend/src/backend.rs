use std::{
	collections::{HashMap, HashSet},
	sync::RwLock,
};

use chrono::{Local, Utc};
use desktop_email_client_shared::{
	BackendApi, Email, EmailAccount, EmailAccountSelection, EmailBody, EmailFolder,
	EmailSearchResult,
};
use uuid::Uuid;

struct EmailProvider {
	account: EmailAccount,
	folders: Vec<EmailFolder>,
	email_uuids: HashSet<Uuid>,
}
impl EmailProvider {
	fn new(account: EmailAccount, folders: Vec<EmailFolder>, email_uuids: HashSet<Uuid>) -> Self {
		Self {
			account,
			folders,
			email_uuids,
		}
	}
}

pub struct Backend {
	email_providers: Vec<EmailProvider>,
	emails: RwLock<HashMap<Uuid, Email>>,
}
impl BackendApi for Backend {
	fn get_email_account_count(&self) -> usize {
		self.email_providers.len()
	}

	fn get_email_accounts(&self) -> Vec<EmailAccount> {
		self.email_providers
			.iter()
			.map(|provider| provider.account.clone())
			.collect()
	}

	/// returns empty Vec when index invalid
	fn get_email_folders(&self, email_account_index: usize) -> Vec<EmailFolder> {
		self.email_providers
			.get(email_account_index)
			.map(|provider| provider.folders.clone())
			.unwrap_or_default()
	}

	fn get_emails_in_folder(
		&self,
		email_account_selection: EmailAccountSelection,
		email_folder_uuid: Option<Uuid>,
	) -> Vec<EmailSearchResult> {
		let mut search_results = vec![];

		for (email_account_index, provider) in self.email_providers.iter().enumerate() {
			if !email_account_selection.is_selected(email_account_index as u32) {
				continue;
			}

			for email_uuid in &provider.email_uuids {
				if let Some(email) = self.emails.read().unwrap().get(email_uuid) {
					let should_be_included = email_folder_uuid
						.as_ref()
						.map(|selected_email_folder_uuid| {
							email.folder_uuid == *selected_email_folder_uuid
						})
						.unwrap_or(true);
					if should_be_included {
						search_results.push(EmailSearchResult {
							email_uuid: *email_uuid,
							email: email.clone(),
						});
					}
				}
			}
		}

		// order by time sent descending (latest to oldest)
		search_results.sort_by(
			|EmailSearchResult { email: email_a, .. }, EmailSearchResult { email: email_b, .. }| {
				email_b.sent_time.cmp(&email_a.sent_time)
			},
		);

		search_results
	}

	fn get_emails_matching_search(
		&self,
		email_account_selection: EmailAccountSelection,
		search: String,
	) -> Vec<EmailSearchResult> {
		let mut search_results: Vec<(i64, Uuid, Email)> = vec![];

		for (email_account_index, provider) in self.email_providers.iter().enumerate() {
			if email_account_selection.is_selected(email_account_index as u32) {
				for email_uuid in &provider.email_uuids {
					if let Some(email) = self.emails.read().unwrap().get(email_uuid)
						&& let Some(score) = email.match_search_pattern(&search)
					{
						search_results.push((score, *email_uuid, email.clone()));
					}
				}
			}
		}

		// order by score descending
		search_results.sort_by(|(a_score, _, _), (b_score, _, _)| b_score.cmp(a_score));

		search_results
			.into_iter()
			.map(|(_score, email_uuid, email)| EmailSearchResult { email_uuid, email })
			.collect::<Vec<_>>()
	}

	fn get_email(&self, email_uuid: Uuid) -> Option<Email> {
		self.emails.read().unwrap().get(&email_uuid).cloned()
	}

	fn mark_email_read(&self, email_uuid: Uuid) {
		if let Some(email) = self.emails.write().unwrap().get_mut(&email_uuid) {
			email.read = true;
		}
	}
}
impl Backend {
	pub fn create_mock() -> Self {
		let mut all_emails = HashMap::new();
		let mut email_providers = Vec::new();

		for i in 0..5 {
			let (email_folders, emails) = create_mock_emails();
			email_providers.push(EmailProvider::new(
				EmailAccount::new(
					format!("Account {}", i + 1),
					format!("account{}@example.com", i + 1),
				),
				email_folders,
				emails.keys().cloned().collect::<HashSet<Uuid>>(),
			));
			all_emails.extend(emails.into_iter());
		}

		Self {
			email_providers,
			emails: RwLock::new(all_emails),
		}
	}
}

fn create_mock_emails() -> (Vec<EmailFolder>, HashMap<Uuid, Email>) {
	let inbox_folder = EmailFolder::single("Inbox");
	let friends_folder = EmailFolder::single("Friends");
	let moodle_folder = EmailFolder::single("Moodle");
	let important_folder = EmailFolder::new(
		"Important",
		vec![friends_folder.clone(), moodle_folder.clone()],
	);
	let status_system_folder = EmailFolder::single("Status system");
	let not_important_folder =
		EmailFolder::new("Not important", vec![status_system_folder.clone()]);

	let emails = [
		Email {
			subject: "Hello".into(),
			author: "alice@example.com".into(),
			body: EmailBody::text_only("Hello from Alice"),
			sent_time: Local::now().with_timezone(&Utc) - std::time::Duration::from_mins(1),
			read: false,
			folder_uuid: friends_folder.uuid,
			tags: vec![friends_folder.name.clone()],
		},
		Email {
			subject: "Status update".into(),
			author: "bob@example.com".into(),
			body: EmailBody::text_only("All systems operational."),
			sent_time: Local::now().with_timezone(&Utc)
				- std::time::Duration::from_mins(45),
				read: false,
				folder_uuid: status_system_folder.uuid,
			tags: vec![status_system_folder.name.clone(), not_important_folder.name.clone()],
		},
		Email {
			subject: "Very looooooooooong subject title, so long that i dont know what to write anymore...".into(),
			author: "loooooong@example.com".into(),
			body: EmailBody::html("<h1>Loooooong</h1> <br>".repeat(20)),
			sent_time: Local::now().with_timezone(&Utc)
				- std::time::Duration::from_hours(24 * 7),
				read: false,
				folder_uuid: inbox_folder.uuid,
			tags: vec![inbox_folder.name.clone()],
		},
		Email {
			subject: "Very looooooooooong subject title, so long that i dont know what to write anymore... (text only)".into(),
			author: "loooooong@example.com".into(),
			body: EmailBody::text_only("Loooooong\n".repeat(20)),
			sent_time: Local::now().with_timezone(&Utc)
				- std::time::Duration::from_hours(24 * 7)
				- std::time::Duration::from_mins(5),
				read: false,
				folder_uuid: inbox_folder.uuid,
			tags: vec![inbox_folder.name.clone()],
		},
		Email {
			subject: "Moodle: You have received feedback!".into(),
			author: "noreply@moodle.com".into(),
			body: EmailBody::text_only("Look on the moodle website for feedback"),
			sent_time: Local::now().with_timezone(&Utc)
				- std::time::Duration::from_mins(120),
				read: false,
				folder_uuid: moodle_folder.uuid,
			tags: vec![moodle_folder.name.clone()],
		},
		Email {
			subject: "New Login".into(),
			author: "noreply@bitwarden.com".into(),
			body: EmailBody::text_only(
				"A new device has logged into your bitwarden account!",
			),
			sent_time: Local::now().with_timezone(&Utc)
				- std::time::Duration::from_mins(10),
				read: false,
				folder_uuid: not_important_folder.uuid,
			tags: vec![not_important_folder.name.clone()],
		},
		Email {
			subject: "Something important from long ago...".to_string(),
			author: "i dont know :/".to_string(),
			body: EmailBody::text_only("i forgor :("),
			sent_time: Local::now().with_timezone(&Utc) - chrono::TimeDelta::weeks(120),
			read: false,
			folder_uuid: important_folder.uuid,
			tags: vec![important_folder.name.clone()],
		},
	]
	.into_iter()
	.map(|email| (Uuid::new_v4(), email))
	.collect::<HashMap<Uuid, Email>>();

	(
		vec![
			inbox_folder.clone(),
			important_folder.clone(),
			not_important_folder.clone(),
		],
		emails,
	)
}
