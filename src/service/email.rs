use std::sync::Arc;

use async_trait::async_trait;
use lettre::message::Mailbox;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use madtofan_microservice_common::errors::{ServiceError, ServiceResult};
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
    creds: Credentials,
    from: Mailbox,
}

impl EmailService {
    pub fn new(config: &Arc<AppConfig>) -> Self {
        let email_address = &config.service_email_address;
        let email_password = &config.service_email_password;
        let creds = Credentials::new(email_address.to_owned(), email_password.to_owned());

        let from = format!(
            "{} <{}>",
            &config.service_email_name, &config.service_email_address
        )
        .parse::<Mailbox>()
        .unwrap();

        Self { creds, from }
    }

    #[cfg(not(test))]
    async fn send_message_email(&self, email: Message) -> ServiceResult<()> {
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("smtp.gmail.com")
                .unwrap()
                .credentials(self.creds.clone())
                .build();

        match mailer.send(email).await {
            Ok(_) => Ok(()),
            Err(_e) => Err(ServiceError::InternalServerErrorWithContext(
                "Sending email failed".to_string(),
            )),
        }
    }

    #[cfg(test)]
    async fn send_message_email(&self, _email: Message) -> ServiceResult<()> {
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("smtp.gmail.com")
                .unwrap()
                .credentials(self.creds.clone())
                .build();

        mailer.test_connection().await.map_err(|_| {
            ServiceError::InternalServerErrorWithContext(
                "Can't communicate with SMTP server".to_string(),
            )
        })?;
        Ok(())
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

        self.send_message_email(email).await
    }

    async fn blast_email(
        &self,
        _addresses: Vec<String>,
        _title: String,
        _body: String,
    ) -> ServiceResult<()> {
        todo!()
    }
}
