# EXECUTION.md: `afri-sms` Rust Crate Roadmap

This document serves as the strict engineering roadmap for the AI agent building the `afri-sms` open-source crate.

## Milestone 1: Initialization & Core Primitives
* [ ] Initialize a new Rust library crate (`cargo new afri-sms --lib`).
* [ ] Configure `Cargo.toml` with necessary dependencies: `reqwest`, `serde`, `serde_json`, `tokio` (for tests), `async-trait`, `thiserror`, and `tracing`.
* [ ] Define the unified `SmsError` enum in `src/error.rs` using `thiserror`.
* [ ] Define the `SmsProvider` async trait and `SmsResponse` struct in `src/provider.rs`.

## Milestone 2: Provider Implementations
* [ ] Implement `SendarProvider` in `src/providers/sendar.rs`. Ensure it matches Sendar's specific JSON schema and authentication headers.
* [ ] Implement `TermiiProvider` in `src/providers/termii.rs`.
* [ ] Implement `MessageCentralProvider` in `src/providers/message_central.rs`.
* [ ] Expose these via a cleanly structured module tree (`pub use providers::*;`).

## Milestone 3: The Fallback Engine & Builder
* [ ] Implement the `MultiSmsClient` struct in `src/client.rs`.
* [ ] Implement the Builder pattern for `MultiSmsClient` allowing users to chain `.primary(provider)` and `.with_fallback(provider)`.
* [ ] Implement the routing logic: `client.send_sms(to, msg)` should try the primary, catch `SmsError::NetworkError` or `SmsError::ProviderError(5xx)`, and automatically await the fallback if configured.
* [ ] Add `tracing::info!` and `tracing::warn!` logs to document when a fallback occurs.

## Milestone 4: Testing & Documentation
* [ ] Set up `wiremock` or `mockito` in the `[dev-dependencies]` to mock HTTP responses.
* [ ] Write unit tests for each provider ensuring payload serialization matches their respective docs.
* [ ] Write an integration test proving the Fallback Engine successfully switches to the secondary provider when the primary mock server returns a 500 status code.
* [ ] Write a comprehensive `README.md` with usage examples, installation instructions, and contribution guidelines.