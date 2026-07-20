use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::{EmailSettings, Settings};

#[derive(Clone)]
pub struct EmailService {
    client: reqwest::Client,
    settings: EmailSettings,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmailDeliveryStatus {
    Sent,
    Skipped,
    RateLimited,
    Failed,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EmailDelivery {
    pub status: EmailDeliveryStatus,
    pub provider_message_id: Option<String>,
}

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("email proxy rate limited the request")]
    RateLimited,
    #[error("email proxy returned status {status}: {body}")]
    ProxyStatus { status: reqwest::StatusCode, body: String },
    #[error("failed to call email proxy")]
    Request(#[from] reqwest::Error),
}

#[derive(Debug, Serialize)]
struct EmailRequest<'a> {
    to: &'a str,
    subject: &'a str,
    html: &'a str,
    text: &'a str,
}

#[derive(Debug, Deserialize)]
struct EmailResponse {
    id: String,
}

impl EmailService {
    pub fn from_settings(settings: &Settings) -> Option<Self> {
        settings.email.clone().map(|settings| Self {
            client: reqwest::Client::new(),
            settings,
        })
    }

    pub async fn send(
        &self,
        to: &str,
        subject: &str,
        html: &str,
        text: &str,
    ) -> Result<String, EmailError> {
        let response = self
            .client
            .post(self.settings.email_url.as_str())
            .bearer_auth(self.settings.app_token.as_str())
            .json(&EmailRequest {
                to,
                subject,
                html,
                text,
            })
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(EmailError::RateLimited);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = match response.text().await {
                Ok(body) => body,
                Err(error) => format!("failed to read proxy error body: {error}"),
            };
            return Err(EmailError::ProxyStatus { status, body });
        }

        let body = response.json::<EmailResponse>().await?;
        Ok(body.id)
    }
}

impl EmailDelivery {
    pub fn skipped() -> Self {
        Self {
            status: EmailDeliveryStatus::Skipped,
            provider_message_id: None,
        }
    }

    pub fn sent(provider_message_id: String) -> Self {
        Self {
            status: EmailDeliveryStatus::Sent,
            provider_message_id: Some(provider_message_id),
        }
    }

    pub fn rate_limited() -> Self {
        Self {
            status: EmailDeliveryStatus::RateLimited,
            provider_message_id: None,
        }
    }

    pub fn failed() -> Self {
        Self {
            status: EmailDeliveryStatus::Failed,
            provider_message_id: None,
        }
    }
}
