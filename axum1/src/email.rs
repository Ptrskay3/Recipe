use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};

use crate::{config::EmailClientSettings, error::ApiError};

#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn parse(s: String) -> Result<Email, ApiError> {
        if validator::ValidateEmail::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(ApiError::unprocessable_entity([("email", "invalid email")]))
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Email> for String {
    fn from(e: Email) -> Self {
        e.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: String,
    authorization_token: SecretString,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: String,
        authorization_token: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }

    pub fn from_config(config: EmailClientSettings) -> Self {
        let sender = config.sender().unwrap().into();
        let timeout = std::time::Duration::from_millis(config.timeout_milliseconds);
        Self::new(config.base_url, sender, config.authorization_token, timeout)
    }

    pub async fn send_mail(
        &self,
        recipient: Email,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };
        self.http_client
            .post(&url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}
