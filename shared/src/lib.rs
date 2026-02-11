mod email;
pub use email::{Email, EmailAccount, EmailBody, EmailFolder, get_email_folder_by_uuid};

mod api;
pub use api::{
	ApiRequest, EmailSearchResult, GetEmail, GetEmailAccountCount, GetEmailAccounts,
	GetEmailFolders, GetEmailsInFolder, GetEmailsMatchingSearch, MarkEmailRead,
	TauriCommandArgsWrapper,
};
