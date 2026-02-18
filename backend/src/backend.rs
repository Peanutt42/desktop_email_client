use std::path::Path;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use desktop_email_client_shared::{
	Api, Email, EmailAccount, EmailBody, EmailFilter, EmailFolder, EmailInfo,
};
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

pub struct Backend {
	db_pool: SqlitePool,
}
impl Backend {
	pub fn new(db_pool: SqlitePool) -> Self {
		Self { db_pool }
	}

	/// `filter_folder_id`: None will include any email, regardless of the folder_id
	async fn get_email_infos(&self, filter_folder_id: Option<i64>) -> Vec<EmailInfo> {
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
				e.folder_id,
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

	pub async fn populate_with_mock_data(&self) {
		populate_db_with_mock_data(&self.db_pool).await
	}
}
#[async_trait]
impl Api for Backend {
	async fn get_email_accounts(&self) -> Vec<EmailAccount> {
		sqlx::query_as!(EmailAccount, "SELECT * FROM email_accounts")
			.fetch_all(&self.db_pool)
			.await
			.expect("failed to get email accounts")
	}

	/// returns empty Vec when index invalid
	async fn get_email_folders(&self, email_account_id: i64) -> Vec<EmailFolder> {
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

	async fn get_emails(&self, email_filter: EmailFilter) -> Vec<EmailInfo> {
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
						e.folder_id,
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

	async fn get_email(&self, email_id: i64) -> Option<Email> {
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
				Email {
					id: row.id,
					email_account_id: row.email_account_id,
					author_name: row.author_name,
					author_address: row.author_address,
					subject: row.subject,
					body: if row.is_body_html {
						EmailBody::Html(row.body)
					} else {
						EmailBody::TextOnly(row.body)
					},
					// TODO: check if timezones conversions work correctly
					sent_time: row.sent_time.and_utc(),
					read: row.read,
					folder_id: row.folder_id,
					tags,
				}
			}),
			Err(ref e) if matches!(e, sqlx::Error::RowNotFound) => None,
			Err(e) => panic!("failed to get email: {e}"),
		}
	}

	async fn mark_email_read(&self, email_id: i64) {
		sqlx::query!("UPDATE emails SET read = TRUE WHERE id = ?", email_id)
			.execute(&self.db_pool)
			.await
			.expect("failed to mark email read");
	}
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
	folder_id: i64,
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
			author_name: value.author_name,
			author_address: value.author_address,
			subject: value.subject,
			body_summary: value.body_summary,
			// TODO: check if timezones conversions work correctly
			sent_time: value.sent_time.and_utc(),
			read: value.read,
			folder_id: value.folder_id,
			tags,
		})
	}
}

/// panics on failure
pub async fn init_db(sqlite_db_filepath: &Path) -> SqlitePool {
	let url = &format!("sqlite:///{}", sqlite_db_filepath.display());

	init_db_from_url(url).await
}

/// panics on failure
pub async fn init_db_from_url(url: &str) -> SqlitePool {
	let need_to_create_db = !Sqlite::database_exists(url).await.unwrap_or(false);

	if need_to_create_db {
		Sqlite::create_database(url)
			.await
			.expect("failed to create sqlite db file");
	}

	let pool = SqlitePoolOptions::new()
		.max_connections(5)
		.connect(url)
		.await
		.expect("failed to connect to sqlite db");

	sqlx::migrate!("./migrations")
		.run(&pool)
		.await
		.expect("failed to migrate sqlite db");

	if need_to_create_db {
		populate_db_with_mock_data(&pool).await;
	}

	pool
}

async fn populate_db_with_mock_data(pool: &SqlitePool) {
	sqlx::query!(
		r#"
PRAGMA foreign_keys = ON;

--------------------------------------------------------------------------------
-- 1. ACCOUNTS
--------------------------------------------------------------------------------
INSERT INTO email_accounts (id, name, address) VALUES
	(1, 'Account 1', 'account1@example.com'),
	(2, 'Account 2', 'account2@example.com'),
	(3, 'Account 3', 'account3@example.com');

--------------------------------------------------------------------------------
-- 2. FOLDERS (Hierarchical)
--------------------------------------------------------------------------------
-- Folders for Account 1 (IDs 1-6)
INSERT INTO email_folders (id, parent_id, email_account_id, name) VALUES
	(1, NULL, 1, 'Inbox'),
	(2, NULL, 1, 'Important'),
	(3, NULL, 1, 'Not important'),
	(4, 2,    1, 'Friends'),
	(5, 2,    1, 'Moodle'),
	(6, 3,    1, 'Status system');

-- Folders for Account 2 (IDs 7-12)
INSERT INTO email_folders (id, parent_id, email_account_id, name) VALUES
	(7, NULL, 2, 'Inbox'),
	(8, NULL, 2, 'Important'),
	(9, NULL, 2, 'Not important'),
	(10, 8,   2, 'Friends'),
	(11, 8,   2, 'Moodle'),
	(12, 9,   2, 'Status system');

-- Folders for Account 3 (IDs 13-18)
INSERT INTO email_folders (id, parent_id, email_account_id, name) VALUES
	(13, NULL, 3, 'Inbox'),
	(14, NULL, 3, 'Important'),
	(15, NULL, 3, 'Not important'),
	(16, 14,  3, 'Friends'),
	(17, 14,  3, 'Moodle'),
	(18, 15,  3, 'Status system');

--------------------------------------------------------------------------------
-- 3. EMAILS
--------------------------------------------------------------------------------

-- Account 1 Emails
INSERT INTO emails (id, email_account_id, author_name, author_address, subject, is_body_html, body, body_summary, sent_time, read, folder_id) VALUES
	(1, 1, 'Alice', 'alice@example.com', 'Hello', 0, 'Hello from Alice', 'Hello from Alice', datetime('now', '-1 minute'), 0, 4),
	(2, 1, 'Bob', 'bob@example.com', 'Status update', 0, 'All systems operational.', 'All systems...', datetime('now', '-45 minutes'), 0, 6),
	(3, 1, 'Loooooong', 'loooooong@example.com', 'Very looooooooooong subject title...', 1, '<h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br>', 'Loooooong', datetime('now', '-7 days'), 0, 1),
	(4, 1, 'Loooooong', 'loooooong@example.com', 'Very looooooooooong subject title (text only)', 0, 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || '', 'Loooooong', datetime('now', '-7 days', '-5 minutes'), 0, 1),
	(5, 1, 'Noreply', 'noreply@moodle.com', 'Moodle: You have received feedback!', 0, 'Look on the moodle website for feedback', 'Look on moodle...', datetime('now', '-120 minutes'), 0, 5),
	(6, 1, 'Noreply', 'noreply@bitwarden.com', 'New Login', 0, 'A new device has logged into your bitwarden account!', 'A new device...', datetime('now', '-10 minutes'), 0, 3),
	(7, 1, 'i dont know', 'i dont know :/', 'Something important from long ago...', 0, 'i forgor :(', 'i forgor :(', datetime('now', '-840 days'), 0, 2);

-- Account 2 Emails
INSERT INTO emails (id, email_account_id, author_name, author_address, subject, is_body_html, body, body_summary, sent_time, read, folder_id) VALUES
	(8, 2, 'Alice', 'alice@example.com', 'Hello', 0, 'Hello from Alice', 'Hello from Alice', datetime('now', '-1 minute'), 0, 10),
	(9, 2, 'Bob', 'bob@example.com', 'Status update', 0, 'All systems operational.', 'All systems...', datetime('now', '-45 minutes'), 0, 12),
	(10, 2, 'Loooooong', 'loooooong@example.com', 'Very looooooooooong subject title...', 1, '<h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br>', 'Loooooong', datetime('now', '-7 days'), 0, 7),
	(11, 2, 'Loooooong', 'loooooong@example.com', 'Very looooooooooong subject title (text only)', 0, 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || '', 'Loooooong', datetime('now', '-7 days', '-5 minutes'), 0, 7),
	(12, 2, 'Noreply', 'noreply@moodle.com', 'Moodle: You have received feedback!', 0, 'Look on the moodle website for feedback', 'Look on moodle...', datetime('now', '-120 minutes'), 0, 11),
	(13, 2, 'Noreply', 'noreply@bitwarden.com', 'New Login', 0, 'A new device has logged into your bitwarden account!', 'A new device...', datetime('now', '-10 minutes'), 0, 9),
	(14, 2, 'i dont know', 'i dont know :/', 'Something important from long ago...', 0, 'i forgor :(', 'i forgor :(', datetime('now', '-840 days'), 0, 8);

-- Account 3 Emails
INSERT INTO emails (id, email_account_id, author_name, author_address, subject, is_body_html, body, body_summary, sent_time, read, folder_id) VALUES
	(15, 3, 'Alice', 'alice@example.com', 'Hello', 0, 'Hello from Alice', 'Hello from Alice', datetime('now', '-1 minute'), 0, 16),
	(16, 3, 'Bob', 'bob@example.com', 'Status update', 0, 'All systems operational.', 'All systems...', datetime('now', '-45 minutes'), 0, 18),
	(17, 3, 'Loooooong', 'loooooong@example.com', 'Very looooooooooong subject title...', 1, '<h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br><h1>Loooooong</h1> <br>', 'Loooooong', datetime('now', '-7 days'), 0, 13),
	(18, 3, 'Loooooong', 'loooooong@example.com', 'Very looooooooooong subject title (text only)', 0, 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || 'Loooooong' || char(10) || '', 'Loooooong', datetime('now', '-7 days', '-5 minutes'), 0, 13),
	(19, 3, 'Noreply', 'noreply@moodle.com', 'Moodle: You have received feedback!', 0, 'Look on the moodle website for feedback', 'Look on moodle...', datetime('now', '-120 minutes'), 0, 17),
	(20, 3, 'Noreply', 'noreply@bitwarden.com', 'New Login', 0, 'A new device has logged into your bitwarden account!', 'A new device...', datetime('now', '-10 minutes'), 0, 15),
	(21, 3, 'i dont know', 'i dont know :/', 'Something important from long ago...', 0, 'i forgor :(', 'i forgor :(', datetime('now', '-840 days'), 0, 14);

--------------------------------------------------------------------------------
-- 4. TAG REFERENCES
--------------------------------------------------------------------------------
-- Account 1 Tags
INSERT INTO email_tags_refs (email_id, folder_id) VALUES (1, 4), (2, 6), (2, 3), (3, 1), (4, 1), (5, 5), (6, 3), (7, 2);

-- Account 2 Tags
INSERT INTO email_tags_refs (email_id, folder_id) VALUES (8, 10), (9, 12), (9, 9), (10, 7), (11, 7), (12, 11), (13, 9), (14, 8);

-- Account 3 Tags
INSERT INTO email_tags_refs (email_id, folder_id) VALUES (15, 16), (16, 18), (16, 15), (17, 13), (18, 13), (19, 17), (20, 15), (21, 14);

"#
	)
	.execute(pool)
	.await
	.expect("failed to populate db with initial mock data");
}
