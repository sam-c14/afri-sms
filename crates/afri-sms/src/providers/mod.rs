//! SMS gateway provider implementations.
//!
//! Provides adapter implementations of [`SmsProvider`](crate::SmsProvider)
//! for major African telecom providers:
//! - [`SendarProvider`]: Modern JSON REST API.
//! - [`TermiiProvider`]: Industry-standard Nigerian SMS gateway.
//! - [`MessageCentralProvider`]: Multi-channel CPaaS gateway.

pub mod message_central;
pub mod sendar;
pub mod termii;

pub use message_central::{MessageCentralBuilder, MessageCentralProvider};
pub use sendar::{SendarBuilder, SendarProvider};
pub use termii::{TermiiBuilder, TermiiProvider};
