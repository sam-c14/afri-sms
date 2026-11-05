//! # afri-sms-skills
//!
//! Agentic documentation, prompt context, tool schemas, and scaffolding CLI
//! for the [`afri-sms`](https://crates.io/crates/afri-sms) ecosystem.
//!
//! ## Architecture
//!
//! This crate is dedicated entirely to **agentic workflows**: providing AI coding agents,
//! prompt templates, and autonomous frameworks with complete documentation and function-calling
//! schemas for integrating African SMS & OTP gateways.
//!
//! All runtime execution logic (including [`MultiSmsClient`](afri_sms::MultiSmsClient),
//! provider adapters, and [`execute_tool`](afri_sms::tools::execute_tool)) lives exclusively
//! within the core [`afri-sms`] crate.
//!
//! ## Usage
//!
//! ### Inspect Skill Documentation
//!
//! ```rust
//! use afri_sms_skills::{skill_markdown, tool_schema_json};
//!
//! let docs = skill_markdown();
//! assert!(docs.contains("African SMS & OTP Integration"));
//!
//! let schema = tool_schema_json();
//! assert_eq!(schema["title"], "SendSmsTool");
//! ```
//!
//! ### Scaffold Agent Skills into a Project
//!
//! You can use the CLI binary:
//! ```bash
//! cargo install afri-sms-skills
//! afri-sms-skills init
//! ```
//!
//! Or programmatically scaffold via the library:
//! ```rust,no_run
//! use std::path::Path;
//! use afri_sms_skills::scaffold;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! scaffold(Path::new(".agent/skills/afri-sms"), false)?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fs;
use std::path::Path;

/// Full markdown content of the `afri-sms` agent skill documentation (`SKILL.md`).
pub const SKILL_MD: &str = include_str!("../assets/SKILL.md");

/// Standard JSON Schema specification for the `send_sms` agent function calling tool.
pub const SCHEMA_JSON: &str = include_str!("../assets/schema.json");

/// Environment variable reference template (`.env.example`) for African telecom gateways.
pub const ENV_EXAMPLE: &str = include_str!("../assets/.env.example");

/// Returns the complete agent skill documentation markdown string.
///
/// # Examples
///
/// ```rust
/// use afri_sms_skills::skill_markdown;
///
/// let markdown = skill_markdown();
/// assert!(markdown.contains("SKILL: African SMS & OTP Integration"));
/// ```
#[must_use]
pub fn skill_markdown() -> &'static str {
    SKILL_MD
}

/// Returns the raw JSON schema string for LLM tool registration.
///
/// # Examples
///
/// ```rust
/// use afri_sms_skills::schema_json;
///
/// let schema_str = schema_json();
/// assert!(schema_str.contains("SendSmsTool"));
/// ```
#[must_use]
pub fn schema_json() -> &'static str {
    SCHEMA_JSON
}

/// Returns the `.env.example` template string.
///
/// # Examples
///
/// ```rust
/// use afri_sms_skills::env_example;
///
/// let env_str = env_example();
/// assert!(env_str.contains("SENDAR_API_KEY"));
/// ```
#[must_use]
pub fn env_example() -> &'static str {
    ENV_EXAMPLE
}

/// Returns the parsed JSON Schema as a [`serde_json::Value`].
///
/// # Examples
///
/// ```rust
/// use afri_sms_skills::tool_schema_json;
///
/// let schema = tool_schema_json();
/// assert_eq!(schema["properties"]["to"]["type"], "string");
/// ```
#[must_use]
pub fn tool_schema_json() -> serde_json::Value {
    serde_json::from_str(SCHEMA_JSON)
        .unwrap_or_else(|_| serde_json::json!({ "error": "invalid embedded schema" }))
}

/// Returns the agent prompt documentation context for injection into LLM system prompts.
///
/// # Examples
///
/// ```rust
/// use afri_sms_skills::agent_prompt_context;
///
/// let prompt = agent_prompt_context();
/// assert!(prompt.contains("African Telecom Provider Quirks"));
/// ```
#[must_use]
pub fn agent_prompt_context() -> &'static str {
    SKILL_MD
}

/// Scaffolds `.agent/skills/afri-sms/` assets into the specified target directory.
///
/// Writes `SKILL.md`, `schema.json`, and `.env.example` into `dest`. If `force` is `false`,
/// existing files will not be overwritten.
///
/// # Errors
///
/// Returns an error if directory creation or file writing fails.
pub fn scaffold(dest: &Path, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dest)?;

    let skill_path = dest.join("SKILL.md");
    let schema_path = dest.join("schema.json");
    let env_path = dest.join(".env.example");

    if !skill_path.exists() || force {
        fs::write(&skill_path, SKILL_MD)?;
    }

    if !schema_path.exists() || force {
        fs::write(&schema_path, SCHEMA_JSON)?;
    }

    if !env_path.exists() || force {
        fs::write(&env_path, ENV_EXAMPLE)?;
    }

    Ok(())
}

#[deprecated(
    since = "0.1.2",
    note = "Execution logic and tool types have moved to `afri_sms::tools`. The `afri-sms-skills` crate is exclusively for agentic documentation and scaffolding."
)]
pub mod tools {
    //! Deprecated: Execution logic and tool types have moved to [`afri_sms::tools`].
    //!
    //! The `afri-sms-skills` crate is strictly responsible for agentic documentation,
    //! prompt templates, schema definitions, and scaffolding assets.
    #[allow(deprecated)]
    pub use afri_sms::tools::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_docs_not_empty() {
        assert!(!skill_markdown().is_empty());
        assert!(!schema_json().is_empty());
        assert!(!env_example().is_empty());
    }

    #[test]
    fn test_tool_schema_json_valid() {
        let schema = tool_schema_json();
        assert_eq!(schema["title"], "SendSmsTool");
        assert_eq!(schema["properties"]["to"]["type"], "string");
        assert_eq!(schema["properties"]["message"]["type"], "string");
    }

    #[test]
    fn test_agent_prompt_context_contains_provider_guidelines() {
        let prompt = agent_prompt_context();
        assert!(prompt.contains("Sendar"));
        assert!(prompt.contains("Termii"));
        assert!(prompt.contains("Message Central"));
    }

    #[test]
    fn test_scaffold_creates_files() {
        let temp_dir = std::env::temp_dir().join(format!("afri_sms_test_{}", std::process::id()));
        let result = scaffold(&temp_dir, true);
        assert!(result.is_ok());

        assert!(temp_dir.join("SKILL.md").exists());
        assert!(temp_dir.join("schema.json").exists());
        assert!(temp_dir.join(".env.example").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
