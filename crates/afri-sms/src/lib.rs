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
//!
//! ## Example
//!
//! ```rust
//! use afri_sms::{MultiSmsClient, SendarProvider, TermiiProvider, SmsError};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), SmsError> {
//!     let primary = SendarProvider::new("sendar-api-key", "FintechBrand");
//!     let fallback = TermiiProvider::new("termii-api-key", "FintechBrand");
//!
//!     let client = MultiSmsClient::builder()
//!         .primary(primary)
//!         .with_fallback(fallback)
//!         .build()?;
//!
//!     assert_eq!(client.primary_name(), "sendar");
//!     assert_eq!(client.fallback_name(), Some("termii"));
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod client;
pub mod error;
pub mod provider;
pub mod providers;

pub use client::{MultiSmsClient, MultiSmsClientBuilder};
pub use error::SmsError;
pub use provider::{SmsProvider, SmsResponse};
pub use providers::*;
