//! Message Central SMS & OTP provider adapter.
//!
//! Provides integration with Message Central's CPaaS platform (`https://cpaas.messagecentral.com`),
//! implementing the Verify Now V3 API specification for token generation, OTP dispatch, and validation.

use std::fmt;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::error::SmsError;
use crate::provider::{SmsProvider, SmsResponse};

const DEFAULT_MESSAGE_CENTRAL_BASE_URL: &str = "https://cpaas.messagecentral.com";
const AUTH_TOKEN_PATH: &str = "/auth/v1/authentication/token";
const SEND_VERIFICATION_PATH: &str = "/verification/v3/send";
const VALIDATE_OTP_PATH: &str = "/verification/v3/validateOtp";
const DEFAULT_FLOW_TYPE: &str = "SMS";
const DEFAULT_TYPE: &str = "OTP";
const DEFAULT_COUNTRY_CODE: &str = "234";

/// Response data returned by Message Central when dispatching an OTP code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageCentralSendOtpResponse {
    /// Unique verification session identifier used for validation.
    pub verification_id: String,

    /// Destination mobile number.
    #[serde(default)]
    pub mobile_number: Option<String>,

    /// Response status code (e.g., "200").
    #[serde(default)]
    pub response_code: Option<String>,

    /// OTP expiry timeout in seconds (e.g. "60").
    #[serde(default)]
    pub timeout: Option<String>,

    /// CPaaS transaction identifier.
    #[serde(default)]
    pub transaction_id: Option<String>,
}

/// SMS gateway adapter for Message Central.
///
/// Implements [`SmsProvider`] to dispatch SMS and OTP verification messages via Message Central Verify Now V3.
///
/// Secrets such as authentication tokens are redacted in `Debug` output to prevent credential leaks.
pub struct MessageCentralProvider {
    auth_token: String,
    sender_id: Option<String>,
    customer_id: Option<String>,
    default_country_code: String,
    flow_type: String,
    base_url: String,
    client: reqwest::Client,
}

impl fmt::Debug for MessageCentralProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageCentralProvider")
            .field("auth_token", &"[REDACTED]")
            .field("sender_id", &self.sender_id)
            .field("customer_id", &self.customer_id)
            .field("default_country_code", &self.default_country_code)
            .field("flow_type", &self.flow_type)
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl MessageCentralProvider {
    /// Creates a new [`MessageCentralProvider`] with default configuration.
    ///
    /// # Arguments
    /// * `auth_token` - The JWT or authentication token issued by Message Central.
    /// * `sender_id` - The registered sender ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::MessageCentralProvider;
    ///
    /// let provider = MessageCentralProvider::new("mc-auth-token-123", "BrandAlert");
    /// ```
    #[must_use]
    pub fn new(auth_token: impl Into<String>, sender_id: impl Into<String>) -> Self {
        let s_id = sender_id.into();
        let opt_sender = if s_id.trim().is_empty() {
            None
        } else {
            Some(s_id)
        };
        Self {
            auth_token: auth_token.into(),
            sender_id: opt_sender,
            customer_id: None,
            default_country_code: DEFAULT_COUNTRY_CODE.to_string(),
            flow_type: DEFAULT_FLOW_TYPE.to_string(),
            base_url: DEFAULT_MESSAGE_CENTRAL_BASE_URL.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Creates a new [`MessageCentralProvider`] using only an auth token (without a custom sender ID).
    ///
    /// # Arguments
    /// * `auth_token` - The JWT or authentication token issued by Message Central.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::MessageCentralProvider;
    ///
    /// let provider = MessageCentralProvider::with_token("mc-auth-token-123");
    /// ```
    #[must_use]
    pub fn with_token(auth_token: impl Into<String>) -> Self {
        Self {
            auth_token: auth_token.into(),
            sender_id: None,
            customer_id: None,
            default_country_code: DEFAULT_COUNTRY_CODE.to_string(),
            flow_type: DEFAULT_FLOW_TYPE.to_string(),
            base_url: DEFAULT_MESSAGE_CENTRAL_BASE_URL.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Creates a builder to configure a [`MessageCentralProvider`].
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::MessageCentralBuilder;
    ///
    /// let provider = MessageCentralBuilder::new()
    ///     .auth_token("mc-auth-token-123")
    ///     .sender_id("BrandAlert")
    ///     .default_country_code("234")
    ///     .build()
    ///     .expect("valid configuration");
    /// ```
    #[must_use]
    pub fn builder() -> MessageCentralBuilder {
        MessageCentralBuilder::new()
    }

    /// Returns the configured sender ID, if any.
    #[must_use]
    pub fn sender_id(&self) -> Option<&str> {
        self.sender_id.as_deref()
    }

    /// Returns the configured base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the configured flow type (e.g. `"SMS"` or `"WHATSAPP"`).
    #[must_use]
    pub fn flow_type(&self) -> &str {
        &self.flow_type
    }

    /// Generates an authentication token using Customer ID and Base64 encoded password key.
    ///
    /// Hits `GET /auth/v1/authentication/token`.
    ///
    /// # Arguments
    /// * `customer_id` - Your Message Central customer identifier.
    /// * `key` - Base-64 encrypted/encoded password.
    /// * `scope` - Optional scope, e.g. `"NEW"` for the first time.
    /// * `country` - Optional country dialing code (e.g. `"234"`).
    /// * `email` - Optional registered email address.
    ///
    /// # Errors
    /// Returns [`SmsError`] if the request fails or credentials are rejected.
    pub async fn generate_token(
        client: &reqwest::Client,
        base_url: &str,
        customer_id: &str,
        key: &str,
        scope: Option<&str>,
        country: Option<&str>,
        email: Option<&str>,
    ) -> Result<String, SmsError> {
        let url = format!("{}{}", base_url.trim_end_matches('/'), AUTH_TOKEN_PATH);

        let mut query = vec![("customerId", customer_id), ("key", key)];

        if let Some(s) = scope {
            query.push(("scope", s));
        }
        if let Some(c) = country {
            query.push(("country", c));
        }
        if let Some(e) = email {
            query.push(("email", e));
        }

        let response = client.get(&url).query(&query).send().await?;
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
            // Parse token from JSON response { "token": "..." } or return raw text if plain
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw_body) {
                if let Some(token) = val.get("token").and_then(|t| t.as_str()) {
                    return Ok(token.to_string());
                }
                if let Some(token) = val
                    .get("data")
                    .and_then(|d| d.get("token"))
                    .and_then(|t| t.as_str())
                {
                    return Ok(token.to_string());
                }
            }
            Ok(raw_body.trim().trim_matches('"').to_string())
        } else {
            Err(SmsError::ProviderError {
                status_code,
                message: raw_body,
            })
        }
    }

    /// Dispatches an OTP verification code to the recipient mobile number.
    ///
    /// Hits `POST /verification/v3/send` per the Verify Now V3 specification.
    ///
    /// # Arguments
    /// * `mobile_number` - Target phone number (e.g. `2348123456789`).
    /// * `flow_type` - Optional channel override (`SMS` or `WHATSAPP`).
    ///
    /// # Errors
    /// Returns [`SmsError`] if the dispatch fails.
    pub async fn send_otp(
        &self,
        mobile_number: &str,
        flow_type: Option<&str>,
    ) -> Result<MessageCentralSendOtpResponse, SmsError> {
        let clean_to = mobile_number.trim();
        let normalized_to = clean_to.strip_prefix('+').unwrap_or(clean_to);
        if normalized_to.is_empty() {
            return Err(SmsError::InvalidPhoneNumber {
                phone_number: mobile_number.to_string(),
                reason: "Phone number cannot be empty".to_string(),
            });
        }

        let channel = flow_type.unwrap_or(&self.flow_type);
        let (country_code, local_number) = self.parse_phone_number(normalized_to);

        let url = format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            SEND_VERIFICATION_PATH
        );

        let mut headers = HeaderMap::new();
        let token_header_name = HeaderName::from_static("authtoken");
        let token_val = HeaderValue::from_str(&self.auth_token).map_err(|_| {
            SmsError::ConfigurationError(
                "Invalid characters in Message Central auth_token".to_string(),
            )
        })?;
        headers.insert(token_header_name, token_val);

        let mut query_params = vec![
            ("countryCode", country_code),
            ("mobileNumber", local_number),
            ("flowType", channel),
            ("type", DEFAULT_TYPE),
        ];

        if let Some(ref sender) = self.sender_id
            && !sender.trim().is_empty()
        {
            query_params.push(("senderId", sender.as_str()));
        }

        if let Some(ref cust_id) = self.customer_id {
            query_params.push(("customerId", cust_id.as_str()));
        }

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .query(&query_params)
            .send()
            .await?;

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

        let parsed: MessageCentralResponse =
            serde_json::from_str(&raw_body).map_err(|_| SmsError::ProviderError {
                status_code,
                message: raw_body.clone(),
            })?;

        let is_ok_code = match parsed.response_code {
            Some(serde_json::Value::Number(n)) => n.as_u64() == Some(200),
            Some(serde_json::Value::String(ref s)) => s == "200" || s.eq_ignore_ascii_case("ok"),
            _ => (200..300).contains(&status_code),
        };

        if is_ok_code && (200..300).contains(&status_code) {
            let data = parsed.data.ok_or_else(|| SmsError::ProviderError {
                status_code,
                message: "Missing data in Message Central response".to_string(),
            })?;

            let verification_id = data
                .verification_id
                .map(|v| match v {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Number(n) => n.to_string(),
                    other => other.to_string(),
                })
                .unwrap_or_else(|| "unknown_verification_id".to_string());

            Ok(MessageCentralSendOtpResponse {
                verification_id,
                mobile_number: data.mobile_number,
                response_code: data.response_code,
                timeout: data.timeout,
                transaction_id: data.transaction_id,
            })
        } else {
            let err_text = parsed
                .data
                .and_then(|d| d.error_message)
                .or(parsed.message)
                .unwrap_or_else(|| raw_body.clone());

            Err(SmsError::ProviderError {
                status_code,
                message: err_text,
            })
        }
    }

    /// Validates an OTP code entered by the end user against the verification session.
    ///
    /// Hits `POST /verification/v3/validateOtp` per the Verify Now V3 specification.
    ///
    /// # Arguments
    /// * `verification_id` - The `verificationId` returned from the `/send` API response.
    /// * `code` - The OTP code entered by the user.
    /// * `flow_type` - Channel used (`SMS` or `WHATSAPP`). Defaults to configured flow type.
    ///
    /// # Errors
    /// Returns [`SmsError`] if network transport fails or the server encounters errors.
    pub async fn validate_otp(
        &self,
        verification_id: &str,
        code: &str,
        flow_type: Option<&str>,
    ) -> Result<bool, SmsError> {
        let channel = flow_type.unwrap_or(&self.flow_type);
        let url = format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            VALIDATE_OTP_PATH
        );

        let mut headers = HeaderMap::new();
        let token_header_name = HeaderName::from_static("authtoken");
        let token_val = HeaderValue::from_str(&self.auth_token).map_err(|_| {
            SmsError::ConfigurationError(
                "Invalid characters in Message Central auth_token".to_string(),
            )
        })?;
        headers.insert(token_header_name, token_val);

        let query_params = [
            ("verificationId", verification_id),
            ("code", code),
            ("flowType", channel),
        ];

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .query(&query_params)
            .send()
            .await?;

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

        let parsed: MessageCentralResponse =
            serde_json::from_str(&raw_body).map_err(|_| SmsError::ProviderError {
                status_code,
                message: raw_body.clone(),
            })?;

        let is_ok = match parsed.response_code {
            Some(serde_json::Value::Number(n)) => n.as_u64() == Some(200),
            Some(serde_json::Value::String(ref s)) => s == "200" || s.eq_ignore_ascii_case("ok"),
            _ => (200..300).contains(&status_code),
        };

        let msg = parsed.message.as_deref().unwrap_or_default();
        Ok(is_ok && (msg.eq_ignore_ascii_case("success") || (200..300).contains(&status_code)))
    }

    /// Extracts the country code and local mobile number from a destination string.
    fn parse_phone_number<'a>(&'a self, to: &'a str) -> (&'a str, &'a str) {
        let clean = to.trim().strip_prefix('+').unwrap_or(to.trim());

        // Check common African country codes
        const CODES: &[&str] = &["234", "254", "233", "256", "27", "20", "212", "250"];
        for code in CODES {
            if let Some(rest) = clean.strip_prefix(code) {
                return (code, rest);
            }
        }

        // If starts with domestic '0', strip leading zero and use default country code
        if let Some(rest) = clean.strip_prefix('0') {
            (&self.default_country_code, rest)
        } else {
            (&self.default_country_code, clean)
        }
    }
}

/// Builder for constructing a customized [`MessageCentralProvider`].
#[derive(Debug, Default)]
pub struct MessageCentralBuilder {
    auth_token: Option<String>,
    sender_id: Option<String>,
    customer_id: Option<String>,
    default_country_code: Option<String>,
    flow_type: Option<String>,
    base_url: Option<String>,
    client: Option<reqwest::Client>,
}

impl MessageCentralBuilder {
    /// Creates an empty [`MessageCentralBuilder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::MessageCentralBuilder;
    ///
    /// let builder = MessageCentralBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Message Central authentication token.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::MessageCentralBuilder;
    ///
    /// let builder = MessageCentralBuilder::new().auth_token("token_123");
    /// ```
    #[must_use]
    pub fn auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Sets the registered sender identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::MessageCentralBuilder;
    ///
    /// let builder = MessageCentralBuilder::new().sender_id("BrandAlert");
    /// ```
    #[must_use]
    pub fn sender_id(mut self, sender: impl Into<String>) -> Self {
        self.sender_id = Some(sender.into());
        self
    }

    /// Sets the optional customer ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::MessageCentralBuilder;
    ///
    /// let builder = MessageCentralBuilder::new().customer_id("cust_456");
    /// ```
    #[must_use]
    pub fn customer_id(mut self, customer_id: impl Into<String>) -> Self {
        self.customer_id = Some(customer_id.into());
        self
    }

    /// Sets the fallback default country code (e.g. `"234"` for Nigeria).
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::MessageCentralBuilder;
    ///
    /// let builder = MessageCentralBuilder::new().default_country_code("254");
    /// ```
    #[must_use]
    pub fn default_country_code(mut self, code: impl Into<String>) -> Self {
        self.default_country_code = Some(code.into());
        self
    }

    /// Sets the dispatch flow type (e.g., `"SMS"` or `"WHATSAPP"`).
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::MessageCentralBuilder;
    ///
    /// let builder = MessageCentralBuilder::new().flow_type("WHATSAPP");
    /// ```
    #[must_use]
    pub fn flow_type(mut self, flow_type: impl Into<String>) -> Self {
        self.flow_type = Some(flow_type.into());
        self
    }

    /// Overrides the default base URL.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::MessageCentralBuilder;
    ///
    /// let builder = MessageCentralBuilder::new().base_url("http://127.0.0.1:8080");
    /// ```
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Reuses a custom `reqwest::Client`.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::MessageCentralBuilder;
    ///
    /// let client = reqwest::Client::new();
    /// let builder = MessageCentralBuilder::new().client(client);
    /// ```
    #[must_use]
    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Validates and constructs the [`MessageCentralProvider`].
    ///
    /// # Errors
    /// Returns [`SmsError::ConfigurationError`] if `auth_token` or `sender_id` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms::providers::MessageCentralBuilder;
    ///
    /// let provider = MessageCentralBuilder::new()
    ///     .auth_token("token-abc")
    ///     .sender_id("BankAlert")
    ///     .build();
    /// assert!(provider.is_ok());
    /// ```
    pub fn build(self) -> Result<MessageCentralProvider, SmsError> {
        let auth_token = match self.auth_token {
            Some(t) if !t.trim().is_empty() => t,
            _ => {
                return Err(SmsError::ConfigurationError(
                    "Message Central auth_token must not be empty".to_string(),
                ));
            }
        };

        let sender_id = self.sender_id.filter(|s| !s.trim().is_empty());

        Ok(MessageCentralProvider {
            auth_token,
            sender_id,
            customer_id: self.customer_id,
            default_country_code: self
                .default_country_code
                .unwrap_or_else(|| DEFAULT_COUNTRY_CODE.to_string()),
            flow_type: self
                .flow_type
                .unwrap_or_else(|| DEFAULT_FLOW_TYPE.to_string()),
            base_url: self
                .base_url
                .unwrap_or_else(|| DEFAULT_MESSAGE_CENTRAL_BASE_URL.to_string()),
            client: self.client.unwrap_or_default(),
        })
    }
}

/// Inner payload structure for Message Central data.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageCentralData {
    verification_id: Option<serde_json::Value>,
    transaction_id: Option<String>,
    mobile_number: Option<String>,
    response_code: Option<String>,
    timeout: Option<String>,
    error_message: Option<String>,
}

/// Message Central response schema.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageCentralResponse {
    response_code: Option<serde_json::Value>,
    message: Option<String>,
    data: Option<MessageCentralData>,
}

#[async_trait]
impl SmsProvider for MessageCentralProvider {
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

        let (country_code, mobile_number) = self.parse_phone_number(clean_to);

        let url = format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            SEND_VERIFICATION_PATH
        );

        let mut headers = HeaderMap::new();
        let token_header_name = HeaderName::from_static("authtoken");
        let token_val = HeaderValue::from_str(&self.auth_token).map_err(|_| {
            SmsError::ConfigurationError(
                "Invalid characters in Message Central auth_token".to_string(),
            )
        })?;
        headers.insert(token_header_name, token_val);

        let mut query_params = vec![
            ("countryCode", country_code),
            ("mobileNumber", mobile_number),
            ("flowType", &self.flow_type),
            ("type", DEFAULT_TYPE),
            ("message", clean_message),
        ];

        if let Some(ref sender) = self.sender_id
            && !sender.trim().is_empty()
        {
            query_params.push(("senderId", sender.as_str()));
        }

        if let Some(ref cust_id) = self.customer_id {
            query_params.push(("customerId", cust_id.as_str()));
        }

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .query(&query_params)
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

        let parsed: MessageCentralResponse =
            serde_json::from_str(&raw_body).map_err(|_| SmsError::ProviderError {
                status_code,
                message: raw_body.clone(),
            })?;

        let is_ok_code = match parsed.response_code {
            Some(serde_json::Value::Number(n)) => n.as_u64() == Some(200),
            Some(serde_json::Value::String(ref s)) => s == "200" || s.eq_ignore_ascii_case("ok"),
            _ => (200..300).contains(&status_code),
        };

        if is_ok_code && (200..300).contains(&status_code) {
            let message_id = parsed
                .data
                .as_ref()
                .and_then(|d| {
                    d.transaction_id.clone().or_else(|| {
                        d.verification_id.as_ref().map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Number(n) => n.to_string(),
                            _ => "unknown_mc_id".to_string(),
                        })
                    })
                })
                .unwrap_or_else(|| "unknown_mc_id".to_string());

            let status = parsed.message.unwrap_or_else(|| "sent".to_string());
            Ok(SmsResponse::new(message_id, status, self.name()).with_raw_response(raw_body))
        } else {
            let err_text = parsed
                .data
                .and_then(|d| d.error_message)
                .or(parsed.message)
                .unwrap_or_else(|| raw_body.clone());

            let lower = err_text.to_lowercase();
            if lower.contains("phone") || lower.contains("number") || lower.contains("mobile") {
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
        "message_central"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_central_redacts_auth_token_in_debug() {
        let provider = MessageCentralProvider::new("super-secret-jwt-token-12345", "BrandAlert");
        let debug_str = format!("{provider:?}");
        assert!(!debug_str.contains("super-secret-jwt-token-12345"));
        assert!(debug_str.contains("[REDACTED]"));
        assert!(debug_str.contains("BrandAlert"));
    }

    #[test]
    fn test_phone_number_parsing() {
        let provider = MessageCentralProvider::new("token", "Sender");

        let (code, num) = provider.parse_phone_number("+2348012345678");
        assert_eq!(code, "234");
        assert_eq!(num, "8012345678");

        let (code, num) = provider.parse_phone_number("254712345678");
        assert_eq!(code, "254");
        assert_eq!(num, "712345678");

        let (code, num) = provider.parse_phone_number("08012345678");
        assert_eq!(code, "234");
        assert_eq!(num, "8012345678");
    }

    #[test]
    fn test_message_central_builder() {
        let provider = MessageCentralBuilder::new()
            .auth_token("token-123")
            .sender_id("BankAlert")
            .customer_id("cust-99")
            .flow_type("WHATSAPP")
            .build()
            .expect("valid builder");

        assert_eq!(provider.sender_id(), Some("BankAlert"));
        assert_eq!(provider.flow_type(), "WHATSAPP");
        assert_eq!(provider.name(), "message_central");
    }

    #[test]
    fn test_message_central_send_otp_response_deserialization() {
        let raw = r#"{
            "responseCode": 200,
            "message": "SUCCESS",
            "data": {
                "verificationId": "v-1234-5678",
                "mobileNumber": "2348123456789",
                "responseCode": "200",
                "errorMessage": null,
                "timeout": "60",
                "smCLI": null,
                "transactionId": "txn-9988"
            }
        }"#;

        let parsed: MessageCentralResponse = serde_json::from_str(raw).expect("valid parse");
        let data = parsed.data.unwrap();
        assert_eq!(data.transaction_id.as_deref(), Some("txn-9988"));
        assert_eq!(data.timeout.as_deref(), Some("60"));
    }

    #[tokio::test]
    async fn test_message_central_empty_recipient_fails() {
        let provider = MessageCentralProvider::new("auth_tok", "Sender");
        let err = provider.send_sms("", "Hello").await.unwrap_err();
        assert!(matches!(err, SmsError::InvalidPhoneNumber { .. }));

        let err_otp = provider.send_otp("", None).await.unwrap_err();
        assert!(matches!(err_otp, SmsError::InvalidPhoneNumber { .. }));
    }
}
