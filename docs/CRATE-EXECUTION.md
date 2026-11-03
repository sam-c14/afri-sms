# EXECUTION.md: `afri-sms` Rust Crate Roadmap

This document serves as the strict engineering roadmap for the AI agent building the `afri-sms` open-source crate.

## Milestone 1: Initialization & Core Primitives
* [x] Initialize a Cargo Workspace with member crates `afri-sms` and `afri-sms-skills`.
* [x] Configure root and member `Cargo.toml` with workspace dependencies: `reqwest`, `serde`, `serde_json`, `tokio` (for tests), `async-trait`, `thiserror`, and `tracing`.
* [x] Define the unified `SmsError` enum in `crates/afri-sms/src/error.rs` using `thiserror`.
* [x] Define the `SmsProvider` async trait and `SmsResponse` struct in `crates/afri-sms/src/provider.rs`.

## Milestone 2: Provider Implementations
* [x] Implement `SendarProvider` in `crates/afri-sms/src/providers/sendar.rs`. Ensure it matches Sendar's specific JSON schema and authentication headers.
* [x] Implement `TermiiProvider` in `crates/afri-sms/src/providers/termii.rs`.
* [x] Implement `MessageCentralProvider` in `crates/afri-sms/src/providers/message_central.rs`.
* [x] Expose these via a cleanly structured module tree (`pub use providers::*;`).

## Milestone 3: The Fallback Engine & Builder
* [x] Implement the `MultiSmsClient` struct in `crates/afri-sms/src/client.rs`.
* [x] Implement the Builder pattern for `MultiSmsClient` allowing users to chain `.primary(provider)` and `.with_fallback(provider)`.
* [x] Implement the routing logic: `client.send_sms(to, msg)` should try the primary, catch `SmsError::NetworkError` or `SmsError::ProviderError(5xx)`, and automatically await the fallback if configured.
* [x] Add `tracing::info!` and `tracing::warn!` logs to document when a fallback occurs.

## Milestone 4: Testing & Documentation
* [x] Set up `wiremock` in `[dev-dependencies]` to mock HTTP responses.
* [x] Write unit tests for each provider ensuring payload serialization matches their respective docs.
* [x] Write an integration test proving the Fallback Engine successfully switches to the secondary provider when the primary mock server returns a 500 status code.
* [x] Write comprehensive documentation across member crates.