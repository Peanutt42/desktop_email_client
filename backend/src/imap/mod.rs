use std::sync::Arc;

use async_imap::Session;
use async_native_tls::{TlsConnector, TlsStream};
use chrono::{DateTime, Utc};
use desktop_email_client_shared::{EmailBody, EmailEnvelope, ReceivedEmail};
use futures_util::TryStreamExt;
use thiserror::Error;
use tokio::net::TcpStream;

use crate::backend::BackendInner;

#[derive(Debug, Error)]
pub enum ImapListenerError {
	#[error("io error: {0}")]
	Io(#[from] std::io::Error),
	#[error("tls error: {0}")]
	Tls(#[from] async_native_tls::Error),
	#[error("imap error: {0}")]
	Imap(#[from] async_imap::error::Error),
	#[error("invalid imap server ip address format: {0}")]
	IpAddress(#[from] std::net::AddrParseError),
	#[error("invalid date time format: {0}")]
	DateTimeParse(#[from] chrono::format::ParseError),
	// TODO: REMOVE, just dev
	#[error("empty inbox")]
	EmptyInbox,
}

pub struct ImapListener {
	account_id: i64,
	imap_session: Session<TlsStream<TcpStream>>,
	backend_inner: Arc<BackendInner>,
}
impl ImapListener {
	pub async fn connect(
		account_id: i64,
		imap_server: &str,
		username: &str,
		password: &str,
		backend_inner: Arc<BackendInner>,
	) -> Result<Self, ImapListenerError> {
		let imap_addr = (imap_server, 993);
		tracing::debug!("connecting to imap server: {}:{}", imap_addr.0, imap_addr.1);
		let tcp_stream = TcpStream::connect(imap_addr).await?;
		let tls = TlsConnector::new();
		let tls_stream = tls.connect(imap_server, tcp_stream).await?;

		let client = async_imap::Client::new(tls_stream);
		let imap_session = client
			.login(username, password)
			.await
			.map_err(|(e, _client)| e)?;

		let mut this = Self {
			account_id,
			imap_session,
			backend_inner,
		};

		this.fetch_first().await;

		Ok(this)
	}

	async fn fetch_first(&mut self) {
		match self.fetch_first_mail_in_inbox().await {
			Ok(email) => {
				self.backend_inner.receive_email(email).await;
			}
			Err(e) => {
				tracing::error!("Failed to fetch first email: {}", e);
			}
		}
	}

	async fn fetch_first_mail_in_inbox(&mut self) -> Result<ReceivedEmail, ImapListenerError> {
		self.imap_session.select("INBOX").await?;
		let messages_stream = self.imap_session.fetch("1", "(RFC822 ENVELOPE)").await?;
		let messages: Vec<_> = messages_stream.try_collect().await?;
		let Some(message) = messages.first() else {
			return Err(ImapListenerError::EmptyInbox);
		};

		let envelope = message.envelope().unwrap();
		let from = envelope.from.as_ref().unwrap().first().unwrap();
		let author_address = String::from_utf8_lossy(from.mailbox.as_ref().unwrap()).to_string();
		let author_name = String::from_utf8_lossy(from.name.as_ref().unwrap()).to_string();
		let subject = String::from_utf8_lossy(envelope.subject.as_ref().unwrap()).to_string();
		// TODO: get body type: message.bodystructure().unwrap()
		let body = String::from_utf8_lossy(message.body().unwrap()).to_string();
		let date_str = String::from_utf8_lossy(envelope.date.as_ref().unwrap()).to_string();
		let sent_time = DateTime::parse_from_rfc2822(&date_str)?.with_timezone(&Utc);

		Ok(ReceivedEmail {
			email_account_id: self.account_id,
			envelope: EmailEnvelope {
				subject,
				author_address,
				author_name,
				sent_time,
			},
			body: EmailBody::TextOnly(body),
			folder_name: "Inbox".to_string(),
		})
	}
}
