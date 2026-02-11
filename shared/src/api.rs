use crate::{Email, EmailAccount, EmailFolder};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSearchResult {
	pub email_uuid: Uuid,
	pub email: Email,
}

pub trait ApiRequest {
	const NAME: &'static str;
	type Args: Serialize + DeserializeOwned + std::fmt::Debug;
	type Output: Serialize + DeserializeOwned;
}

macro_rules! generate_api_request {
	($name:literal, $type_name:ident, $args:ty, $output:ty) => {
		pub struct $type_name;
		impl ApiRequest for $type_name {
			const NAME: &'static str = $name;
			type Args = $args;
			type Output = $output;
		}
	};
}

generate_api_request!("get_email_account_count", GetEmailAccountCount, (), u64);
generate_api_request!(
	"get_email_accounts",
	GetEmailAccounts,
	(),
	Vec<EmailAccount>
);
generate_api_request!(
	"get_emails_in_folder",
	GetEmailsInFolder,
	Option<Uuid>,
	Vec<EmailSearchResult>
);
generate_api_request!(
	"get_emails_matching_search",
	GetEmailsMatchingSearch,
	String,
	Vec<EmailSearchResult>
);
generate_api_request!("get_email", GetEmail, Uuid, Option<Email>);
generate_api_request!("get_email_folders", GetEmailFolders, u64, Vec<EmailFolder>);
generate_api_request!("mark_email_read", MarkEmailRead, Uuid, ());

/// simply used such that all tauri commands just have this one argument of type `ArgsWrapper`
/// such that we dont have to "curry" and "uncurry" between invoking tauri ipc command and handeling the ipc command
#[derive(Debug, Serialize, Deserialize)]
pub struct TauriCommandArgsWrapper<Args> {
	pub args: Args,
}
