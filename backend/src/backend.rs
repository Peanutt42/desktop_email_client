use std::{
	collections::HashMap,
	str::FromStr,
	sync::{Arc, RwLock},
};

use async_trait::async_trait;
use desktop_email_client_shared::{
	Api, DatabaseChangedEvent, DatabaseTable, EmailAccount, EmailFilter, EmailFolder, EmailInfo,
	EmailProvider, EmailProviderType, EmailRow, ReceivedEmail,
};

use crate::{
	Database, email_account_provider_store::delete_email_account_provider, imap::ImapListener,
	read_email_account_provider, save_email_account_provider,
};

pub struct BackendInner {
	database: Database,
	on_database_update_callback: Box<dyn Fn(DatabaseChangedEvent) + Send + Sync>,
	/// imap email account id to imap listener
	imap_listeners: Arc<RwLock<HashMap<i64, ImapListener>>>,
}
impl BackendInner {
	pub fn on_database_table_changed(&self, table: DatabaseTable) {
		(self.on_database_update_callback)(DatabaseChangedEvent {
			changed_table: table,
		});
	}

	pub async fn receive_email(&self, email: ReceivedEmail) {
		let folder_id = self
			.database
			.insert_email_folder(email.email_account_id, email.folder_name)
			.await;

		let _email_id = self
			.database
			.insert_email(
				email.email_account_id,
				email.envelope,
				email.body,
				folder_id,
			)
			.await;

		self.on_database_table_changed(DatabaseTable::Emails);
	}
}

pub struct Backend {
	inner: Arc<BackendInner>,
}
impl Backend {
	pub async fn new(
		database: Database,
		on_database_update_callback: Box<dyn Fn(DatabaseChangedEvent) + Send + Sync>,
	) -> Self {
		let inner = Arc::new(BackendInner {
			database,
			on_database_update_callback,
			imap_listeners: Arc::new(RwLock::new(HashMap::new())),
		});

		/*let email_accounts = inner.database.get_email_accounts().await;

		for email_account in email_accounts {
			if let EmailProvider::Imap {
				host,
				username,
				password,
			} = email_account.provider
			{
				match ImapListener::connect(
					email_account.id,
					&host,
					&username,
					&password,
					inner.clone(),
				)
				.await
				{
					Ok(imap_listener) => {
						inner
							.imap_listeners
							.write()
							.unwrap()
							.insert(email_account.id, imap_listener);
					}
					Err(e) => tracing::error!(
						"failed to connect to imap server {} with username {} and email: {} : {}",
						host,
						username,
						email_account.address,
						e
					),
				}
			}
		}*/

		Self { inner }
	}
}
#[async_trait]
impl Api for Backend {
	async fn insert_email_account(
		&self,
		name: String,
		address: String,
		provider: EmailProvider,
	) -> i64 {
		if let Err(e) = save_email_account_provider(&address, &provider) {
			tracing::error!(
				"failed to save email account provider for email account address {}: {}",
				address,
				e
			);
		}

		let email_account_id = self
			.inner
			.database
			.insert_email_account(name, address, provider.get_type())
			.await;

		self.inner
			.on_database_table_changed(DatabaseTable::EmailAccounts);

		email_account_id
	}

	async fn remove_email_account(&self, email_account_id: i64) {
		let Ok(address_and_provider_type) = sqlx::query!(
			"SELECT address, provider_type FROM email_accounts WHERE id = ?",
			email_account_id
		)
		.fetch_one(&self.inner.database.db_pool)
		.await
		else {
			tracing::error!(
				"failed to remove email account, invalid id {}",
				email_account_id
			);
			return;
		};

		let email_account_address = &address_and_provider_type.address;
		let Ok(email_provider_type) =
			EmailProviderType::from_str(&address_and_provider_type.provider_type)
		else {
			tracing::error!(
				"invalid provider_type {}, cannot remove email account with id {}",
				address_and_provider_type.provider_type,
				email_account_id
			);
			return;
		};

		self.inner
			.database
			.remove_email_account(email_account_id)
			.await;

		if let Err(e) = delete_email_account_provider(email_account_address, email_provider_type) {
			tracing::error!(
				"failed to delete email account provider for email account address {}: {}",
				email_account_address,
				e
			);
		}

		self.inner
			.on_database_table_changed(DatabaseTable::EmailAccounts);
	}

	async fn get_email_accounts(&self) -> Vec<EmailAccount> {
		let rows = self.inner.database.get_email_accounts().await;
		rows.into_iter()
			.filter_map(
				|row| match EmailProviderType::from_str(&row.provider_type) {
					Ok(provider_type) => {
						match read_email_account_provider(&row.address, provider_type) {
							Ok(email_provider) => Some(EmailAccount {
								id: row.id,
								name: row.name,
								address: row.address,
								provider: email_provider,
							}),
							Err(e) => {
								tracing::error!(
									"failed to read email account provider for email account id {}: {}",
									row.id,
									e
								);
								None
							}
						}
					}
					Err(()) => {
						tracing::error!("invalid provider type for email account id {}", row.id);
						None
					}
				},
			)
			.collect()
	}
	/// returns empty Vec when index invalid
	async fn get_email_folders(&self, email_account_id: i64) -> Vec<EmailFolder> {
		self.inner
			.database
			.get_email_folders(email_account_id)
			.await
	}
	async fn get_emails(&self, email_filter: EmailFilter) -> Vec<EmailInfo> {
		self.inner.database.get_emails(email_filter).await
	}
	async fn get_email(&self, email_id: i64) -> Option<EmailRow> {
		self.inner.database.get_email(email_id).await
	}
	async fn mark_email_read(&self, email_id: i64) {
		self.inner.database.mark_email_read(email_id).await;
		self.inner.on_database_table_changed(DatabaseTable::Emails);
	}
}
