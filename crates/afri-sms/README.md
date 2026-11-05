# afri-sms

[![Crates.io](https://img.shields.io/crates/v/afri-sms.svg)](https://crates.io/crates/afri-sms)
[![Documentation](https://docs.rs/afri-sms/badge.svg)](https://docs.rs/afri-sms)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../Cargo.toml)

The core implementation crate for `afri-sms`.

Provides asynchronous adapters for African telecom gateways (Sendar, Termii, Message Central) alongside an automated secondary fallback routing engine and LLM agent function-calling execution tools.

## Installation

```toml
[dependencies]
afri-sms = "0.1.2"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

---

## The Five Key Use Cases

### 1. Multi-Provider Secondary Fallback Routing (`MultiSmsClient`)

Automatically fails over to the secondary provider if the primary experiences transient outages (5xx server errors, timeouts) while immediately aborting without wasting credits on client errors (400 Bad Request):

```rust
use afri_sms::{MultiSmsClient, SendarProvider, TermiiProvider, SmsError};

#[tokio::main]
async fn main() -> Result<(), SmsError> {
    let primary = SendarProvider::builder()
        .api_key("sendar_live_key")
        .sender_id("BankAlert")
        .wallet_type("transactional")
        .build()?;

    let fallback = TermiiProvider::builder()
        .api_key("termii_live_key")
        .sender_id("BankAlert")
        .channel("dnd")
        .build()?;

    let client = MultiSmsClient::builder()
        .primary(primary)
        .with_fallback(fallback)
        .build()?;

    let response = client
        .send_sms("+2348012345678", "Your OTP is 482910")
        .await?;

    println!("Delivered via {}: {}", response.provider, response.message_id);
    Ok(())
}
```

### 2. Sendar: Fast Transactional SMS & Status Lookup

High-speed JSON REST gateway for transactional notifications with real-time delivery status querying:

```rust
use afri_sms::{SendarProvider, SmsError};

#[tokio::main]
async fn main() -> Result<(), SmsError> {
    let sendar = SendarProvider::builder()
        .api_key("sendar-api-key")
        .sender_id("FintechBrand")
        .wallet_type("transactional")
        .build()?;

    // Send SMS:
    let response = sendar.send_sms("+2348012345678", "Transfer of NGN 5,000 successful.").await?;

    // Query real-time delivery status by message ID:
    let status = sendar.get_message_status(&response.message_id).await?;
    println!("Status: {}, Recipient: {:?}", status.status, status.to);
    Ok(())
}
```

### 3. Termii: SMS, Dynamic OTP Tokens & In-App Tokens

Standard SMS with automatic leading `+` sanitization for Nigerian aggregators, dynamic OTP PIN dispatch with placeholders, and zero-cost in-app tokens:

```rust
use afri_sms::{TermiiProvider, TermiiSendTokenRequest, TermiiGenerateTokenRequest, TermiiPinType, SmsError};

#[tokio::main]
async fn main() -> Result<(), SmsError> {
    let termii = TermiiProvider::builder()
        .api_key("termii-api-key")
        .sender_id("SecureApp")
        .channel("dnd") // Use DND to bypass carrier blocking on financial alerts
        .build()?;

    // 1. Standard SMS (automatically cleans '+234...' to '234...'):
    let res = termii.send_sms("+2348012345678", "Security code: 998811").await?;

    // 2. Dispatch dynamic OTP PIN via Termii Send Token API:
    let token = termii.send_token(TermiiSendTokenRequest {
        pin_type: TermiiPinType::Numeric,
        to: "2348012345678".to_string(),
        from: None,
        channel: Some("dnd".to_string()),
        pin_attempts: 3,
        pin_time_to_live: 10,
        pin_length: 6,
        pin_placeholder: "< 123456 >".to_string(),
        message_text: "Your PIN is < 123456 >".to_string(),
    }).await?;

    // 3. Generate In-App Token (no SMS charges):
    let in_app = termii.generate_in_app_token(TermiiGenerateTokenRequest {
        pin_type: TermiiPinType::Numeric,
        phone_number: "2348012345678".to_string(),
        pin_attempts: 3,
        pin_time_to_live: 5,
        pin_length: 6,
    }).await?;
    Ok(())
}
```

### 4. Message Central: CPaaS SMS & Verify Now V3 OTP

Multi-channel CPaaS SMS dispatch with automated international dialing code splitting, plus the full "Verify Now" OTP lifecycle (`send_otp` and `validate_otp`):

```rust
use afri_sms::{MessageCentralProvider, SmsError};

#[tokio::main]
async fn main() -> Result<(), SmsError> {
    let mc = MessageCentralProvider::builder()
        .auth_token("mc-auth-token")
        .customer_id("C-BA489D17CAE0402")
        .sender_id("BrandAlert")
        .default_country_code("234")
        .build()?;

    // 1. Dispatch SMS:
    let res = mc.send_sms("+2348012345678", "Your code is 482910").await?;

    // 2. Initiate OTP session via Verify Now V3:
    let otp = mc.send_otp("+2348012345678", Some("SMS")).await?;
    println!("Verification ID: {}", otp.verification_id);

    // 3. Validate user-submitted OTP code:
    let is_valid = mc.validate_otp(&otp.verification_id, "482910", Some("SMS")).await?;
    println!("Is valid: {}", is_valid);
    Ok(())
}
```

### 5. LLM Agent Tools & Function Calling (`afri_sms::tools`)

Strongly-typed function-calling JSON schemas and runtime dispatch execution for autonomous AI coding agents:

```rust
use afri_sms::{MultiSmsClient, SendarProvider};
use afri_sms::tools::{execute_tool, SendSmsToolInput, SendSmsToolOutput, ProviderChoice, send_sms_tool_definition};

#[tokio::main]
async fn main() {
    // 1. Obtain tool schema for LLM function calling registration:
    let tool_schema = send_sms_tool_definition();

    // 2. Execute incoming tool call:
    let client = MultiSmsClient::new(SendarProvider::new("sendar-api-key", "MyBrand"));
    let input = SendSmsToolInput::new("+2348012345678", "Your OTP is 123456")
        .with_provider(ProviderChoice::Sendar);

    let output = execute_tool(&client, input).await;
    println!("Success: {}, Delivered by: {:?}", output.success, output.delivered_by);
}
```
