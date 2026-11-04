# afri-sms

[![Crates.io](https://img.shields.io/crates/v/afri-sms.svg)](https://crates.io/crates/afri-sms)
[![Documentation](https://docs.rs/afri-sms/badge.svg)](https://docs.rs/afri-sms)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../Cargo.toml)

The core implementation crate for `afri-sms`.

Provides asynchronous adapters for African telecom gateways (Sendar, Termii, Message Central) alongside an automated secondary fallback routing engine.

## Installation

```toml
[dependencies]
afri-sms = "0.1.1"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

## Quick Start

```rust
use afri_sms::{MultiSmsClient, SendarProvider, TermiiProvider, SmsError};

#[tokio::main]
async fn main() -> Result<(), SmsError> {
    let primary = SendarProvider::builder()
        .api_key("sendar_live_key")
        .sender_id("BankAlert")
        .route("transactional")
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

    println!("Delivered by {}: {}", response.provider, response.message_id);
    Ok(())
}
```
