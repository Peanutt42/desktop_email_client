mod email;
pub use email::{
	Email, EmailAccount, EmailBody, EmailBodySummary, EmailEnvelope, EmailFolder, EmailInfo,
	EmailProvider, EmailProviderType, EmailRow, ReceivedEmail,
};

mod api;
pub use api::{
	Api, ApiClient, DATABASE_CHANGED_EVENT_NAME, DEV_NON_IPC_API_ROUTE, DatabaseChangedEvent,
	DatabaseTable, EmailFilter, Request, Response, TauriCommandArgsWrapper,
};

mod api_macro;
