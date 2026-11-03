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
//! * [`SmsProvider`]: The unified async trait that standardizes sending SMS across all gateways.
//! * [`SmsResponse`]: Normalized response containing provider ID, status, and optional raw response.
//! * [`SmsError`]: Unified error enum mapping provider-specific errors to common variants.
//!
//! ## Example
//!
//! ```rust
//! use async_trait::async_trait;
//! use afri_sms::{SmsError, SmsProvider, SmsResponse};
//!
//! struct MockProvider;
//!
//! #[async_trait]
//! impl SmsProvider for MockProvider {
//!     async fn send_sms(&self, to: &str, message: &str) -> Result<SmsResponse, SmsError> {
//!         Ok(SmsResponse::new("msg_001", "sent", self.name()))
//!     }
//!
//!     fn name(&self) -> &'static str {
//!         "mock"
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), SmsError> {
//!     let provider = MockProvider;
//!     let response = provider.send_sms("+2348012345678", "Your OTP is 987654").await?;
//!     assert_eq!(response.message_id, "msg_001");
//!     assert_eq!(response.provider, "mock");
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod provider;

pub use error::SmsError;
pub use provider::{SmsProvider, SmsResponse};
