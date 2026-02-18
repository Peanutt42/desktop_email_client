mod email;
pub use email::{Email, EmailAccount, EmailBody, EmailFolder};

mod api;
pub use api::{
	Api, DEV_NON_IPC_API_ROUTE, EmailFilter, EmailInfo, Request, Response, TauriCommandArgsWrapper,
};

mod api_macro;
