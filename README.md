# afri-sms

[![Crates.io](https://img.shields.io/crates/v/afri-sms.svg)](https://crates.io/crates/afri-sms)
[![Documentation](https://docs.rs/afri-sms/badge.svg)](https://docs.rs/afri-sms)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](Cargo.toml)

A unified, highly-optimized SMS and OTP adapter crate for African telecom providers with automated fallback routing.

## Overview

African telecom and SMS aggregators differ wildly in API conventions, authentication methods, payload structures, and delivery reliability. `afri-sms` abstracts these nuances behind a clean, idiomatic Rust Adapter pattern.

### Key Features

* **Unified Provider Trait:** Standardized `#[async_trait] pub trait SmsProvider` for consistent API interaction across providers.
* **Automated Fallback Engine:** Configure primary and secondary gateways to automatically reroute traffic upon server errors (5xx) or network timeouts.
* **Zero-Boilerplate Errors:** Comprehensive error classification via `thiserror` (`SmsError`).
* **High Performance & Resource Efficiency:** Zero-cost abstractions, zero unneeded allocations, and shared `reqwest::Client` connection pooling.

## Status

Currently completing **Milestone 1: Initialization & Core Primitives**.
