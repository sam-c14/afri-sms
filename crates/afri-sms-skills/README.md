# afri-sms-skills

[![Crates.io](https://img.shields.io/crates/v/afri-sms-skills.svg)](https://crates.io/crates/afri-sms-skills)
[![Documentation](https://docs.rs/afri-sms-skills/badge.svg)](https://docs.rs/afri-sms-skills)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../Cargo.toml)

LLM agent tool definitions, JSON schemas, prompt definitions, and CLI scaffolder for [`afri-sms`](https://crates.io/crates/afri-sms).

Designed for integration with modern agent frameworks (OpenAI Function Calling, Anthropic Tools, Gemini Function Declarations, Rig, LangChain, AutoGen).

---

## Architecture Note

- **[`afri-sms`](https://crates.io/crates/afri-sms)** contains all runtime execution logic, provider adapters, fallback routing, and tool execution (`afri_sms::tools::{execute_tool, SendSmsToolInput, SendSmsToolOutput, ProviderChoice, send_sms_tool_definition}`).
- **`afri-sms-skills`** is purely responsible for **Agentic workflows**: prompt context, function-calling schemas, documentation, and repository scaffolding so autonomous agents understand the core library and generate correct code.

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

### CLI Inspection Commands:

```bash
# Print tool JSON Schema for LLM registration:
afri-sms-skills --schema

# Print full skill documentation (SKILL.md):
afri-sms-skills --docs

# Print environment template:
afri-sms-skills --env
```

---

## Library Usage (For Agent Orchestrators)

Add as a library dependency in `Cargo.toml`:

```toml
[dependencies]
afri-sms-skills = "0.1.2"
```

### Retrieve Function Calling JSON Schema

```rust
use afri_sms_skills::tool_schema_json;

let tool_schema = tool_schema_json();
println!("{}", serde_json::to_string_pretty(&tool_schema).unwrap());
```

### Inject Agent Skill Context into System Prompts

```rust
use afri_sms_skills::agent_prompt_context;

let context = agent_prompt_context();
println!("Agent Prompt Context:\n{}", context);
```

### Programmatic Workspace Scaffolding

```rust,no_run
use std::path::Path;
use afri_sms_skills::scaffold;

scaffold(Path::new(".agent/skills/afri-sms"), false).unwrap();
```

---

## Tool Execution (In `afri-sms`)

To execute SMS tools within your application or agent runtime, use `afri-sms`:

```toml
[dependencies]
afri-sms = "0.1.2"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

```rust
use afri_sms::{MultiSmsClient, SendarProvider};
use afri_sms::tools::{execute_tool, SendSmsToolInput, ProviderChoice};

#[tokio::main]
async fn main() {
    let client = MultiSmsClient::new(SendarProvider::new("api-key", "MyBrand"));
    let input = SendSmsToolInput::new("+2348012345678", "OTP: 9021")
        .with_provider(ProviderChoice::Sendar);

    let result = execute_tool(&client, input).await;
    println!("Success: {}, Delivered by: {:?}", result.success, result.delivered_by);
}
```
