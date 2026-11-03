# Product Requirements Document: `afri-sms` (Rust Crate)

## 1. Executive Summary
`afri-sms` is an asynchronous, high-performance Rust crate designed to provide a unified, developer-friendly interface for sending SMS and OTPs via African telecom providers (e.g., Sendar, Message Central, Termii, BulkSMSNigeria). It utilizes the Adapter (Facade) pattern to abstract away provider-specific HTTP quirks, offering a strongly-typed, unified API with built-in automated fallback routing to guarantee high delivery rates.

## 2. Core Features
* **Unified Provider Trait:** A single `async_trait` that standardizes the payload and execution logic regardless of the underlying API (JSON vs. Form-Data, Bearer vs. Query Params).
* **Builder Pattern Configuration:** A clean, ergonomic API for developers to initialize clients, set API keys, and configure sender IDs.
* **Automated Fallback Engine:** The ability to register a primary and a secondary provider. If the primary provider times out or returns a 5xx error, the crate automatically serializes the payload for the secondary provider and retries.
* **Zero-Boilerplate Errors:** A unified `SmsError` enum (powered by `thiserror`) that normalizes errors across different platforms (e.g., mapping a Termii 400 and a Sendar 400 to a single `SmsError::InvalidPhoneNumber`).

## 3. Architecture & Interfaces

### 3.1 Dependencies
* `reqwest` (with `json` and `rustls-tls` features) - Async HTTP client.
* `serde`, `serde_json` - Serialization/Deserialization.
* `async-trait` - To allow async functions in the provider traits.
* `thiserror` - For idiomatic Rust error handling.
* `tracing` - For non-blocking, structured logging of requests and fallbacks.

### 3.2 The Core Trait
```rust
#[async_trait]
pub trait SmsProvider: Send + Sync {
    async fn send_sms(&self, to: &str, message: &str) -> Result<SmsResponse, SmsError>;
    fn name(&self) -> &'static str;
}
```

### 3.3 The Multi-Provider Client
```rust
pub struct MultiSmsClient {
    primary: Box<dyn SmsProvider>,
    fallback: Option<Box<dyn SmsProvider>>,
}
```

## 4. Supported Providers (V1 Scope)
1. **Sendar:** Modern JSON REST API.
2. **Message Central:** Modern JSON REST API with WhatsApp fallback support.
3. **Termii:** The legacy industry standard (JSON based).
4. **BulkSMSNigeria:** Form-data / Query-param based (good for testing diverse payload structures).

## 5. Security & Performance
* Secrets (API Keys) must be marked with `#[serde(skip)]` or `secrecy` crate equivalents in debug logs to prevent leaking in consumer apps.
* HTTP clients (`reqwest::Client`) must be reused and passed by reference/Arc internally to prevent socket exhaustion.