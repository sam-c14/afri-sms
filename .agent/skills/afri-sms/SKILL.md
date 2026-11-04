---
name: afri-sms
description: Unified SMS & OTP adapter for African telecom providers (Sendar, Termii, Message Central) with automated fallback routing. Use when dispatching SMS messages, sending OTP verification codes, integrating African payment/fintech notifications, or configuring telecom gateways with automated failover.
---

# SKILL: African SMS & OTP Integration (`afri-sms`)

## Overview
`afri-sms` is a production-grade, asynchronous Rust SDK that unifies African telecom aggregators into a single, cohesive interface. It features an automated secondary fallback routing engine designed specifically to mitigate volatile carrier downtime and network timeouts common across African telecommunications infrastructure.

## Trigger Conditions & When to Use
Activate this skill when:
- Writing or maintaining code that sends SMS notifications, OTP codes, or 2FA verification to users in African countries (Nigeria, Kenya, Ghana, South Africa, etc.).
- Implementing gateway redundancy or failover between SMS aggregators (e.g., Sendar as primary, Termii as fallback).
- Integrating African telecom providers: **Sendar**, **Termii**, or **Message Central CPaaS**.
- Writing integration tests or mocking SMS delivery in Rust using `wiremock`.
- Building LLM autonomous agent tools for communication and verification workflows.

---

## Workspace Setup & Dependencies

Add the following to your `Cargo.toml`:

```toml
[dependencies]
afri-sms = "0.1.1"
afri-sms-skills = "0.1.1" # Optional: for agent schemas & tool execution helpers
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

---

## Environment Variables & Credentials

Create a `.env` file in your project root (a template is available in `.env.example`):

| Variable | Provider | Required | Description |
|---|---|---|---|
| `SENDAR_API_KEY` | Sendar | Yes | API-KEY authentication token |
| `SENDAR_SENDER_ID` | Sendar | Yes | Approved brand sender name |
| `SENDAR_WALLET_TYPE` | Sendar | No | `transactional` (default), `promotional`, or `otp` |
| `TERMII_API_KEY` | Termii | Yes | Termii API key (sent in JSON body) |
| `TERMII_SENDER_ID` | Termii | Yes | Approved sender ID (3 to 11 characters) |
| `TERMII_CHANNEL` | Termii | No | `dnd` (default for OTP) or `generic` |
| `MESSAGE_CENTRAL_AUTH_TOKEN` | Message Central | Yes | CPaaS bearer/JWT token (in `authToken` header) |
| `MESSAGE_CENTRAL_SENDER_ID` | Message Central | Yes | Approved sender ID / shortcode |
| `MESSAGE_CENTRAL_CUSTOMER_ID` | Message Central | No | Optional CPaaS customer ID |
| `MESSAGE_CENTRAL_COUNTRY_CODE`| Message Central | No | Default dialing code (e.g. `234`) |

---

## African Telecom Provider Quirks & Rules

### 1. Sendar (`sendar`)
- **Base URL:** `https://sms-submit.sendar.io/api/sms/send`
- **Authentication:** `API-KEY: <SENDAR_API_KEY>` HTTP header.
- **Payload:** JSON `{ "contact": [{ "number": "234...", "body": "...", "sms_type": "plain" }], "sender_id": "...", "wallet_type": "transactional" }`
- **Delivery Status:** `GET /get/sms/{uid}` queries real-time status by message ID.
- **Strengths:** Low-latency REST API, well-suited for high-throughput transactional alerts.

### 2. Termii (`termii`)
- **Base URL:** `https://api.ng.termii.com/api/sms/send`
- **Authentication:** In-body credentials: `{ "api_key": "...", ... }`. No auth header.
- **Phone Number Formatting:** **Termii rejects the leading `+` sign!**
  - E.164 `+2348012345678` must become `2348012345678`.
  - `afri-sms` handles this sanitization automatically for you.
- **Sender ID Rules:** Must be 3 to 11 alphanumeric characters. Unregistered sender IDs trigger an HTTP 400 error.
- **DND Route:** Nigeria's NCC enforces "Do Not Disturb" (DND). Always use the `dnd` channel for OTP and banking notifications to bypass carrier DND blocking.

### 3. Message Central (`message_central`)
- **Base URL:** `https://cpaas.messagecentral.com/verification/v3/send`
- **Authentication:** Custom HTTP header: `authToken: <MESSAGE_CENTRAL_AUTH_TOKEN>`.
- **Dialing Code Separation:** Message Central requires splitting the international dialing code (e.g., `234`) and national number. `afri-sms` automatically parses and splits E.164 phone numbers.

---

## Fallback Routing Decision Matrix

The core engine (`MultiSmsClient`) automatically determines whether an error is transient or deterministic:

| Error Type | Status / Source | Failover Action | Reason |
|---|---|---|---|
| **Server Error** | HTTP `500`, `502`, `503`, `504` | **Reroute to Fallback** | Aggregator gateway down or carrier route degraded. |
| **Network Timeout** | Connection timeout, DNS failure | **Reroute to Fallback** | Carrier pipe unresponsive. |
| **Client Error** | HTTP `400 Bad Request` | **Abort Immediately (No Failover)** | Invalid phone number or invalid payload; retrying burns credits. |
| **Unauthorized** | HTTP `401`, `403` | **Abort Immediately (No Failover)** | Invalid API key; fallback won't fix misconfigured credentials. |
| **Sender ID Error** | Sender ID unapproved | **Abort Immediately (No Failover)** | Telecom compliance violation. |

---

## Production Code Recipes

### Recipe 1: Resilient Multi-Provider Client with Fallback (Recommended)

```rust
use afri_sms::{MultiSmsClient, SendarProvider, TermiiProvider, SmsError};

#[tokio::main]
async fn main() -> Result<(), SmsError> {
    // 1. Initialize Primary Provider (Sendar)
    let sendar_key = std::env::var("SENDAR_API_KEY")
        .map_err(|e| SmsError::ConfigurationError(e.to_string()))?;
    let primary = SendarProvider::builder()
        .api_key(sendar_key)
        .sender_id("BankAlert")
        .route("transactional")
        .build()?;

    // 2. Initialize Secondary Fallback Provider (Termii)
    let termii_key = std::env::var("TERMII_API_KEY")
        .map_err(|e| SmsError::ConfigurationError(e.to_string()))?;
    let fallback = TermiiProvider::builder()
        .api_key(termii_key)
        .sender_id("BankAlert")
        .channel("dnd")
        .build()?;

    // 3. Compose MultiSmsClient
    let client = MultiSmsClient::builder()
        .primary(primary)
        .with_fallback(fallback)
        .build()?;

    // 4. Send Message (automatically fails over if primary fails with 5xx or timeout)
    let response = client
        .send_sms("+2348012345678", "Your verification OTP is 492019")
        .await?;

    println!("Delivered via '{}' [Message ID: {}]", response.provider, response.message_id);
    Ok(())
}
```

### Recipe 2: LLM Agent Tool Dispatching

When building an autonomous coding or conversational agent:

```rust
use afri_sms::{MultiSmsClient, SendarProvider, TermiiProvider};
use afri_sms_skills::{execute_tool, SendSmsToolInput, ProviderChoice};

#[tokio::main]
async fn main() {
    let client = MultiSmsClient::new(SendarProvider::new("api-key", "MyApp"));

    // Tool input typically deserialized from LLM function calling JSON
    let input = SendSmsToolInput {
        to: "+2348012345678".to_string(),
        message: "Your delivery is arriving in 15 minutes.".to_string(),
        provider: Some(ProviderChoice::Sendar),
    };

    let result = execute_tool(&client, input).await;
    if result.success {
        println!("Success: ID={}, Provider={:?}", 
            result.message_id.unwrap_or_default(), 
            result.delivered_by
        );
    } else {
        eprintln!("Delivery failed: {}", result.details);
    }
}
```

### Recipe 3: Integration Testing with `wiremock`

```rust
use afri_sms::{SendarProvider, TermiiProvider, MultiSmsClient, SmsProvider};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_primary_failover_to_secondary() {
    let primary_server = MockServer::start().await;
    let fallback_server = MockServer::start().await;

    // Primary server returns 503 Service Unavailable
    Mock::given(method("POST"))
        .and(path("/sms/send"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&primary_server)
        .await;

    // Fallback server succeeds with 200 OK
    Mock::given(method("POST"))
        .and(path("/api/sms/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "termii-9988",
            "message": "Successfully Sent",
            "balance": 500,
            "user": "TestUser"
        })))
        .mount(&fallback_server)
        .await;

    let primary = SendarProvider::builder()
        .api_key("test-key")
        .sender_id("TestApp")
        .base_url(primary_server.uri())
        .build()
        .unwrap();

    let fallback = TermiiProvider::builder()
        .api_key("test-key")
        .sender_id("TestApp")
        .base_url(fallback_server.uri())
        .build()
        .unwrap();

    let client = MultiSmsClient::builder()
        .primary(primary)
        .with_fallback(fallback)
        .build()
        .unwrap();

    let res = client.send_sms("+2348012345678", "Test fallback").await.unwrap();
    assert_eq!(res.provider, "termii");
    assert_eq!(res.message_id, "termii-9988");
}
```

### Recipe 4: Implementing a Custom Provider

Any custom SMS gateway can be seamlessly attached to `MultiSmsClient`:

```rust
use afri_sms::{SmsProvider, SmsResponse, SmsError};
use async_trait::async_trait;

pub struct MyCustomProvider {
    pub api_key: String,
}

#[async_trait]
impl SmsProvider for MyCustomProvider {
    fn name(&self) -> &'static str {
        "my_custom_provider"
    }

    async fn send_sms(&self, to: &str, message: &str) -> Result<SmsResponse, SmsError> {
        // Custom gateway HTTP request logic here...
        Ok(SmsResponse::new("custom-id-123", self.name(), "Delivered"))
    }
}
```

---

## Security & Diagnostics

1. **Automatic Credential Redaction:** All provider structs implement a custom `fmt::Debug` that automatically masks API keys and tokens with `[REDACTED]`. It is always safe to log provider instances with `tracing::debug!("{:?}", client)`.
2. **Structured Logging:** The crate emits telemetry using the `tracing` crate:
   - `tracing::info!` on successful dispatches.
   - `tracing::warn!` when the primary fails and fallback routing begins.
   - `tracing::error!` when all providers are exhausted.
3. **No Raw Panics:** Library code is guaranteed `#![forbid(unsafe_code)]` with zero unwrap or panic paths.
