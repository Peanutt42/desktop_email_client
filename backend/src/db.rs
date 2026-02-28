use std::path::Path;

use chrono::{Duration, NaiveDateTime, Utc};
use desktop_email_client_shared::{
	Email, EmailBody, EmailBodySummary, EmailEnvelope, EmailFilter, EmailFolder, EmailInfo,
	EmailProviderType, EmailRow,
};
use futures_util::{StreamExt, stream};
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

pub struct Database {
	pub(crate) db_pool: SqlitePool,
}
impl Database {
	/// panics on failure
	pub async fn init_from_file(sqlite_db_filepath: &Path) -> Self {
		let url = &format!("sqlite:///{}", sqlite_db_filepath.display());

		Self::init_from_url(url).await
	}

	/// panics on failure
	pub async fn init_from_url(url: &str) -> Self {
		let need_to_create_db = !Sqlite::database_exists(url).await.unwrap_or(false);

		if need_to_create_db {
			Sqlite::create_database(url)
				.await
				.expect("failed to create sqlite db file");
		}

		let db_pool = SqlitePoolOptions::new()
			.max_connections(5)
			.connect(url)
			.await
			.expect("failed to connect to sqlite db");

		sqlx::migrate!("./migrations")
			.run(&db_pool)
			.await
			.expect("failed to migrate sqlite db");

		let this = Self { db_pool };

		if need_to_create_db {
			this.populate_with_mock_data().await;
		}

		this
	}

	/// `filter_folder_id`: None will include any email, regardless of the folder_id
	pub async fn get_email_infos(&self, filter_folder_id: Option<i64>) -> Vec<EmailInfo> {
		let emails = sqlx::query_as!(
			EmailInfoRow,
			r#"
			SELECT
				e.id,
				e.email_account_id,
				e.author_name,
				e.author_address,
				e.subject,
				e.body_summary,
				e.sent_time,
				e.read,
				-- Ensure we get '[]' instead of '[null]' when no tags exist
				COALESCE(
					(SELECT json_group_array(f.name)
					FROM email_tags_refs etr
					JOIN email_folders f ON etr.folder_id = f.id
					WHERE etr.email_id = e.id),
					'[]'
				) as "tags!"
			FROM emails e
			WHERE (?1 IS NULL OR e.folder_id = ?1)
			GROUP BY e.id
			ORDER BY e.sent_time DESC
			"#,
			filter_folder_id
		)
		.fetch_all(&self.db_pool)
		.await
		.expect("failed to get emails");

		emails
			.into_iter()
			.map(|email_info_row| {
				email_info_row
					.try_into()
					.expect("failed to decode Json output of sqlite request")
			})
			.collect::<Vec<EmailInfo>>()
	}

	/// returns inserted email account ID
	pub async fn insert_email_account(
		&self,
		name: String,
		email_address: String,
		provider_type: EmailProviderType,
	) -> i64 {
		let provider_type_str = provider_type.as_str();

		sqlx::query_scalar!(
			r#"
			INSERT INTO email_accounts (name, address, provider_type)
			VALUES (?, ?, ?)
			RETURNING id
			"#,
			name,
			email_address,
			provider_type_str
		)
		.fetch_one(&self.db_pool)
		.await
		.expect("failed to insert email account")
	}

	pub async fn remove_email_account(&self, email_account_id: i64) {
		sqlx::query!("DELETE FROM email_accounts WHERE id = ?", email_account_id)
			.execute(&self.db_pool)
			.await
			.expect("failed to delete email account");
	}

	/// inserts email folder and returns inserted email folder ID (updates row on conflict)
	// TODO: have '/' with nested folders supported!
	pub async fn insert_email_folder(&self, email_account_id: i64, name: String) -> i64 {
		// TODO
		let parent_id: Option<i64> = None;

		sqlx::query_scalar!(
			r#"
			INSERT INTO email_folders (parent_id, email_account_id, name)
			VALUES (?1, ?2, ?3)
			ON CONFLICT (parent_id, email_account_id, name) DO UPDATE SET parent_id = ?1, email_account_id = ?2, name = ?3
			RETURNING id
			"#,
			parent_id,
			email_account_id,
			name
		)
		.fetch_one(&self.db_pool)
		.await
		.expect("failed to insert email folder")
		.expect("SQL query should have returned an email folder ID")
	}

	pub async fn insert_email(
		&self,
		email_account_id: i64,
		envelope: EmailEnvelope,
		body: EmailBody,
		folder_id: i64,
	) -> i64 {
		let body_is_html = body.is_html();
		let body_summary = body.extract_summary().map(EmailBodySummary::take_str);
		let body_content = body.content_str();

		// TODO: Insert email tags into SQLite DB

		sqlx::query_scalar!(
			"INSERT INTO emails (email_account_id, author_name, author_address, subject, is_body_html, body, body_summary, sent_time, folder_id) VALUES
			(?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
			email_account_id,
			envelope.author_name,
			envelope.author_address,
			envelope.subject,
			body_is_html,
			body_content,
			body_summary,
			envelope.sent_time,
			folder_id
		)
		.fetch_one(&self.db_pool)
		.await
		.expect("failed to insert email")
		.expect("SQL query should have returned an email ID")
	}

	pub async fn get_email_accounts(&self) -> Vec<EmailAccountRow> {
		sqlx::query_as!(EmailAccountRow, "SELECT * FROM email_accounts")
			.fetch_all(&self.db_pool)
			.await
			.expect("failed to get email accounts")
	}

	/// returns empty Vec when index invalid
	pub async fn get_email_folders(&self, email_account_id: i64) -> Vec<EmailFolder> {
		let rows = sqlx::query_as!(
			EmailFolderRow,
			"SELECT * FROM email_folders WHERE email_account_id = ?",
			email_account_id
		)
		.fetch_all(&self.db_pool)
		.await
		.expect("failed to get email_folders");

		fn build_tree(all_rows: &[EmailFolderRow], parent_id: Option<i64>) -> Vec<EmailFolder> {
			all_rows
				.iter()
				.filter_map(|row| {
					if row.parent_id == parent_id {
						Some(EmailFolder {
							id: row.id,
							name: row.name.clone(),
							subfolders: build_tree(all_rows, Some(row.id)),
						})
					} else {
						None
					}
				})
				.collect()
		}

		build_tree(&rows, None)
	}

	pub async fn get_emails(&self, email_filter: EmailFilter) -> Vec<EmailInfo> {
		match email_filter {
			EmailFilter::Any => self.get_email_infos(None).await,
			EmailFilter::Folder { folder_id } => self.get_email_infos(Some(folder_id)).await,
			EmailFilter::MatchingSearch { search } => {
				let emails = sqlx::query_as!(
					EmailInfoRow,
					r#"
					SELECT
						e.id,
						e.email_account_id,
						e.author_name,
						e.author_address,
						e.subject,
						e.body_summary,
						e.sent_time,
						e.read,
						-- Ensure we get '[]' instead of '[null]' when no tags exist
						COALESCE(
							(SELECT json_group_array(f.name)
							FROM email_tags_refs etr
							JOIN email_folders f ON etr.folder_id = f.id
							WHERE etr.email_id = e.id),
							'[]'
						) as "tags!"
					FROM emails e
					JOIN emails_fts fts ON e.id = fts.rowid
					WHERE emails_fts MATCH ?
					ORDER BY rank
					"#,
					search
				)
				.fetch_all(&self.db_pool)
				.await
				.expect("failed to get emails");

				emails
					.into_iter()
					.map(|email_info_row| {
						email_info_row
							.try_into()
							.expect("failed to decode Json output of sqlite request")
					})
					.collect::<Vec<EmailInfo>>()
			}
		}
	}

	pub async fn get_email(&self, email_id: i64) -> Option<EmailRow> {
		match sqlx::query!(
			r#"
			SELECT
				*,
				-- Ensure we get '[]' instead of '[null]' when no tags exist
				COALESCE(
					(SELECT json_group_array(f.name)
					FROM email_tags_refs etr
					JOIN email_folders f ON etr.folder_id = f.id
					WHERE etr.email_id = id),
					'[]'
				) as "tags!"
			FROM emails
			WHERE id = ?
			"#,
			email_id
		)
		.fetch_one(&self.db_pool)
		.await
		{
			Ok(row) => Some({
				let tags = sqlx::types::Json::decode_from_string(&row.tags)
					.expect("failed to decode Json output of sqlite request")
					.0;
				EmailRow {
					id: row.id,
					email: Email {
						email_account_id: row.email_account_id,
						envelope: EmailEnvelope {
							author_name: row.author_name,
							author_address: row.author_address,
							subject: row.subject,
							sent_time: row.sent_time.and_utc(),
						},
						body: if row.is_body_html {
							EmailBody::Html(row.body)
						} else {
							EmailBody::TextOnly(row.body)
						},
						body_summary: row.body_summary,
						read: row.read,
						folder_id: row.folder_id,
						tags,
					},
				}
			}),
			Err(ref e) if matches!(e, sqlx::Error::RowNotFound) => None,
			Err(e) => panic!("failed to get email: {e}"),
		}
	}

	pub async fn mark_email_read(&self, email_id: i64) {
		sqlx::query!("UPDATE emails SET read = TRUE WHERE id = ?", email_id)
			.execute(&self.db_pool)
			.await
			.expect("failed to mark email read");
	}

	pub async fn populate_with_mock_data(&self) {
		let account_ids = stream::iter(0..3)
			.map(|i| async move {
				self.insert_email_account(
					format!("Account {}", i + 1),
					format!("account{}@example.com", i + 1),
					EmailProviderType::Mock,
				)
				.await
			})
			.collect::<Vec<_>>()
			.await;

		for account_id in account_ids {
			let account_id = account_id.await;

			let inbox_id = self
				.insert_email_folder(account_id, "Inbox".to_string())
				.await;
			let important_id = self
				.insert_email_folder(account_id, "Important".to_string())
				.await;
			let not_important_id = self
				.insert_email_folder(account_id, "Not important".to_string())
				.await;
			// TODO: Important/Friends, Important/Moodle, Not important/Status system
			let friends_id = self
				.insert_email_folder(account_id, "Friends".to_string())
				.await;
			let moode_id = self
				.insert_email_folder(account_id, "Moodle".to_string())
				.await;
			let status_system_id = self
				.insert_email_folder(account_id, "Status system".to_string())
				.await;

			/*
			* (1, 1, 'Alice', 'alice@example.com', 'Hello', 0, 'Hello from Alice', 'Hello from Alice', datetime('now', '-1 minute'), 0, 4),
			   (2, 1, 'Bob', 'bob@example.com', 'Status update', 0, 'All systems operational.', 'All systems...', datetime('now', '-45 minutes'), 0, 6),
			   (3, 1, 'Loooooong', 'loooooong@example.com', 'Very looooooooooong subject title...', 1, '<h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br>', 'Loooooong', datetime('now', '-7 days'), 0, 1),
			   (4, 1, 'Loooooong', 'loooooong@example.com', 'Very looooooooooong subject title (text only)', 0, 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || '', 'Loooooong', datetime('now', '-7 days', '-5 minutes'), 0, 1),
			   (5, 1, 'Noreply', 'noreply@moodle.com', 'Moodle: You have received feedback!', 0, 'Look on the moodle website for feedback', 'Look on moodle...', datetime('now', '-120 minutes'), 0, 5),
			   (6, 1, 'Noreply', 'noreply@bitwarden.com', 'New Login', 0, 'A new device has logged into your bitwarden account!', 'A new device...', datetime('now', '-10 minutes'), 0, 3),
			   (7, 1, 'i dont know', 'i dont know :/', 'Something important from long ago...', 0, 'i forgor :(', 'i forgor :(', datetime('now', '-840 days'), 0, 2);
			*/

			self.insert_email(
				account_id,
				EmailEnvelope {
					author_name: "Alice".to_string(),
					author_address: "alice@example.com".to_string(),
					subject: "Hello".to_string(),
					sent_time: Utc::now() - Duration::minutes(1),
				},
				EmailBody::TextOnly("Hello from Alice".to_string()),
				friends_id,
			)
			.await;

			self.insert_email(
				account_id,
				EmailEnvelope {
					author_name: "Bob".to_string(),
					author_address: "bob@example.com".to_string(),
					subject: "Status update".to_string(),
					sent_time: Utc::now() - Duration::minutes(45),
				},
				EmailBody::TextOnly("All systems operational.".to_string()),
				status_system_id,
			)
			.await;

			let long_html_body = (0..20)
				.map(|_| "<h1>Loooooong</h1><br>")
				.collect::<String>();

			self.insert_email(
				account_id,
				EmailEnvelope {
					author_name: "Loooooong".to_string(),
					author_address: "loooooong@example.com".to_string(),
					subject: "Very looooooooooong subject title...".to_string(),
					sent_time: Utc::now() - Duration::days(7),
				},
				EmailBody::Html(long_html_body),
				inbox_id,
			)
			.await;

			let long_text_body = (0..20).map(|_| "Loooooong").collect::<Vec<_>>().join("\n");

			self.insert_email(
				account_id,
				EmailEnvelope {
					author_name: "Loooooong".to_string(),
					author_address: "loooooong@example.com".to_string(),
					subject: "Very looooooooooong subject title (text only)".to_string(),
					sent_time: Utc::now() - Duration::days(7) - Duration::minutes(5),
				},
				EmailBody::TextOnly(long_text_body),
				inbox_id,
			)
			.await;

			self.insert_email(
				account_id,
				EmailEnvelope {
					author_name: "Noreply".to_string(),
					author_address: "noreply@moodle.com".to_string(),
					subject: "Moodle: You have received feedback!".to_string(),
					sent_time: Utc::now() - Duration::minutes(120),
				},
				EmailBody::TextOnly("Look on the moodle website for feedback".to_string()),
				moode_id,
			)
			.await;

			self.insert_email(
				account_id,
				EmailEnvelope {
					author_name: "Noreply".to_string(),
					author_address: "noreply@bitwarden.com".to_string(),
					subject: "New Login".to_string(),
					sent_time: Utc::now() - Duration::minutes(10),
				},
				EmailBody::TextOnly(
					"A new device has logged into your bitwarden account!".to_string(),
				),
				not_important_id,
			)
			.await;

			self.insert_email(
				account_id,
				EmailEnvelope {
					author_name: "i dont know".to_string(),
					author_address: "i dont know :/".to_string(),
					subject: "Something important from long ago...".to_string(),
					sent_time: Utc::now() - Duration::days(840),
				},
				EmailBody::TextOnly("i forgor :(".to_string()),
				important_id,
			)
			.await;
		}
	}
}

#[derive(sqlx::FromRow)]
pub struct EmailAccountRow {
	pub id: i64,
	pub name: String,
	pub address: String,
	pub provider_type: String,
}

#[derive(sqlx::FromRow)]
struct EmailFolderRow {
	id: i64,
	parent_id: Option<i64>,
	#[allow(unused)]
	email_account_id: i64,
	name: String,
}

/// sqlite row of emails with only the information needed to display the email list item or search results
/// large data like the `body` field is missing
#[derive(sqlx::FromRow)]
struct EmailInfoRow {
	id: i64,
	email_account_id: i64,
	author_name: String,
	author_address: String,
	subject: String,
	body_summary: Option<String>,
	sent_time: NaiveDateTime,
	read: bool,
	/// json encoded array of string!!!
	tags: String,
}

impl TryFrom<EmailInfoRow> for EmailInfo {
	type Error = sqlx::error::BoxDynError;

	fn try_from(value: EmailInfoRow) -> Result<Self, Self::Error> {
		let tags: Vec<String> = sqlx::types::Json::decode_from_string(&value.tags)?.0;

		Ok(EmailInfo {
			email_id: value.id,
			email_account_id: value.email_account_id,
			envelope: EmailEnvelope {
				author_name: value.author_name,
				author_address: value.author_address,
				subject: value.subject,
				sent_time: value.sent_time.and_utc(),
			},
			body_summary: value.body_summary.and_then(|str| str.try_into().ok()),
			read: value.read,
			tags,
		})
	}
}
