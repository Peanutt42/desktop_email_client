mod email;
pub use email::{
	Email, EmailAccount, EmailBody, EmailBodySummary, EmailEnvelope, EmailFolder, EmailInfo,
	EmailProvider, EmailProviderType, EmailRow, ReceivedEmail,
};

mod api;
pub use api::{
	API_ROUTE, Api, ApiClient, DATABASE_CHANGED_EVENT_NAME, DatabaseChangedEvent, DatabaseTable,
	EmailFilter, Request, Response, TauriCommandArgsWrapper,
};

mod api_macro;
