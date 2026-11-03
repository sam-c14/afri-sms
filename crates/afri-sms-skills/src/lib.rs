//! # afri-sms-skills
//!
//! LLM agent tool definitions, function calling schemas, and prompt templates
//! for integrating `afri-sms` into agentic AI systems.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Placeholder module for agent tool definitions.
pub mod tools {
    /// Describes an SMS dispatch intent for an LLM agent.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    pub struct SendSmsToolInput {
        /// Recipient phone number in international format.
        pub to: String,
        /// Message content to dispatch.
        pub message: String,
    }
}
