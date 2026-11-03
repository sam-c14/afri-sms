# afri-sms-skills

[![Crates.io](https://img.shields.io/crates/v/afri-sms-skills.svg)](https://crates.io/crates/afri-sms-skills)
[![Documentation](https://docs.rs/afri-sms-skills/badge.svg)](https://docs.rs/afri-sms-skills)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../Cargo.toml)

LLM agent tool definitions, JSON schemas, prompt definitions, and CLI scaffolder for [`afri-sms`](https://crates.io/crates/afri-sms).

Designed for integration with modern agent frameworks (OpenAI Function Calling, Anthropic Tools, Gemini Function Declarations, Rig, LangChain, AutoGen).

---

## CLI Installation & Scaffolding (Like `npx`)

Install globally as a CLI tool:

```bash
cargo install afri-sms-skills
```

### Scaffold Agent Skills Directly Into Your Repository:

```bash
# Automatically creates .agent/skills/afri-sms/ in the current directory
afri-sms-skills
```

Or target a custom directory:

```bash
afri-sms-skills --dir ./my-agent/skills/afri-sms
```

Output:
```text
✓ Created /path/to/project/.agent/skills/afri-sms/SKILL.md
✓ Created /path/to/project/.agent/skills/afri-sms/schema.json
✓ Created /path/to/project/.agent/skills/afri-sms/.env.example

Successfully scaffolded afri-sms skill into: /path/to/project/.agent/skills/afri-sms
```

---

## Library Usage (In Your Agent Application)

Add as a library dependency in `Cargo.toml`:

```toml
[dependencies]
afri-sms = "0.1.0"
afri-sms-skills = "0.1.1"
```

### Retrieve Function Calling JSON Schema

```rust
use afri_sms_skills::send_sms_tool_definition;

let tool_schema = send_sms_tool_definition();
println!("{}", serde_json::to_string_pretty(&tool_schema).unwrap());
```

### Parse Agent Function Call Payloads

```rust
use afri_sms_skills::{SendSmsToolInput, ProviderChoice};

let input: SendSmsToolInput = serde_json::from_str(r#"{
    "to": "+2348012345678",
    "message": "Security code: 882910",
    "provider": "termii"
}"#).unwrap();

assert_eq!(input.provider, Some(ProviderChoice::Termii));
```

### Direct Tool Execution

```rust
use afri_sms::{MultiSmsClient, SendarProvider};
use afri_sms_skills::{execute_tool, SendSmsToolInput};

#[tokio::main]
async fn main() {
    let client = MultiSmsClient::new(SendarProvider::new("api-key", "MyBrand"));
    let input = SendSmsToolInput::new("+2348012345678", "OTP: 9021");

    let result = execute_tool(&client, input).await;
    println!("Success: {}, Delivered by: {:?}", result.success, result.delivered_by);
}
```
