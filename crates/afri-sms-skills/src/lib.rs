//! # afri-sms-skills
//!
//! LLM agent tool definitions, function calling schemas, and prompt templates
//! for integrating the [`afri-sms`] crate into autonomous AI agents.
//!
//! ## Overview
//!
//! Provides strongly-typed function calling schemas compatible with modern LLM
//! interfaces (e.g. OpenAI function calling, Gemini tool use, Anthropic tools).
//! Allows autonomous agents to inspect SMS provider status, format African recipient
//! phone numbers, and execute resilient SMS dispatches.
//!
//! ## Examples
//!
//! ```rust
//! use afri_sms_skills::tools::{SendSmsToolInput, ProviderChoice};
//!
//! let input = SendSmsToolInput {
//!     to: "+2348012345678".to_string(),
//!     message: "Your verification code is 482910".to_string(),
//!     provider: Some(ProviderChoice::Sendar),
//! };
//!
//! assert_eq!(input.to, "+2348012345678");
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod tools;

pub use tools::{
    ProviderChoice, SendSmsToolInput, SendSmsToolOutput, execute_tool, send_sms_tool_definition,
};
