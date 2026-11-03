//! SMS Provider trait and common data contracts.
//!
//! Defines the core [`SmsProvider`] async trait and the normalized [`SmsResponse`] structure.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::SmsError;

/// Standardized response returned by any [`SmsProvider`].
///
/// Normalizes the varying responses returned by distinct telecom/SMS provider APIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmsResponse {
    /// Unique message identifier assigned by the provider gateway.
    pub message_id: String,

    /// High-level delivery or acceptance status (e.g., "sent", "queued", "accepted").
    pub status: String,

    /// The name of the provider that handled the request (e.g., "sendar", "termii").
    pub provider: String,

    /// Optional raw JSON or text payload returned by the provider gateway for auditing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_response: Option<String>,
}

impl SmsResponse {
    /// Creates a new [`SmsResponse`] instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::SmsResponse;
    ///
    /// let res = SmsResponse::new("msg_01", "sent", "sendar");
    /// assert_eq!(res.message_id, "msg_01");
    /// assert_eq!(res.status, "sent");
    /// assert_eq!(res.provider, "sendar");
    /// assert!(res.raw_response.is_none());
    /// ```
    #[must_use]
    pub fn new(
        message_id: impl Into<String>,
        status: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            message_id: message_id.into(),
            status: status.into(),
            provider: provider.into(),
            raw_response: None,
        }
    }

    /// Attaches the provider's raw HTTP response payload for debugging or logging.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::SmsResponse;
    ///
    /// let res = SmsResponse::new("msg_01", "sent", "sendar")
    ///     .with_raw_response(r#"{"success":true,"id":"msg_01"}"#);
    /// assert_eq!(res.raw_response.as_deref(), Some(r#"{"success":true,"id":"msg_01"}"#));
    /// ```
    #[must_use]
    pub fn with_raw_response(mut self, raw: impl Into<String>) -> Self {
        self.raw_response = Some(raw.into());
        self
    }
}

/// The core asynchronous interface implemented by all SMS gateway adapters.
///
/// Any supported SMS gateway (e.g., Sendar, Termii, Message Central, BulkSMSNigeria)
/// must implement this trait.
///
/// # Examples
///
/// ```
/// use async_trait::async_trait;
/// use afri_sms::{SmsError, SmsProvider, SmsResponse};
///
/// struct MockProvider;
///
/// #[async_trait]
/// impl SmsProvider for MockProvider {
///     async fn send_sms(&self, to: &str, message: &str) -> Result<SmsResponse, SmsError> {
///         Ok(SmsResponse::new("mock_id", "sent", self.name()))
///     }
///
///     fn name(&self) -> &'static str {
///         "mock"
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), SmsError> {
///     let provider = MockProvider;
///     let res = provider.send_sms("+2348012345678", "Hello").await?;
///     assert_eq!(res.provider, "mock");
///     assert_eq!(res.status, "sent");
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait SmsProvider: Send + Sync {
    /// Dispatches an SMS message to the specified recipient phone number.
    ///
    /// # Arguments
    /// * `to` - Recipient phone number (typically in international E.164 format or provider-expected standard).
    /// * `message` - The text content to be delivered.
    ///
    /// # Errors
    /// Returns an [`SmsError`] if the provider rejects the payload, the network fails, or credentials are invalid.
    async fn send_sms(&self, to: &str, message: &str) -> Result<SmsResponse, SmsError>;

    /// Returns the static canonical identifier for this provider (e.g., `"sendar"`, `"termii"`).
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyProvider;

    #[async_trait]
    impl SmsProvider for DummyProvider {
        async fn send_sms(&self, to: &str, message: &str) -> Result<SmsResponse, SmsError> {
            if to.is_empty() {
                return Err(SmsError::InvalidRequest(
                    "Recipient cannot be empty".to_string(),
                ));
            }
            Ok(SmsResponse::new("dummy-id-456", "delivered", self.name())
                .with_raw_response(format!("{{\"to\":\"{to}\",\"msg\":\"{message}\"}}")))
        }

        fn name(&self) -> &'static str {
            "dummy"
        }
    }

    #[tokio::test]
    async fn test_dummy_provider_send_success() {
        let provider = DummyProvider;
        let response = provider
            .send_sms("+2348123456789", "Test verification code: 123456")
            .await
            .expect("Dummy send should succeed");

        assert_eq!(response.message_id, "dummy-id-456");
        assert_eq!(response.status, "delivered");
        assert_eq!(response.provider, "dummy");
        assert!(response.raw_response.is_some());
    }

    #[tokio::test]
    async fn test_dummy_provider_empty_recipient_fails() {
        let provider = DummyProvider;
        let err = provider
            .send_sms("", "Hello")
            .await
            .expect_err("Empty recipient should fail");

        match err {
            SmsError::InvalidRequest(msg) => {
                assert!(msg.contains("Recipient cannot be empty"));
            }
            other => panic!("Expected InvalidRequest, got {other:?}"),
        }
    }
}
