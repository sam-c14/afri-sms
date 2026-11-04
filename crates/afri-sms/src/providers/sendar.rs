//! Sendar SMS provider adapter.
//!
//! Provides integration with the Sendar REST API (`https://sms-submit.sendar.io/api/`).

use std::fmt;

use async_trait::async_trait;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::error::SmsError;
use crate::provider::{SmsProvider, SmsResponse};

const DEFAULT_SENDAR_BASE_URL: &str = "https://sms-submit.sendar.io/api";
const SEND_SMS_PATH: &str = "/sms/send";
const DEFAULT_WALLET_TYPE: &str = "transactional";
const DEFAULT_SMS_TYPE: &str = "plain";

/// Delivery report and status information for a Sendar SMS message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendarMessageStatus {
    /// Unique identifier for the specific SMS.
    pub message_id: String,

    /// Real-time delivery status (e.g., "delivered", "queued", "failed").
    pub status: String,

    /// Recipient phone number in international format.
    #[serde(default)]
    pub to: Option<String>,

    /// The registered Sender Name used to send the message.
    #[serde(default)]
    pub sender_id: Option<String>,

    /// Cost per SMS unit charged for this message.
    #[serde(default)]
    pub cost_per_unit: Option<f64>,

    /// Billing transaction identifier.
    #[serde(default)]
    pub billing_event_id: Option<String>,

    /// Telecom gateway specific message identifier.
    #[serde(default)]
    pub gateway_message_id: Option<String>,

    /// ISO 8601 creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,

    /// ISO 8601 last update timestamp.
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// SMS gateway adapter for Sendar.
///
/// Implements [`SmsProvider`] to dispatch SMS messages via Sendar's JSON REST API
/// as specified in the Sendar Messaging API Technical Specification.
///
/// Secrets like API keys are automatically redacted in `Debug` output to prevent leaking credentials.
pub struct SendarProvider {
    api_key: String,
    sender_id: String,
    base_url: String,
    wallet_type: String,
    sms_type: String,
    client: reqwest::Client,
}

impl fmt::Debug for SendarProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SendarProvider")
            .field("api_key", &"[REDACTED]")
            .field("sender_id", &self.sender_id)
            .field("base_url", &self.base_url)
            .field("wallet_type", &self.wallet_type)
            .field("sms_type", &self.sms_type)
            .finish()
    }
}

impl SendarProvider {
    /// Creates a new [`SendarProvider`] with default configuration (transactional wallet, plain SMS).
    ///
    /// # Arguments
    /// * `api_key` - Your Sendar API key (`API-KEY` header).
    /// * `sender_id` - The approved alphanumeric sender name or brand.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::SendarProvider;
    ///
    /// let provider = SendarProvider::new("sk_live_sample", "FintechApp");
    /// ```
    #[must_use]
    pub fn new(api_key: impl Into<String>, sender_id: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            sender_id: sender_id.into(),
            base_url: DEFAULT_SENDAR_BASE_URL.to_string(),
            wallet_type: DEFAULT_WALLET_TYPE.to_string(),
            sms_type: DEFAULT_SMS_TYPE.to_string(),
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
    ///     .api_key("sk_live_sample")
    ///     .sender_id("FintechApp")
    ///     .wallet_type("transactional")
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

    /// Returns the configured billing account wallet type (e.g. `"transactional"`, `"promotional"`, or `"otp"`).
    #[must_use]
    pub fn wallet_type(&self) -> &str {
        &self.wallet_type
    }

    /// Returns the configured billing route/wallet type (alias for [`wallet_type`](Self::wallet_type)).
    #[must_use]
    pub fn route(&self) -> &str {
        &self.wallet_type
    }

    /// Returns the configured SMS type (e.g. `"plain"` or `"unicode"`).
    #[must_use]
    pub fn sms_type(&self) -> &str {
        &self.sms_type
    }

    /// Retrieves the real-time delivery status of a specific message using its `message_id`.
    ///
    /// Hits the Sendar `GET /get/sms/{uid}` endpoint.
    ///
    /// # Arguments
    /// * `message_id` - The unique identifier returned during the send request.
    ///
    /// # Errors
    /// Returns [`SmsError`] if the request fails, authentication is rejected, or message is not found.
    pub async fn get_message_status(
        &self,
        message_id: &str,
    ) -> Result<SendarMessageStatus, SmsError> {
        let clean_id = message_id.trim();
        if clean_id.is_empty() {
            return Err(SmsError::InvalidRequest(
                "Message ID cannot be empty".to_string(),
            ));
        }

        let url = format!(
            "{}/get/sms/{}",
            self.base_url.trim_end_matches('/'),
            clean_id
        );

        let mut headers = HeaderMap::new();
        let api_key_name = HeaderName::from_static("api-key");
        let header_val = HeaderValue::from_str(&self.api_key).map_err(|_| {
            SmsError::ConfigurationError("Invalid characters in Sendar API key".to_string())
        })?;
        headers.insert(api_key_name, header_val);

        let response = self.client.get(&url).headers(headers).send().await?;
        let status_code = response.status().as_u16();
        let raw_body = response.text().await?;

        if status_code == 401 || status_code == 403 {
            return Err(SmsError::AuthenticationFailed(raw_body));
        }

        if (500..600).contains(&status_code) {
            return Err(SmsError::ProviderError {
                status_code,
                message: raw_body,
            });
        }

        if (200..300).contains(&status_code) {
            let status: SendarMessageStatus =
                serde_json::from_str(&raw_body).map_err(|_| SmsError::ProviderError {
                    status_code,
                    message: raw_body.clone(),
                })?;
            Ok(status)
        } else {
            Err(SmsError::ProviderError {
                status_code,
                message: raw_body,
            })
        }
    }
}

/// Builder for constructing a customized [`SendarProvider`].
#[derive(Debug, Default)]
pub struct SendarBuilder {
    api_key: Option<String>,
    sender_id: Option<String>,
    base_url: Option<String>,
    wallet_type: Option<String>,
    sms_type: Option<String>,
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

    /// Sets the Sendar API key (`API-KEY` header).
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::SendarBuilder;
    ///
    /// let builder = SendarBuilder::new().api_key("sk_live_123");
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

    /// Sets the billing account wallet type (`"transactional"`, `"promotional"`, or `"otp"`).
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::SendarBuilder;
    ///
    /// let builder = SendarBuilder::new().wallet_type("otp");
    /// ```
    #[must_use]
    pub fn wallet_type(mut self, wallet_type: impl Into<String>) -> Self {
        self.wallet_type = Some(wallet_type.into());
        self
    }

    /// Sets the delivery route / wallet type (alias for [`wallet_type`](Self::wallet_type)).
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
        self.wallet_type = Some(route.into());
        self
    }

    /// Sets the SMS type (`"plain"` or `"unicode"`). Defaults to `"plain"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::SendarBuilder;
    ///
    /// let builder = SendarBuilder::new().sms_type("plain");
    /// ```
    #[must_use]
    pub fn sms_type(mut self, sms_type: impl Into<String>) -> Self {
        self.sms_type = Some(sms_type.into());
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
    ///     .api_key("sk_live_123")
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
            wallet_type: self
                .wallet_type
                .unwrap_or_else(|| DEFAULT_WALLET_TYPE.to_string()),
            sms_type: self
                .sms_type
                .unwrap_or_else(|| DEFAULT_SMS_TYPE.to_string()),
            client: self.client.unwrap_or_default(),
        })
    }
}

/// Sendar contact object within request payload.
#[derive(Debug, Serialize)]
struct SendarContactPayload<'a> {
    number: &'a str,
    body: &'a str,
    sms_type: &'a str,
}

/// Sendar outgoing request payload per technical specification.
#[derive(Debug, Serialize)]
struct SendarRequestPayload<'a> {
    contact: Vec<SendarContactPayload<'a>>,
    wallet_type: &'a str,
    sender_id: &'a str,
}

/// Sendar single recipient data inside response.
#[derive(Debug, Deserialize)]
struct SendarDataEntry {
    #[allow(dead_code)]
    number: Option<String>,
    message_id: Option<String>,
    status: Option<String>,
}

/// Sendar API response schema per technical specification.
#[derive(Debug, Deserialize)]
struct SendarApiResponse {
    status: Option<serde_json::Value>,
    #[allow(dead_code)]
    total: Option<u64>,
    #[allow(dead_code)]
    queued: Option<u64>,
    batch_id: Option<String>,
    #[allow(dead_code)]
    key_type: Option<String>,
    #[allow(dead_code)]
    total_cost: Option<f64>,
    data: Option<Vec<SendarDataEntry>>,
    message: Option<String>,
    error: Option<String>,
    detail: Option<String>,
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

        // Sendar technical specification expects international format (e.g. 2348147765111)
        let normalized_to = clean_to.strip_prefix('+').unwrap_or(clean_to);

        let url = format!("{}{}", self.base_url.trim_end_matches('/'), SEND_SMS_PATH);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let api_key_name = HeaderName::from_static("api-key");
        let header_val = HeaderValue::from_str(&self.api_key).map_err(|_| {
            SmsError::ConfigurationError("Invalid characters in Sendar API key".to_string())
        })?;
        headers.insert(api_key_name, header_val);

        let contact_entry = SendarContactPayload {
            number: normalized_to,
            body: clean_message,
            sms_type: &self.sms_type,
        };

        let payload = SendarRequestPayload {
            contact: vec![contact_entry],
            wallet_type: &self.wallet_type,
            sender_id: &self.sender_id,
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

        let is_status_ok = match &parsed.status {
            Some(serde_json::Value::Bool(b)) => *b,
            Some(serde_json::Value::String(s)) => {
                s == "true"
                    || s.eq_ignore_ascii_case("success")
                    || s.eq_ignore_ascii_case("queued")
                    || s.eq_ignore_ascii_case("ok")
            }
            _ => (200..300).contains(&status_code),
        };

        if is_status_ok && (200..300).contains(&status_code) {
            let msg_id = parsed
                .data
                .as_ref()
                .and_then(|entries| entries.first())
                .and_then(|entry| entry.message_id.clone())
                .or_else(|| parsed.batch_id.clone())
                .unwrap_or_else(|| "unknown_sendar_id".to_string());

            let status = parsed
                .data
                .as_ref()
                .and_then(|entries| entries.first())
                .and_then(|entry| entry.status.clone())
                .unwrap_or_else(|| "queued".to_string());

            Ok(SmsResponse::new(msg_id, status, self.name()).with_raw_response(raw_body))
        } else {
            let err_text = parsed
                .detail
                .or(parsed.error)
                .or(parsed.message)
                .unwrap_or_else(|| raw_body.clone());

            let lower = err_text.to_lowercase();
            if lower.contains("balance") || lower.contains("fund") || lower.contains("credit") {
                Err(SmsError::InsufficientBalance(err_text))
            } else if lower.contains("phone")
                || lower.contains("number")
                || lower.contains("recipient")
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
        "sendar"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sendar_redacts_api_key_in_debug() {
        let provider = SendarProvider::new("sk_live_super-secret-key-12345", "BrandApp");
        let debug_str = format!("{provider:?}");
        assert!(!debug_str.contains("sk_live_super-secret-key-12345"));
        assert!(debug_str.contains("[REDACTED]"));
        assert!(debug_str.contains("BrandApp"));
    }

    #[test]
    fn test_sendar_builder_valid() {
        let provider = SendarBuilder::new()
            .api_key("sk_live_123")
            .sender_id("BankID")
            .base_url("http://localhost:8080")
            .wallet_type("transactional")
            .build()
            .expect("should succeed");

        assert_eq!(provider.sender_id(), "BankID");
        assert_eq!(provider.base_url(), "http://localhost:8080");
        assert_eq!(provider.wallet_type(), "transactional");
        assert_eq!(provider.route(), "transactional");
        assert_eq!(provider.name(), "sendar");
    }

    #[test]
    fn test_sendar_builder_missing_credentials_fails() {
        let res = SendarBuilder::new().sender_id("BankID").build();
        assert!(matches!(res, Err(SmsError::ConfigurationError(_))));

        let res2 = SendarBuilder::new().api_key("sk_live_123").build();
        assert!(matches!(res2, Err(SmsError::ConfigurationError(_))));
    }

    #[test]
    fn test_sendar_payload_serialization() {
        let contact = SendarContactPayload {
            number: "2348147765111",
            body: "Hi Kene, your account has been activated.",
            sms_type: "plain",
        };
        let payload = SendarRequestPayload {
            contact: vec![contact],
            wallet_type: "transactional",
            sender_id: "LocalNews",
        };
        let json = serde_json::to_string(&payload).expect("serialization succeeds");
        assert!(json.contains("\"contact\":["));
        assert!(json.contains("\"number\":\"2348147765111\""));
        assert!(json.contains("\"sender_id\":\"LocalNews\""));
        assert!(json.contains("\"wallet_type\":\"transactional\""));
        assert!(json.contains("\"sms_type\":\"plain\""));
    }

    #[test]
    fn test_sendar_response_deserialization() {
        let raw = r#"{
            "status": true,
            "total": 1,
            "queued": 1,
            "batch_id": "4ab83a19-e851-42a3-be04-3a48c5134e23",
            "key_type": "live",
            "total_cost": 6.5,
            "data": [
                {
                    "number": "2348147765111",
                    "message_id": "4b1331d1-1ec2-4f5d-8b17-5c64ad8609ae",
                    "status": "queued"
                }
            ]
        }"#;

        let parsed: SendarApiResponse = serde_json::from_str(raw).expect("valid parse");
        assert_eq!(
            parsed.batch_id.as_deref(),
            Some("4ab83a19-e851-42a3-be04-3a48c5134e23")
        );
        let first = parsed.data.as_ref().unwrap().first().unwrap();
        assert_eq!(
            first.message_id.as_deref(),
            Some("4b1331d1-1ec2-4f5d-8b17-5c64ad8609ae")
        );
        assert_eq!(first.status.as_deref(), Some("queued"));
    }

    #[test]
    fn test_sendar_delivery_report_deserialization() {
        let raw = r#"{
            "message_id": "4b1331d1-1ec2-4f5d-8b17-5c64ad8609ae",
            "status": "delivered",
            "to": "2348147765111",
            "sender_id": "LocalNews",
            "cost_per_unit": 6.5,
            "billing_event_id": "Txn_batch_43a19",
            "gateway_message_id": "00000010",
            "created_at": "2026-04-13T17:44:47.911324Z",
            "updated_at": "2026-04-13T17:44:47.911324Z"
        }"#;

        let status: SendarMessageStatus = serde_json::from_str(raw).expect("valid parse");
        assert_eq!(status.message_id, "4b1331d1-1ec2-4f5d-8b17-5c64ad8609ae");
        assert_eq!(status.status, "delivered");
        assert_eq!(status.to.as_deref(), Some("2348147765111"));
        assert_eq!(status.cost_per_unit, Some(6.5));
    }

    #[tokio::test]
    async fn test_sendar_empty_recipient_fails() {
        let provider = SendarProvider::new("sk_live_test", "Sender");
        let err = provider.send_sms("", "Hello").await.unwrap_err();
        assert!(matches!(err, SmsError::InvalidPhoneNumber { .. }));
    }

    #[tokio::test]
    async fn test_sendar_empty_message_fails() {
        let provider = SendarProvider::new("sk_live_test", "Sender");
        let err = provider.send_sms("2348000000000", "").await.unwrap_err();
        assert!(matches!(err, SmsError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn test_sendar_empty_message_id_for_status_fails() {
        let provider = SendarProvider::new("sk_live_test", "Sender");
        let err = provider.get_message_status("   ").await.unwrap_err();
        assert!(matches!(err, SmsError::InvalidRequest(_)));
    }
}
