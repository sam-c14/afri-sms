//! Sendar SMS provider adapter.
//!
//! Provides integration with the Sendar REST API (`https://api.sendar.io`).

use std::fmt;

use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::error::SmsError;
use crate::provider::{SmsProvider, SmsResponse};

const DEFAULT_SENDAR_BASE_URL: &str = "https://api.sendar.io";
const SEND_SMS_PATH: &str = "/v2/sms/send";

/// SMS gateway adapter for Sendar.
///
/// Implements [`SmsProvider`] to dispatch SMS messages via Sendar's JSON REST API.
///
/// Secrets like API keys are automatically redacted in `Debug` output to prevent leaking credentials.
pub struct SendarProvider {
    api_key: String,
    sender_id: String,
    base_url: String,
    route: Option<String>,
    client: reqwest::Client,
}

impl fmt::Debug for SendarProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SendarProvider")
            .field("api_key", &"[REDACTED]")
            .field("sender_id", &self.sender_id)
            .field("base_url", &self.base_url)
            .field("route", &self.route)
            .finish()
    }
}

impl SendarProvider {
    /// Creates a new [`SendarProvider`] with default configuration.
    ///
    /// # Arguments
    /// * `api_key` - Your Sendar bearer token / API key.
    /// * `sender_id` - The approved alphanumeric sender name or brand.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::SendarProvider;
    ///
    /// let provider = SendarProvider::new("my-api-key", "FintechApp");
    /// ```
    #[must_use]
    pub fn new(api_key: impl Into<String>, sender_id: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            sender_id: sender_id.into(),
            base_url: DEFAULT_SENDAR_BASE_URL.to_string(),
            route: None,
            client: reqwest::Client::new(),
        }
    }

    /// Creates a builder to configure a [`SendarProvider`].
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::SendarBuilder;
    ///
    /// let provider = SendarBuilder::new()
    ///     .api_key("my-api-key")
    ///     .sender_id("FintechApp")
    ///     .route("transactional")
    ///     .build()
    ///     .expect("valid configuration");
    /// ```
    #[must_use]
    pub fn builder() -> SendarBuilder {
        SendarBuilder::new()
    }

    /// Returns the configured sender ID.
    #[must_use]
    pub fn sender_id(&self) -> &str {
        &self.sender_id
    }

    /// Returns the configured base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Builder for constructing a customized [`SendarProvider`].
#[derive(Debug, Default)]
pub struct SendarBuilder {
    api_key: Option<String>,
    sender_id: Option<String>,
    base_url: Option<String>,
    route: Option<String>,
    client: Option<reqwest::Client>,
}

impl SendarBuilder {
    /// Creates an empty [`SendarBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::SendarBuilder;
    ///
    /// let builder = SendarBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Sendar API key.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::SendarBuilder;
    ///
    /// let builder = SendarBuilder::new().api_key("token_123");
    /// ```
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Sets the registered sender identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::SendarBuilder;
    ///
    /// let builder = SendarBuilder::new().sender_id("FintechApp");
    /// ```
    #[must_use]
    pub fn sender_id(mut self, sender: impl Into<String>) -> Self {
        self.sender_id = Some(sender.into());
        self
    }

    /// Overrides the default base URL (useful for integration tests or private gateways).
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::SendarBuilder;
    ///
    /// let builder = SendarBuilder::new().base_url("http://127.0.0.1:8080");
    /// ```
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Sets the delivery route type (e.g., `"transactional"` or `"promotional"`).
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::SendarBuilder;
    ///
    /// let builder = SendarBuilder::new().route("transactional");
    /// ```
    #[must_use]
    pub fn route(mut self, route: impl Into<String>) -> Self {
        self.route = Some(route.into());
        self
    }

    /// Reuses a custom or shared `reqwest::Client` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::SendarBuilder;
    ///
    /// let client = reqwest::Client::builder().build().unwrap();
    /// let builder = SendarBuilder::new().client(client);
    /// ```
    #[must_use]
    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Finalizes and validates the configuration, building the [`SendarProvider`].
    ///
    /// # Errors
    /// Returns [`SmsError::ConfigurationError`] if `api_key` or `sender_id` is missing or empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::SendarBuilder;
    ///
    /// let provider = SendarBuilder::new()
    ///     .api_key("key_123")
    ///     .sender_id("BankAlert")
    ///     .build();
    /// assert!(provider.is_ok());
    /// ```
    pub fn build(self) -> Result<SendarProvider, SmsError> {
        let api_key = match self.api_key {
            Some(k) if !k.trim().is_empty() => k,
            _ => {
                return Err(SmsError::ConfigurationError(
                    "Sendar API key must not be empty".to_string(),
                ));
            }
        };

        let sender_id = match self.sender_id {
            Some(s) if !s.trim().is_empty() => s,
            _ => {
                return Err(SmsError::ConfigurationError(
                    "Sendar sender_id must not be empty".to_string(),
                ));
            }
        };

        Ok(SendarProvider {
            api_key,
            sender_id,
            base_url: self
                .base_url
                .unwrap_or_else(|| DEFAULT_SENDAR_BASE_URL.to_string()),
            route: self.route,
            client: self.client.unwrap_or_default(),
        })
    }
}

/// Sendar outgoing request payload.
#[derive(Debug, Serialize)]
struct SendarRequestPayload<'a> {
    to: &'a str,
    sender_id: &'a str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    route: Option<&'a str>,
}

/// Sendar API response schema.
#[derive(Debug, Deserialize)]
struct SendarApiResponse {
    status: Option<String>,
    message_id: Option<String>,
    message: Option<String>,
    error: Option<String>,
}

#[async_trait]
impl SmsProvider for SendarProvider {
    async fn send_sms(&self, to: &str, message: &str) -> Result<SmsResponse, SmsError> {
        let clean_to = to.trim();
        if clean_to.is_empty() {
            return Err(SmsError::InvalidPhoneNumber {
                phone_number: to.to_string(),
                reason: "Phone number cannot be empty".to_string(),
            });
        }

        let clean_message = message.trim();
        if clean_message.is_empty() {
            return Err(SmsError::InvalidRequest(
                "SMS message content cannot be empty".to_string(),
            ));
        }

        let url = format!("{}{}", self.base_url.trim_end_matches('/'), SEND_SMS_PATH);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let auth_val = format!("Bearer {}", self.api_key);
        let header_val = HeaderValue::from_str(&auth_val).map_err(|_| {
            SmsError::ConfigurationError("Invalid characters in Sendar API key".to_string())
        })?;
        headers.insert(AUTHORIZATION, header_val);

        let payload = SendarRequestPayload {
            to: clean_to,
            sender_id: &self.sender_id,
            message: clean_message,
            route: self.route.as_deref(),
        };

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        let status_code = response.status().as_u16();
        let raw_body = response.text().await?;

        if (500..600).contains(&status_code) {
            return Err(SmsError::ProviderError {
                status_code,
                message: raw_body,
            });
        }

        if status_code == 401 || status_code == 403 {
            return Err(SmsError::AuthenticationFailed(raw_body));
        }

        if status_code == 402 {
            return Err(SmsError::InsufficientBalance(raw_body));
        }

        if status_code == 429 {
            return Err(SmsError::RateLimitExceeded(raw_body));
        }

        let parsed: SendarApiResponse =
            serde_json::from_str(&raw_body).map_err(|_| SmsError::ProviderError {
                status_code,
                message: raw_body.clone(),
            })?;

        if (200..300).contains(&status_code) {
            let msg_id = parsed
                .message_id
                .unwrap_or_else(|| "unknown_sendar_id".to_string());
            let status = parsed.status.unwrap_or_else(|| "sent".to_string());

            Ok(SmsResponse::new(msg_id, status, self.name()).with_raw_response(raw_body))
        } else {
            let err_text = parsed
                .error
                .or(parsed.message)
                .unwrap_or_else(|| raw_body.clone());

            let lower = err_text.to_lowercase();
            if lower.contains("phone") || lower.contains("number") || lower.contains("recipient") {
                Err(SmsError::InvalidPhoneNumber {
                    phone_number: to.to_string(),
                    reason: err_text,
                })
            } else {
                Err(SmsError::ProviderError {
                    status_code,
                    message: err_text,
                })
            }
        }
    }

    fn name(&self) -> &'static str {
        "sendar"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sendar_redacts_api_key_in_debug() {
        let provider = SendarProvider::new("super-secret-key-12345", "BrandApp");
        let debug_str = format!("{provider:?}");
        assert!(!debug_str.contains("super-secret-key-12345"));
        assert!(debug_str.contains("[REDACTED]"));
        assert!(debug_str.contains("BrandApp"));
    }

    #[test]
    fn test_sendar_builder_valid() {
        let provider = SendarBuilder::new()
            .api_key("key-123")
            .sender_id("BankID")
            .base_url("http://localhost:8080")
            .route("transactional")
            .build()
            .expect("should succeed");

        assert_eq!(provider.sender_id(), "BankID");
        assert_eq!(provider.base_url(), "http://localhost:8080");
        assert_eq!(provider.name(), "sendar");
    }

    #[test]
    fn test_sendar_builder_missing_credentials_fails() {
        let res = SendarBuilder::new().sender_id("BankID").build();
        assert!(matches!(res, Err(SmsError::ConfigurationError(_))));

        let res2 = SendarBuilder::new().api_key("key-123").build();
        assert!(matches!(res2, Err(SmsError::ConfigurationError(_))));
    }

    #[test]
    fn test_sendar_payload_serialization() {
        let payload = SendarRequestPayload {
            to: "+2348012345678",
            sender_id: "FintechCo",
            message: "Your OTP is 1234",
            route: Some("transactional"),
        };
        let json = serde_json::to_string(&payload).expect("serialization succeeds");
        assert!(json.contains("\"to\":\"+2348012345678\""));
        assert!(json.contains("\"sender_id\":\"FintechCo\""));
        assert!(json.contains("\"route\":\"transactional\""));
    }
}
