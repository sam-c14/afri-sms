//! Agent tool definitions and JSON schemas.

use serde::{Deserialize, Serialize};

/// Supported African SMS gateway options selectable by an LLM agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderChoice {
    /// Modern JSON REST gateway.
    Sendar,
    /// Nigerian telecom aggregator standard.
    Termii,
    /// Multi-channel verification and OTP gateway.
    MessageCentral,
}

impl ProviderChoice {
    /// Returns the canonical provider name.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms_skills::ProviderChoice;
    ///
    /// assert_eq!(ProviderChoice::Sendar.as_str(), "sendar");
    /// assert_eq!(ProviderChoice::Termii.as_str(), "termii");
    /// assert_eq!(ProviderChoice::MessageCentral.as_str(), "message_central");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sendar => "sendar",
            Self::Termii => "termii",
            Self::MessageCentral => "message_central",
        }
    }
}

/// Input parameters for the `send_sms` agent tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendSmsToolInput {
    /// Recipient phone number in international or regional format (e.g., `+2348012345678`).
    pub to: String,

    /// Text content of the SMS or OTP message.
    pub message: String,

    /// Optional specific provider to prioritize. If omitted, default routing applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderChoice>,
}

impl SendSmsToolInput {
    /// Creates a new tool input parameter instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms_skills::SendSmsToolInput;
    ///
    /// let input = SendSmsToolInput::new("+2348012345678", "Hello from agent");
    /// assert_eq!(input.to, "+2348012345678");
    /// assert_eq!(input.message, "Hello from agent");
    /// assert!(input.provider.is_none());
    /// ```
    #[must_use]
    pub fn new(to: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            message: message.into(),
            provider: None,
        }
    }

    /// Sets the preferred provider.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms_skills::{SendSmsToolInput, ProviderChoice};
    ///
    /// let input = SendSmsToolInput::new("+2348012345678", "OTP: 1234")
    ///     .with_provider(ProviderChoice::Termii);
    /// assert_eq!(input.provider, Some(ProviderChoice::Termii));
    /// ```
    #[must_use]
    pub fn with_provider(mut self, provider: ProviderChoice) -> Self {
        self.provider = Some(provider);
        self
    }
}

/// Output returned to an LLM agent following an SMS dispatch attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendSmsToolOutput {
    /// Whether the message was successfully dispatched.
    pub success: bool,

    /// Unique identifier returned by the gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,

    /// Canonical name of the provider that successfully handled the delivery.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered_by: Option<String>,

    /// Human-readable status description or error details.
    pub details: String,
}

impl SendSmsToolOutput {
    /// Creates a successful tool execution output.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms_skills::SendSmsToolOutput;
    ///
    /// let output = SendSmsToolOutput::success("msg_123", "sendar", "Sent successfully");
    /// assert!(output.success);
    /// assert_eq!(output.message_id.as_deref(), Some("msg_123"));
    /// assert_eq!(output.delivered_by.as_deref(), Some("sendar"));
    /// ```
    #[must_use]
    pub fn success(
        message_id: impl Into<String>,
        delivered_by: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            success: true,
            message_id: Some(message_id.into()),
            delivered_by: Some(delivered_by.into()),
            details: details.into(),
        }
    }

    /// Creates an error tool execution output.
    ///
    /// # Examples
    ///
    /// ```
    /// use afri_sms_skills::SendSmsToolOutput;
    ///
    /// let output = SendSmsToolOutput::failure("Gateway timeout on primary and fallback");
    /// assert!(!output.success);
    /// assert!(output.message_id.is_none());
    /// ```
    #[must_use]
    pub fn failure(details: impl Into<String>) -> Self {
        Self {
            success: false,
            message_id: None,
            delivered_by: None,
            details: details.into(),
        }
    }
}

/// Returns the standard JSON Schema tool definition for `send_sms` compatible with LLM agent systems.
///
/// # Examples
///
/// ```
/// use afri_sms_skills::send_sms_tool_definition;
///
/// let def = send_sms_tool_definition();
/// assert_eq!(def["name"], "send_sms");
/// assert!(def["parameters"]["properties"]["to"].is_object());
/// ```
#[must_use]
pub fn send_sms_tool_definition() -> serde_json::Value {
    serde_json::json!({
        "name": "send_sms",
        "description": "Dispatches an SMS or OTP message to a recipient phone number across African telecom gateways with automated secondary fallback routing.",
        "parameters": {
            "type": "object",
            "properties": {
                "to": {
                    "type": "string",
                    "description": "Recipient phone number in E.164 format (e.g. +2348012345678, +254712345678)."
                },
                "message": {
                    "type": "string",
                    "description": "Text message content to deliver (e.g. OTP code, transaction alert, login notification)."
                },
                "provider": {
                    "type": "string",
                    "enum": ["sendar", "termii", "message_central"],
                    "description": "Optional specific gateway to prioritize."
                }
            },
            "required": ["to", "message"]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_sms_tool_schema_validity() {
        let schema = send_sms_tool_definition();
        assert_eq!(schema["name"], "send_sms");
        assert_eq!(schema["parameters"]["type"], "object");
        assert!(schema["parameters"]["required"].is_array());
    }

    #[test]
    fn test_tool_serialization_roundtrip() {
        let input = SendSmsToolInput::new("+2348012345678", "Code: 9988")
            .with_provider(ProviderChoice::Sendar);

        let json = serde_json::to_string(&input).expect("serializes");
        let deserialized: SendSmsToolInput = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(input, deserialized);
    }

    #[test]
    fn test_tool_output_constructors() {
        let ok = SendSmsToolOutput::success("m1", "termii", "Delivered");
        assert!(ok.success);
        assert_eq!(ok.message_id.as_deref(), Some("m1"));

        let fail = SendSmsToolOutput::failure("Connection dropped");
        assert!(!fail.success);
        assert_eq!(fail.details, "Connection dropped");
    }
}
