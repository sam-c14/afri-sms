//! Error types for the `afri-sms` crate.
//!
//! Provides a unified [`SmsError`] enumeration that normalizes HTTP errors,
//! provider API response failures, serialization issues, and network faults.

use thiserror::Error;

/// Unified error enum for all SMS operations across all providers.
///
/// Encapsulates network failures, HTTP status codes, provider validation errors,
/// and client configuration issues into normalized error variants.
#[derive(Debug, Error)]
pub enum SmsError {
    /// Network-level connection or transport failure (timeout, DNS resolution, connection dropped, etc.).
    #[error("Network transport error: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// HTTP or logical error returned by an SMS provider API.
    #[error("Provider error [HTTP {status_code}]: {message}")]
    ProviderError {
        /// HTTP status code returned by the provider gateway.
        status_code: u16,
        /// Descriptive error message returned by the provider.
        message: String,
    },

    /// The recipient phone number is invalid or improperly formatted.
    #[error("Invalid phone number '{phone_number}': {reason}")]
    InvalidPhoneNumber {
        /// The phone number that failed validation.
        phone_number: String,
        /// Description of why the phone number is invalid.
        reason: String,
    },

    /// Authentication failed due to missing, invalid, or expired API credentials.
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Account balance on the provider is insufficient to complete the operation.
    #[error("Insufficient provider balance: {0}")]
    InsufficientBalance(String),

    /// Rate limit or throttling threshold exceeded on the provider API.
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Failed to serialize request or deserialize response payload.
    #[error("Serialization/deserialization failure: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Request payload validation failed before submission.
    #[error("Invalid request parameters: {0}")]
    InvalidRequest(String),

    /// Configuration error (e.g. missing API keys or invalid base URLs).
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Both primary and fallback providers failed to deliver the SMS message.
    #[error("All providers failed. Primary error: {primary}, Fallback error: {fallback:?}")]
    AllProvidersFailed {
        /// Error encountered by the primary provider.
        primary: Box<SmsError>,
        /// Error encountered by the fallback provider, if configured.
        fallback: Option<Box<SmsError>>,
    },
}

impl SmsError {
    /// Checks if the error is considered transient/retriable (e.g. network timeout or HTTP 5xx server error).
    ///
    /// When this returns `true`, the fallback engine should attempt the secondary provider.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::SmsError;
    ///
    /// let server_error = SmsError::ProviderError {
    ///     status_code: 502,
    ///     message: "Bad Gateway".to_string(),
    /// };
    /// assert!(server_error.is_fallback_eligible());
    ///
    /// let client_error = SmsError::InvalidPhoneNumber {
    ///     phone_number: "invalid".to_string(),
    ///     reason: "Bad format".to_string(),
    /// };
    /// assert!(!client_error.is_fallback_eligible());
    /// ```
    #[must_use]
    pub fn is_fallback_eligible(&self) -> bool {
        match self {
            Self::NetworkError(_) => true,
            Self::ProviderError { status_code, .. } => *status_code >= 500 && *status_code < 600,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_eligible_provider_errors() {
        let err_500 = SmsError::ProviderError {
            status_code: 500,
            message: "Internal Server Error".to_string(),
        };
        assert!(err_500.is_fallback_eligible());

        let err_503 = SmsError::ProviderError {
            status_code: 503,
            message: "Service Unavailable".to_string(),
        };
        assert!(err_503.is_fallback_eligible());

        let err_400 = SmsError::ProviderError {
            status_code: 400,
            message: "Bad Request".to_string(),
        };
        assert!(!err_400.is_fallback_eligible());

        let err_401 = SmsError::AuthenticationFailed("Unauthorized".to_string());
        assert!(!err_401.is_fallback_eligible());

        let err_phone = SmsError::InvalidPhoneNumber {
            phone_number: "123".to_string(),
            reason: "Too short".to_string(),
        };
        assert!(!err_phone.is_fallback_eligible());
    }

    #[test]
    fn test_error_display_formatting() {
        let err = SmsError::InvalidPhoneNumber {
            phone_number: "+2348000000000".to_string(),
            reason: "Invalid country code".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid phone number '+2348000000000': Invalid country code"
        );

        let prov_err = SmsError::ProviderError {
            status_code: 504,
            message: "Gateway Timeout".to_string(),
        };
        assert_eq!(
            prov_err.to_string(),
            "Provider error [HTTP 504]: Gateway Timeout"
        );
    }
}
