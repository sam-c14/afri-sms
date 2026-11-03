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
//! * [`providers`]: Concrete implementations for African telecom providers:
//!   - [`SendarProvider`]: Modern JSON REST API.
//!   - [`TermiiProvider`]: Industry-standard Nigerian SMS gateway.
//!   - [`MessageCentralProvider`]: CPaaS SMS and OTP verification gateway.
//!
//! ## Example
//!
//! ```rust
//! use afri_sms::{SmsError, SmsProvider, SendarProvider};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), SmsError> {
//!     let provider = SendarProvider::builder()
//!         .api_key("your_sendar_api_key")
//!         .sender_id("FintechBrand")
//!         .route("transactional")
//!         .build()?;
//!
//!     assert_eq!(provider.name(), "sendar");
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod provider;
pub mod providers;

pub use error::SmsError;
pub use provider::{SmsProvider, SmsResponse};
pub use providers::*;
