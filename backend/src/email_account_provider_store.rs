use desktop_email_client_shared::{EmailProvider, EmailProviderType, Password};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const KEYRING_SERVICE: &str = "desktop_email_client";

#[derive(Debug, Error)]
pub enum EmailAccountProviderStoreError {
	#[error("Keyring error: {0}")]
	Keyring(#[from] keyring::Error),
	#[error("Serialization error: {0}")]
	Serialization(#[from] serde_json::Error),
	#[error("Invalid version: expected {expected}, actual {actual}")]
	InvalidVersion { expected: String, actual: String },
}

fn get_keyring_entry(email_account_address: &str) -> keyring::Result<keyring::Entry> {
	keyring::Entry::new(KEYRING_SERVICE, email_account_address)
}

pub fn save_email_account_provider(
	email_account_address: &str,
	provider: &EmailProvider,
) -> Result<(), EmailAccountProviderStoreError> {
	match &provider {
		EmailProvider::ManualImapSmtp {
			imap_host: host,
			imap_username: username,
			imap_password: password,
		} => {
			let imap_provider_info = ImapProviderInfo {
				version: IMAP_PROVIDER_INFO_VERSION.to_string(),
				host: host.clone(),
				username: username.clone(),
				password: password.clone(),
			};
			let info_json = serde_json::to_string(&imap_provider_info)?;
			get_keyring_entry(email_account_address)?
				.set_password(&info_json)
				.map_err(|e| e.into())
		}
		EmailProvider::Mock => Ok(()),
	}
}

pub fn read_email_account_provider(
	email_account_address: &str,
	email_provider_type: EmailProviderType,
) -> Result<EmailProvider, EmailAccountProviderStoreError> {
	match email_provider_type {
		EmailProviderType::Mock => Ok(EmailProvider::Mock),
		EmailProviderType::ManualImapSmtp => {
			let info_json = get_keyring_entry(email_account_address)?.get_password()?;
			let imap_provider_info: ImapProviderInfo = serde_json::from_str(&info_json)?;
			if imap_provider_info.version != IMAP_PROVIDER_INFO_VERSION {
				return Err(EmailAccountProviderStoreError::InvalidVersion {
					expected: IMAP_PROVIDER_INFO_VERSION.to_string(),
					actual: imap_provider_info.version,
				});
			}
			Ok(EmailProvider::ManualImapSmtp {
				imap_host: imap_provider_info.host,
				imap_username: imap_provider_info.username,
				imap_password: imap_provider_info.password,
			})
		}
	}
}

pub fn delete_email_account_provider(
	email_account_address: &str,
	email_provider_type: EmailProviderType,
) -> Result<(), keyring::Error> {
	match email_provider_type {
		EmailProviderType::ManualImapSmtp => {
			get_keyring_entry(email_account_address)?.delete_credential()
		}
		EmailProviderType::Mock => Ok(()),
	}
}

const IMAP_PROVIDER_INFO_VERSION: &str = "1";

#[derive(Debug, Serialize, Deserialize)]
struct ImapProviderInfo {
	version: String,
	host: String,
	username: String,
	password: Password,
}
