---
name: afri-sms
description: Unified SMS & OTP adapter for African telecom providers (Sendar, Termii, Message Central) with automated fallback routing. Use when dispatching SMS messages, sending OTP verification codes, integrating African payment/fintech notifications, or configuring telecom gateways with automated failover.
---

# SKILL: African SMS & OTP Integration (`afri-sms`)

## Overview
`afri-sms` provides a unified, high-performance interface for dispatching SMS and OTP verification messages across African telecom aggregators. It includes an automated secondary fallback routing engine to guarantee high delivery rates even when primary telecom gateways experience 5xx server downtime or network timeouts.

## Supported Providers
* **Sendar (`sendar`):** Modern JSON REST API. Authenticated via `Authorization: Bearer <key>`. Ideal for high-throughput transactional alerts.
* **Termii (`termii`):** Industry standard in Nigeria. Authenticated via in-body `api_key`. Requires numbers formatted without leading `+` (e.g. `2348012345678`). Supports DND bypass routing.
* **Message Central (`message_central`):** Multi-channel CPaaS gateway with OTP verification and WhatsApp routing capabilities. Authenticated via `authToken` header.

## Best Practices for African Telecoms
1. **Recipient Formatting:** Always sanitize phone numbers to full international format (e.g., `+234...` for Nigeria, `+254...` for Kenya, `+233...` for Ghana).
2. **Fallback Strategy:**
   - **Transient Errors (HTTP 5xx, Network Timeout):** Automatically failover to secondary provider.
   - **Deterministic Client Errors (HTTP 400, Invalid Phone, Unauthorized):** Abort immediately without retrying to prevent wasting API balance.
3. **Connection Pooling:** Reuse `reqwest::Client` connection pools across requests to eliminate TLS handshake overhead.
4. **Credential Security:** Never log API keys or bearer tokens in cleartext.

## Rust Implementation Example

```rust
use afri_sms::{MultiSmsClient, SendarProvider, TermiiProvider, SmsError};

#[tokio::main]
async fn main() -> Result<(), SmsError> {
    let primary = SendarProvider::builder()
        .api_key(std::env::var("SENDAR_API_KEY").map_err(|e| SmsError::ConfigurationError(e.to_string()))?)
        .sender_id("BankAlert")
        .route("transactional")
        .build()?;

    let fallback = TermiiProvider::builder()
        .api_key(std::env::var("TERMII_API_KEY").map_err(|e| SmsError::ConfigurationError(e.to_string()))?)
        .sender_id("BankAlert")
        .channel("dnd")
        .build()?;

    let client = MultiSmsClient::builder()
        .primary(primary)
        .with_fallback(fallback)
        .build()?;

    let response = client
        .send_sms("+2348012345678", "Your login OTP is 948210")
        .await?;

    println!("Delivered via {} [Message ID: {}]", response.provider, response.message_id);
    Ok(())
}
```
