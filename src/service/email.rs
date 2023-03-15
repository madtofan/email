use std::sync::Arc;

use async_trait::async_trait;
use common::errors::{ServiceError, ServiceResult};
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use mockall::automock;

use crate::config::AppConfig;

#[automock]
#[async_trait]
pub trait EmailServiceTrait {
    async fn send_email(&self, address: String, title: String, body: String) -> ServiceResult<()>;
    async fn blast_email(
        &self,
        addresses: Vec<String>,
        title: String,
        body: String,
    ) -> ServiceResult<()>;
}

pub type DynEmailServiceTrait = Arc<dyn EmailServiceTrait + Send + Sync>;

pub struct EmailService {
    mailer: SmtpTransport,
    from: Mailbox,
}

impl EmailService {
    pub fn new(config: &Arc<AppConfig>) -> Self {
        let email_address = &config.service_email_address;
        let email_password = &config.service_email_password;
        let creds = Credentials::new(email_address.to_owned(), email_password.to_owned());

        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

        let from = format!(
            "{} <{}>",
            &config.service_email_name, &config.service_email_address
        )
        .parse::<Mailbox>()
        .unwrap();

        Self { mailer, from }
    }
}

#[async_trait]
impl EmailServiceTrait for EmailService {
    async fn send_email(&self, address: String, title: String, body: String) -> ServiceResult<()> {
        let recipient = address
            .parse::<Mailbox>()
            .map_err(|_| ServiceError::BadRequest("Email address is invalid".to_string()))?;

        let email = Message::builder()
            .from(self.from.clone())
            .to(recipient)
            .subject(&title)
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .unwrap();

        self.mailer
            .send(&email)
            .map_err(|err| ServiceError::InternalServerErrorWithContext(err.to_string()))?;

        match self.mailer.send(&email) {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => panic!("Could not send email: {e:?}"),
        }
        todo!()
    }

    async fn blast_email(
        &self,
        addresses: Vec<String>,
        title: String,
        body: String,
    ) -> ServiceResult<()> {
        todo!()
    }
}
