//! # afri-sms
//!
//! A unified, highly-optimized SMS/OTP adapter crate for African telecom providers
//! with automated fallback routing.
//!
//! ## Overview
//!
//! African telecom and SMS aggregators differ wildly in API conventions, authentication
//! methods, and delivery reliability. `afri-sms` abstracts these differences behind
//! a unified Adapter interface, providing strongly-typed payloads, zero-boilerplate
//! error handling, and automated secondary fallback routing to maximize delivery rates.
//!
//! ## Core Architecture
//!
//! * [`MultiSmsClient`]: High-level orchestrator with automated secondary fallback routing.
//! * [`SmsProvider`]: The unified async trait that standardizes sending SMS across all gateways.
//! * [`SmsResponse`]: Normalized response containing provider ID, status, and optional raw response.
//! * [`SmsError`]: Unified error enum mapping provider-specific errors to common variants.
//! * [`providers`]: Concrete implementations for African telecom providers:
//!   - [`SendarProvider`]: Modern JSON REST API.
//!   - [`TermiiProvider`]: Industry-standard Nigerian SMS gateway.
//!   - [`MessageCentralProvider`]: CPaaS SMS and OTP verification gateway.
//! * [`tools`]: Function-calling schemas, tool definitions, and [`execute_tool`] dispatcher for autonomous agents and apps.
//!
//! ## Core Architecture & Five Key Use Cases
//!
//! `afri-sms` is structured around five primary operational use cases:
//!
//! ### 1. Multi-Provider Fallback Routing (`MultiSmsClient`)
//! Combines primary and secondary gateways, automatically failing over on transient errors (5xx server errors, network timeouts) while aborting on non-retriable 400 Bad Request errors:
//!
//! ```rust
//! use afri_sms::{MultiSmsClient, SendarProvider, TermiiProvider, SmsError};
//!
//! # async fn example() -> Result<(), SmsError> {
//! let primary = SendarProvider::builder()
//!     .api_key("sendar-api-key")
//!     .sender_id("BankAlert")
//!     .wallet_type("transactional")
//!     .build()?;
//!
//! let fallback = TermiiProvider::builder()
//!     .api_key("termii-api-key")
//!     .sender_id("BankAlert")
//!     .channel("dnd")
//!     .build()?;
//!
//! let client = MultiSmsClient::builder()
//!     .primary(primary)
//!     .with_fallback(fallback)
//!     .build()?;
//!
//! assert_eq!(client.primary_name(), "sendar");
//! assert_eq!(client.fallback_name(), Some("termii"));
//! # Ok(())
//! # }
//! ```
//!
//! ### 2. Sendar: Fast Transactional SMS & Status Lookup
//! Provides high-throughput JSON REST dispatch and real-time delivery status querying:
//!
//! ```rust
//! use afri_sms::{SendarProvider, SmsError};
//!
//! # async fn example() -> Result<(), SmsError> {
//! let sendar = SendarProvider::builder()
//!     .api_key("sendar-api-key")
//!     .sender_id("FintechBrand")
//!     .wallet_type("transactional")
//!     .build()?;
//!
//! assert_eq!(sendar.wallet_type(), "transactional");
//! // Dispatch SMS:
//! // let res = sendar.send_sms("+2348012345678", "Transfer of NGN 5,000 successful.").await?;
//! // Query delivery status by message ID:
//! // let status = sendar.get_message_status(&res.message_id).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 3. Termii: SMS, Dynamic OTP Tokens & In-App Tokens
//! Automates Nigerian phone sanitization (`+234...` to `234...`), dynamic OTP PIN generation with custom templates, and cost-free in-app token generation:
//!
//! ```rust
//! use afri_sms::{TermiiProvider, TermiiSendTokenRequest, TermiiGenerateTokenRequest, TermiiPinType, SmsError};
//!
//! # async fn example() -> Result<(), SmsError> {
//! let termii = TermiiProvider::builder()
//!     .api_key("termii-api-key")
//!     .sender_id("SecureApp")
//!     .channel("dnd")
//!     .build()?;
//!
//! // Dynamic SMS OTP Token
//! let token_req = TermiiSendTokenRequest {
//!     pin_type: TermiiPinType::Numeric,
//!     to: "2348012345678".to_string(),
//!     from: None,
//!     channel: Some("dnd".to_string()),
//!     pin_attempts: 3,
//!     pin_time_to_live: 10,
//!     pin_length: 6,
//!     pin_placeholder: "< 123456 >".to_string(),
//!     message_text: "Your security code is < 123456 >".to_string(),
//! };
//! // termii.send_token(token_req).await?;
//!
//! // In-App Token (zero SMS transmission cost)
//! let in_app_req = TermiiGenerateTokenRequest {
//!     pin_type: TermiiPinType::Numeric,
//!     phone_number: "2348012345678".to_string(),
//!     pin_attempts: 3,
//!     pin_time_to_live: 5,
//!     pin_length: 6,
//! };
//! // termii.generate_in_app_token(in_app_req).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 4. Message Central: CPaaS SMS & Verify Now V3 OTP
//! Provides CPaaS SMS delivery and the complete "Verify Now" OTP verification lifecycle (`send_otp` and `validate_otp`) with automatic country code splitting:
//!
//! ```rust
//! use afri_sms::{MessageCentralProvider, SmsError};
//!
//! # async fn example() -> Result<(), SmsError> {
//! let mc = MessageCentralProvider::builder()
//!     .auth_token("mc-auth-token")
//!     .customer_id("C-BA489D17CAE0402")
//!     .sender_id("BrandAlert")
//!     .default_country_code("234")
//!     .build()?;
//!
//! // 1. Initiate OTP verification session:
//! // let otp = mc.send_otp("+2348012345678", Some("SMS")).await?;
//! // 2. Validate OTP code entered by user:
//! // let is_valid = mc.validate_otp(&otp.verification_id, "482910", Some("SMS")).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### 5. LLM Agent Tools & Function Calling (`afri_sms::tools`)
//! Supplies strongly-typed function-calling JSON schemas and runtime dispatch execution for autonomous AI coding agents:
//!
//! ```rust
//! use afri_sms::{MultiSmsClient, SendarProvider};
//! use afri_sms::tools::{execute_tool, SendSmsToolInput, ProviderChoice, send_sms_tool_definition};
//!
//! # async fn example() {
//! // 1. Get standard LLM function-calling schema:
//! let schema = send_sms_tool_definition();
//! assert_eq!(schema["name"], "send_sms");
//!
//! // 2. Execute incoming tool call:
//! let client = MultiSmsClient::new(SendarProvider::new("api-key", "TestBrand"));
//! let input = SendSmsToolInput::new("+2348012345678", "OTP: 9812")
//!     .with_provider(ProviderChoice::Sendar);
//!
//! let output = execute_tool(&client, input).await;
//! assert!(output.details.contains("SMS") || !output.success);
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod client;
pub mod error;
pub mod provider;
pub mod providers;
pub mod tools;

pub use client::{MultiSmsClient, MultiSmsClientBuilder};
pub use error::SmsError;
pub use provider::{SmsProvider, SmsResponse};
pub use providers::*;
pub use tools::{
    ProviderChoice, SendSmsToolInput, SendSmsToolOutput, execute_tool, send_sms_tool_definition,
};
