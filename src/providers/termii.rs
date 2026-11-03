//! Termii SMS provider adapter.
//!
//! Provides integration with the Termii messaging platform (`https://api.ng.termii.com`).

use std::fmt;

use async_trait::async_trait;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::error::SmsError;
use crate::provider::{SmsProvider, SmsResponse};

const DEFAULT_TERMII_BASE_URL: &str = "https://api.ng.termii.com";
const SEND_SMS_PATH: &str = "/api/sms/send";
const DEFAULT_CHANNEL: &str = "dnd";
const DEFAULT_MSG_TYPE: &str = "plain";

/// SMS gateway adapter for Termii.
///
/// Implements [`SmsProvider`] to dispatch SMS messages via Termii's JSON API.
///
/// The Termii API expects credentials within the JSON body payload rather than
/// an HTTP header. Secrets are redacted in `Debug` output to prevent credential leakage.
pub struct TermiiProvider {
    api_key: String,
    sender_id: String,
    channel: String,
    msg_type: String,
    base_url: String,
    client: reqwest::Client,
}

impl fmt::Debug for TermiiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TermiiProvider")
            .field("api_key", &"[REDACTED]")
            .field("sender_id", &self.sender_id)
            .field("channel", &self.channel)
            .field("msg_type", &self.msg_type)
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl TermiiProvider {
    /// Creates a new [`TermiiProvider`] with default configuration (DND channel, plain text).
    ///
    /// # Arguments
    /// * `api_key` - Your Termii account API key.
    /// * `sender_id` - The approved alphanumeric sender ID (3-11 characters).
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::TermiiProvider;
    ///
    /// let provider = TermiiProvider::new("termii-key-123", "MyAlert");
    /// ```
    #[must_use]
    pub fn new(api_key: impl Into<String>, sender_id: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            sender_id: sender_id.into(),
            channel: DEFAULT_CHANNEL.to_string(),
            msg_type: DEFAULT_MSG_TYPE.to_string(),
            base_url: DEFAULT_TERMII_BASE_URL.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Creates a builder to configure a [`TermiiProvider`].
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::TermiiBuilder;
    ///
    /// let provider = TermiiBuilder::new()
    ///     .api_key("termii-key-123")
    ///     .sender_id("MyAlert")
    ///     .channel("generic")
    ///     .build()
    ///     .expect("valid configuration");
    /// ```
    #[must_use]
    pub fn builder() -> TermiiBuilder {
        TermiiBuilder::new()
    }

    /// Returns the configured sender ID.
    #[must_use]
    pub fn sender_id(&self) -> &str {
        &self.sender_id
    }

    /// Returns the configured route channel (e.g. `"dnd"` or `"generic"`).
    #[must_use]
    pub fn channel(&self) -> &str {
        &self.channel
    }

    /// Returns the configured base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Builder for constructing a customized [`TermiiProvider`].
#[derive(Debug, Default)]
pub struct TermiiBuilder {
    api_key: Option<String>,
    sender_id: Option<String>,
    channel: Option<String>,
    msg_type: Option<String>,
    base_url: Option<String>,
    client: Option<reqwest::Client>,
}

impl TermiiBuilder {
    /// Creates an empty [`TermiiBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::TermiiBuilder;
    ///
    /// let builder = TermiiBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Termii API key.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::TermiiBuilder;
    ///
    /// let builder = TermiiBuilder::new().api_key("api_secret");
    /// ```
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Sets the registered Termii sender ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::TermiiBuilder;
    ///
    /// let builder = TermiiBuilder::new().sender_id("BrandName");
    /// ```
    #[must_use]
    pub fn sender_id(mut self, sender: impl Into<String>) -> Self {
        self.sender_id = Some(sender.into());
        self
    }

    /// Sets the delivery channel (e.g., `"dnd"` for transactional or `"generic"` for promotional).
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::TermiiBuilder;
    ///
    /// let builder = TermiiBuilder::new().channel("generic");
    /// ```
    #[must_use]
    pub fn channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }

    /// Sets the message encoding type (e.g., `"plain"` or `"unicode"`).
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::TermiiBuilder;
    ///
    /// let builder = TermiiBuilder::new().msg_type("plain");
    /// ```
    #[must_use]
    pub fn msg_type(mut self, msg_type: impl Into<String>) -> Self {
        self.msg_type = Some(msg_type.into());
        self
    }

    /// Overrides the default Termii API base URL.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::TermiiBuilder;
    ///
    /// let builder = TermiiBuilder::new().base_url("http://127.0.0.1:8080");
    /// ```
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Reuses an existing `reqwest::Client` connection pool.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::TermiiBuilder;
    ///
    /// let client = reqwest::Client::new();
    /// let builder = TermiiBuilder::new().client(client);
    /// ```
    #[must_use]
    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Validates and constructs the [`TermiiProvider`].
    ///
    /// # Errors
    /// Returns [`SmsError::ConfigurationError`] if `api_key` or `sender_id` is missing or empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::TermiiBuilder;
    ///
    /// let provider = TermiiBuilder::new()
    ///     .api_key("key-123")
    ///     .sender_id("BankAlert")
    ///     .build();
    /// assert!(provider.is_ok());
    /// ```
    pub fn build(self) -> Result<TermiiProvider, SmsError> {
        let api_key = match self.api_key {
            Some(k) if !k.trim().is_empty() => k,
            _ => {
                return Err(SmsError::ConfigurationError(
                    "Termii API key must not be empty".to_string(),
                ));
            }
        };

        let sender_id = match self.sender_id {
            Some(s) if !s.trim().is_empty() => s,
            _ => {
                return Err(SmsError::ConfigurationError(
                    "Termii sender_id must not be empty".to_string(),
                ));
            }
        };

        Ok(TermiiProvider {
            api_key,
            sender_id,
            channel: self.channel.unwrap_or_else(|| DEFAULT_CHANNEL.to_string()),
            msg_type: self
                .msg_type
                .unwrap_or_else(|| DEFAULT_MSG_TYPE.to_string()),
            base_url: self
                .base_url
                .unwrap_or_else(|| DEFAULT_TERMII_BASE_URL.to_string()),
            client: self.client.unwrap_or_default(),
        })
    }
}

/// Termii request payload.
#[derive(Debug, Serialize)]
struct TermiiRequestPayload<'a> {
    api_key: &'a str,
    to: &'a str,
    from: &'a str,
    sms: &'a str,
    #[serde(rename = "type")]
    msg_type: &'a str,
    channel: &'a str,
}

/// Termii response schema.
#[derive(Debug, Deserialize)]
struct TermiiApiResponse {
    code: Option<String>,
    message_id: Option<serde_json::Value>,
    message_id_str: Option<String>,
    message: Option<String>,
    error: Option<String>,
}

#[async_trait]
impl SmsProvider for TermiiProvider {
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

        // Termii requires international format without leading '+'
        let normalized_to = clean_to.strip_prefix('+').unwrap_or(clean_to);

        let url = format!("{}{}", self.base_url.trim_end_matches('/'), SEND_SMS_PATH);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = TermiiRequestPayload {
            api_key: &self.api_key,
            to: normalized_to,
            from: &self.sender_id,
            sms: clean_message,
            msg_type: &self.msg_type,
            channel: &self.channel,
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

        let parsed: TermiiApiResponse =
            serde_json::from_str(&raw_body).map_err(|_| SmsError::ProviderError {
                status_code,
                message: raw_body.clone(),
            })?;

        // Check if Termii returned an error inside JSON even on 200/400
        let msg_text = parsed.message.as_deref().unwrap_or_default().to_lowercase();
        if msg_text.contains("insufficient balance") || msg_text.contains("zero balance") {
            return Err(SmsError::InsufficientBalance(
                parsed.message.unwrap_or(raw_body),
            ));
        }

        if (200..300).contains(&status_code) {
            let message_id = if let Some(str_id) = parsed.message_id_str {
                str_id
            } else if let Some(val) = parsed.message_id {
                match val {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => "unknown_termii_id".to_string(),
                }
            } else {
                "unknown_termii_id".to_string()
            };

            let status = parsed.code.unwrap_or_else(|| "sent".to_string());
            Ok(SmsResponse::new(message_id, status, self.name()).with_raw_response(raw_body))
        } else {
            let err_text = parsed
                .error
                .or(parsed.message)
                .unwrap_or_else(|| raw_body.clone());

            let lower = err_text.to_lowercase();
            if lower.contains("phone") || lower.contains("number") || lower.contains("destination")
            {
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
        "termii"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_termii_redacts_api_key_in_debug() {
        let provider = TermiiProvider::new("super-secret-termii-key", "TermiiAlert");
        let debug_str = format!("{provider:?}");
        assert!(!debug_str.contains("super-secret-termii-key"));
        assert!(debug_str.contains("[REDACTED]"));
        assert!(debug_str.contains("TermiiAlert"));
    }

    #[test]
    fn test_termii_builder_defaults() {
        let provider = TermiiBuilder::new()
            .api_key("key-xyz")
            .sender_id("BankApp")
            .build()
            .expect("valid builder");

        assert_eq!(provider.channel(), "dnd");
        assert_eq!(provider.sender_id(), "BankApp");
        assert_eq!(provider.name(), "termii");
    }

    #[test]
    fn test_termii_payload_serialization() {
        let payload = TermiiRequestPayload {
            api_key: "test_key",
            to: "2348012345678",
            from: "BrandID",
            sms: "Welcome!",
            msg_type: "plain",
            channel: "dnd",
        };
        let json = serde_json::to_string(&payload).expect("payload serializes");
        assert!(json.contains("\"api_key\":\"test_key\""));
        assert!(json.contains("\"to\":\"2348012345678\""));
        assert!(json.contains("\"from\":\"BrandID\""));
        assert!(json.contains("\"type\":\"plain\""));
        assert!(json.contains("\"channel\":\"dnd\""));
    }
}
