# afri-sms-skills

[![Crates.io](https://img.shields.io/crates/v/afri-sms-skills.svg)](https://crates.io/crates/afri-sms-skills)
[![Documentation](https://docs.rs/afri-sms-skills/badge.svg)](https://docs.rs/afri-sms-skills)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../Cargo.toml)

LLM agent tool definitions, JSON schemas, and prompt definitions for `afri-sms`.

Designed for integration with modern agent frameworks (OpenAI Function Calling, Anthropic Tools, Gemini Function Declarations, Rig, LangChain, AutoGen).

## Installation

```toml
[dependencies]
afri-sms-skills = "0.1.0"
```

## Tool Definition Schema

Retrieve the JSON Schema definition for `send_sms`:

```rust
use afri_sms_skills::send_sms_tool_definition;

let tool_schema = send_sms_tool_definition();
println!("{}", serde_json::to_string_pretty(&tool_schema).unwrap());
```

## Parsing Agent Inputs

```rust
use afri_sms_skills::{SendSmsToolInput, ProviderChoice};

let input: SendSmsToolInput = serde_json::from_str(r#"{
    "to": "+2348012345678",
    "message": "Security code: 882910",
    "provider": "termii"
}"#).unwrap();

assert_eq!(input.provider, Some(ProviderChoice::Termii));
```
