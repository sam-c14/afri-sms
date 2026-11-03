//! Multi-provider SMS client with automated fallback routing.
//!
//! Provides [`MultiSmsClient`] and [`MultiSmsClientBuilder`] to orchestrate
//! redundant SMS gateway delivery across African telecom providers.

use std::fmt;

use async_trait::async_trait;
use tracing::{error, info, warn};

use crate::error::SmsError;
use crate::provider::{SmsProvider, SmsResponse};

/// A multi-provider SMS client capable of automated secondary fallback routing.
///
/// Dispatches outgoing SMS messages to a configured primary provider. If the primary
/// encounters a transient network failure or remote HTTP 5xx server error, traffic is
/// automatically rerouted to the secondary fallback provider.
///
/// # Examples
///
/// ```
/// use afri_sms::{MultiSmsClient, SendarProvider, TermiiProvider};
///
/// let primary = SendarProvider::new("sendar-key", "BrandID");
/// let fallback = TermiiProvider::new("termii-key", "BrandID");
///
/// let client = MultiSmsClient::builder()
///     .primary(primary)
///     .with_fallback(fallback)
///     .build()
///     .expect("valid client");
///
/// assert_eq!(client.primary_name(), "sendar");
/// assert_eq!(client.fallback_name(), Some("termii"));
/// ```
pub struct MultiSmsClient {
    primary: Box<dyn SmsProvider>,
    fallback: Option<Box<dyn SmsProvider>>,
}

impl fmt::Debug for MultiSmsClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MultiSmsClient")
            .field("primary", &self.primary.name())
            .field("fallback", &self.fallback.as_ref().map(|fb| fb.name()))
            .finish()
    }
}

impl MultiSmsClient {
    /// Creates a new [`MultiSmsClient`] with a primary provider and no fallback.
    ///
    /// # Arguments
    /// * `primary` - The main [`SmsProvider`] to use for SMS dispatch.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::{MultiSmsClient, SendarProvider};
    ///
    /// let sendar = SendarProvider::new("api-key", "Sender");
    /// let client = MultiSmsClient::new(sendar);
    /// assert_eq!(client.primary_name(), "sendar");
    /// assert!(client.fallback_name().is_none());
    /// ```
    #[must_use]
    pub fn new(primary: impl SmsProvider + 'static) -> Self {
        Self {
            primary: Box::new(primary),
            fallback: None,
        }
    }

    /// Initializes a fluent [`MultiSmsClientBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::{MultiSmsClient, SendarProvider};
    ///
    /// let client = MultiSmsClient::builder()
    ///     .primary(SendarProvider::new("key", "Sender"))
    ///     .build()
    ///     .expect("configured");
    /// ```
    #[must_use]
    pub fn builder() -> MultiSmsClientBuilder {
        MultiSmsClientBuilder::new()
    }

    /// Appends or updates a secondary fallback provider.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::{MultiSmsClient, SendarProvider, TermiiProvider};
    ///
    /// let client = MultiSmsClient::new(SendarProvider::new("k1", "ID"))
    ///     .with_fallback(TermiiProvider::new("k2", "ID"));
    ///
    /// assert_eq!(client.fallback_name(), Some("termii"));
    /// ```
    #[must_use]
    pub fn with_fallback(mut self, fallback: impl SmsProvider + 'static) -> Self {
        self.fallback = Some(Box::new(fallback));
        self
    }

    /// Returns the static canonical name of the primary provider.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::{MultiSmsClient, SendarProvider};
    ///
    /// let client = MultiSmsClient::new(SendarProvider::new("key", "Sender"));
    /// assert_eq!(client.primary_name(), "sendar");
    /// ```
    #[must_use]
    pub fn primary_name(&self) -> &'static str {
        self.primary.name()
    }

    /// Returns the static canonical name of the secondary fallback provider, if configured.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::{MultiSmsClient, SendarProvider, TermiiProvider};
    ///
    /// let client = MultiSmsClient::builder()
    ///     .primary(SendarProvider::new("k1", "ID"))
    ///     .with_fallback(TermiiProvider::new("k2", "ID"))
    ///     .build()
    ///     .expect("valid client");
    ///
    /// assert_eq!(client.fallback_name(), Some("termii"));
    /// ```
    #[must_use]
    pub fn fallback_name(&self) -> Option<&'static str> {
        self.fallback.as_ref().map(|fb| fb.name())
    }

    /// Dispatches an SMS message through the primary provider, automatically retrying with
    /// the secondary fallback provider if transient server/network failures occur.
    ///
    /// # Arguments
    /// * `to` - Recipient phone number.
    /// * `message` - The text content to be delivered.
    ///
    /// # Errors
    /// Returns [`SmsError`] if both providers fail, or if a non-transient client error occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use afri_sms::{MultiSmsClient, SmsError, SmsProvider, SmsResponse};
    ///
    /// struct MockProvider;
    /// #[async_trait]
    /// impl SmsProvider for MockProvider {
    ///     async fn send_sms(&self, to: &str, msg: &str) -> Result<SmsResponse, SmsError> {
    ///         Ok(SmsResponse::new("msg-1", "sent", self.name()))
    ///     }
    ///     fn name(&self) -> &'static str { "mock" }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), SmsError> {
    ///     let client = MultiSmsClient::new(MockProvider);
    ///     let res = client.send_sms("+2348012345678", "Hello!").await?;
    ///     assert_eq!(res.status, "sent");
    ///     Ok(())
    /// }
    /// ```
    pub async fn send_sms(&self, to: &str, message: &str) -> Result<SmsResponse, SmsError> {
        info!(
            provider = self.primary.name(),
            to = to,
            "Dispatching SMS via primary provider"
        );

        match self.primary.send_sms(to, message).await {
            Ok(response) => {
                info!(
                    provider = self.primary.name(),
                    message_id = response.message_id.as_str(),
                    "Primary provider successfully dispatched SMS"
                );
                Ok(response)
            }
            Err(primary_err) => {
                if primary_err.is_fallback_eligible() {
                    if let Some(ref fallback) = self.fallback {
                        warn!(
                            primary = self.primary.name(),
                            fallback = fallback.name(),
                            error = %primary_err,
                            to = to,
                            "Primary provider failed with transient/server error; triggering fallback"
                        );

                        match fallback.send_sms(to, message).await {
                            Ok(fallback_response) => {
                                info!(
                                    fallback = fallback.name(),
                                    message_id = fallback_response.message_id.as_str(),
                                    "Fallback provider successfully dispatched SMS"
                                );
                                Ok(fallback_response)
                            }
                            Err(fallback_err) => {
                                error!(
                                    primary = self.primary.name(),
                                    primary_error = %primary_err,
                                    fallback = fallback.name(),
                                    fallback_error = %fallback_err,
                                    to = to,
                                    "Both primary and fallback SMS providers failed"
                                );
                                Err(SmsError::AllProvidersFailed {
                                    primary: Box::new(primary_err),
                                    fallback: Some(Box::new(fallback_err)),
                                })
                            }
                        }
                    } else {
                        warn!(
                            primary = self.primary.name(),
                            error = %primary_err,
                            to = to,
                            "Primary provider failed but no fallback provider is configured"
                        );
                        Err(primary_err)
                    }
                } else {
                    // Non-transient errors (invalid phone, bad request, auth failed) should not trigger fallback
                    warn!(
                        primary = self.primary.name(),
                        error = %primary_err,
                        to = to,
                        "Primary provider returned non-retriable error; aborting without fallback"
                    );
                    Err(primary_err)
                }
            }
        }
    }
}

#[async_trait]
impl SmsProvider for MultiSmsClient {
    async fn send_sms(&self, to: &str, message: &str) -> Result<SmsResponse, SmsError> {
        self.send_sms(to, message).await
    }

    fn name(&self) -> &'static str {
        self.primary.name()
    }
}

/// Builder for creating and configuring a [`MultiSmsClient`].
#[derive(Default)]
pub struct MultiSmsClientBuilder {
    primary: Option<Box<dyn SmsProvider>>,
    fallback: Option<Box<dyn SmsProvider>>,
}

impl fmt::Debug for MultiSmsClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MultiSmsClientBuilder")
            .field("primary", &self.primary.as_ref().map(|p| p.name()))
            .field("fallback", &self.fallback.as_ref().map(|f| f.name()))
            .finish()
    }
}

impl MultiSmsClientBuilder {
    /// Creates a new, unconfigured [`MultiSmsClientBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::MultiSmsClientBuilder;
    ///
    /// let builder = MultiSmsClientBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the primary SMS gateway provider.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::{MultiSmsClientBuilder, SendarProvider};
    ///
    /// let builder = MultiSmsClientBuilder::new()
    ///     .primary(SendarProvider::new("api-key", "Sender"));
    /// ```
    #[must_use]
    pub fn primary(mut self, provider: impl SmsProvider + 'static) -> Self {
        self.primary = Some(Box::new(provider));
        self
    }

    /// Configures the secondary fallback SMS gateway provider.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::{MultiSmsClientBuilder, SendarProvider, TermiiProvider};
    ///
    /// let builder = MultiSmsClientBuilder::new()
    ///     .primary(SendarProvider::new("k1", "ID"))
    ///     .with_fallback(TermiiProvider::new("k2", "ID"));
    /// ```
    #[must_use]
    pub fn with_fallback(mut self, provider: impl SmsProvider + 'static) -> Self {
        self.fallback = Some(Box::new(provider));
        self
    }

    /// Validates and constructs the [`MultiSmsClient`].
    ///
    /// # Errors
    /// Returns [`SmsError::ConfigurationError`] if no primary provider was set.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::{MultiSmsClientBuilder, SendarProvider};
    ///
    /// let client = MultiSmsClientBuilder::new()
    ///     .primary(SendarProvider::new("api-key", "Sender"))
    ///     .build();
    /// assert!(client.is_ok());
    /// ```
    pub fn build(self) -> Result<MultiSmsClient, SmsError> {
        let primary = match self.primary {
            Some(p) => p,
            None => {
                return Err(SmsError::ConfigurationError(
                    "Primary SMS provider must be configured".to_string(),
                ));
            }
        };

        Ok(MultiSmsClient {
            primary,
            fallback: self.fallback,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSuccessProvider {
        provider_name: &'static str,
    }

    #[async_trait]
    impl SmsProvider for MockSuccessProvider {
        async fn send_sms(&self, _to: &str, _message: &str) -> Result<SmsResponse, SmsError> {
            Ok(SmsResponse::new("mock_id", "sent", self.name()))
        }

        fn name(&self) -> &'static str {
            self.provider_name
        }
    }

    struct MockFailingProvider {
        provider_name: &'static str,
        error_to_return: fn() -> SmsError,
    }

    #[async_trait]
    impl SmsProvider for MockFailingProvider {
        async fn send_sms(&self, _to: &str, _message: &str) -> Result<SmsResponse, SmsError> {
            Err((self.error_to_return)())
        }

        fn name(&self) -> &'static str {
            self.provider_name
        }
    }

    #[tokio::test]
    async fn test_primary_success_does_not_trigger_fallback() {
        let primary = MockSuccessProvider {
            provider_name: "primary_success",
        };
        let fallback = MockSuccessProvider {
            provider_name: "fallback_provider",
        };

        let client = MultiSmsClient::builder()
            .primary(primary)
            .with_fallback(fallback)
            .build()
            .expect("client builds");

        let res = client
            .send_sms("+2348000000000", "Hello OTP")
            .await
            .expect("should succeed");

        assert_eq!(res.provider, "primary_success");
    }

    #[tokio::test]
    async fn test_fallback_triggered_on_500_provider_error() {
        let primary = MockFailingProvider {
            provider_name: "failing_primary",
            error_to_return: || SmsError::ProviderError {
                status_code: 502,
                message: "Bad Gateway".to_string(),
            },
        };
        let fallback = MockSuccessProvider {
            provider_name: "healthy_fallback",
        };

        let client = MultiSmsClient::builder()
            .primary(primary)
            .with_fallback(fallback)
            .build()
            .expect("client builds");

        let res = client
            .send_sms("+2348000000000", "Hello OTP")
            .await
            .expect("fallback should succeed");

        assert_eq!(res.provider, "healthy_fallback");
    }

    #[tokio::test]
    async fn test_non_transient_error_does_not_trigger_fallback() {
        let primary = MockFailingProvider {
            provider_name: "primary_invalid_phone",
            error_to_return: || SmsError::InvalidPhoneNumber {
                phone_number: "+123".to_string(),
                reason: "Invalid length".to_string(),
            },
        };
        let fallback = MockSuccessProvider {
            provider_name: "healthy_fallback",
        };

        let client = MultiSmsClient::builder()
            .primary(primary)
            .with_fallback(fallback)
            .build()
            .expect("client builds");

        let err = client
            .send_sms("+123", "Hello OTP")
            .await
            .expect_err("should return primary non-transient error");

        match err {
            SmsError::InvalidPhoneNumber { phone_number, .. } => {
                assert_eq!(phone_number, "+123");
            }
            other => panic!("Expected InvalidPhoneNumber, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_both_providers_failing_returns_all_providers_failed() {
        let primary = MockFailingProvider {
            provider_name: "primary_fail",
            error_to_return: || SmsError::ProviderError {
                status_code: 500,
                message: "Internal server error".to_string(),
            },
        };
        let fallback = MockFailingProvider {
            provider_name: "fallback_fail",
            error_to_return: || SmsError::ProviderError {
                status_code: 503,
                message: "Service unavailable".to_string(),
            },
        };

        let client = MultiSmsClient::builder()
            .primary(primary)
            .with_fallback(fallback)
            .build()
            .expect("client builds");

        let err = client
            .send_sms("+2348000000000", "Hello OTP")
            .await
            .expect_err("should return AllProvidersFailed");

        match err {
            SmsError::AllProvidersFailed { primary, fallback } => {
                assert!(matches!(*primary, SmsError::ProviderError { .. }));
                assert!(matches!(
                    fallback.map(|f| *f),
                    Some(SmsError::ProviderError { .. })
                ));
            }
            other => panic!("Expected AllProvidersFailed, got {other:?}"),
        }
    }

    #[test]
    fn test_builder_missing_primary_fails() {
        let res = MultiSmsClientBuilder::new().build();
        assert!(matches!(res, Err(SmsError::ConfigurationError(_))));
    }

    #[test]
    fn test_client_debug_formatting() {
        let client = MultiSmsClient::builder()
            .primary(MockSuccessProvider {
                provider_name: "prim",
            })
            .with_fallback(MockSuccessProvider {
                provider_name: "fall",
            })
            .build()
            .expect("builds");

        let debug_str = format!("{client:?}");
        assert!(debug_str.contains("prim"));
        assert!(debug_str.contains("fall"));
    }
}
