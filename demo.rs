//! # afri-sms Comprehensive Showcase Demo
//!
//! This demo illustrates the primary use-cases for all SMS & OTP providers
//! supported by `afri-sms`:
//!
//! 1. **Sendar (`SendarProvider`)**:
//!    - Fast JSON REST API for transactional alerts.
//!    - Real-time message delivery status lookup (`get_message_status`).
//!
//! 2. **Termii (`TermiiProvider`)**:
//!    - Standard SMS with automatic Nigerian phone formatting (leading `+` stripping).
//!    - Dynamic SMS OTP verification token dispatch (`send_token`).
//!    - In-app token generation without SMS transmission (`generate_in_app_token`).
//!
//! 3. **Message Central (`MessageCentralProvider`)**:
//!    - Global/African CPaaS SMS dispatch.
//!    - "Verify Now" OTP verification lifecycle (`send_otp` -> `validate_otp`).
//!
//! 4. **Multi-Provider Fallback Orchestration (`MultiSmsClient`)**:
//!    - Primary-to-secondary gateway failover (e.g. Sendar -> Termii).
//!    - Automatic recovery from carrier downtime, 5xx outages, and timeouts.
//!
//! 5. **LLM Autonomous Agent Tool Dispatch (`afri_sms::tools`)**:
//!    - Function-calling JSON schema generation for AI agents.
//!    - Strongly-typed tool invocation (`execute_tool`).
//!
//! ## Configuration
//! Populate the empty constants below with your live credentials, or supply them
//! as environment variables (e.g., `export SENDAR_API_KEY="..."`).

use std::env;
use std::error::Error;

use afri_sms::tools::{ProviderChoice, SendSmsToolInput, execute_tool, send_sms_tool_definition};
use afri_sms::{
    MessageCentralProvider, MultiSmsClient, SendarProvider, SmsError, SmsProvider,
    TermiiGenerateTokenRequest, TermiiPinType, TermiiProvider, TermiiSendTokenRequest,
};

// ============================================================================
// 1. PROVIDER CREDENTIAL CONSTANTS
//    (Set your API credentials here or via environment variables)
// ============================================================================

/// Sendar API Key (HTTP Header `API-KEY`)
const SENDAR_API_KEY: &str = "";

/// Approved Sendar Sender ID
const SENDAR_SENDER_ID: &str = "";

/// Sendar Wallet Type: "transactional", "promotional", or "otp"
const SENDAR_WALLET_TYPE: &str = "transactional";

/// Termii API Key
const TERMII_API_KEY: &str = "";

/// Approved Termii Sender ID (3 to 11 alphanumeric characters)
const TERMII_SENDER_ID: &str = "";

/// Termii Channel: "dnd" (for OTP/transactional bypassing DND) or "generic"
const TERMII_CHANNEL: &str = "dnd";

/// Message Central Customer ID (e.g. "C-BA489D17CAE0402")
const MESSAGE_CENTRAL_CUSTOMER_ID: &str = "";

/// Message Central Auth Token / JWT (HTTP Header `authToken`)
const MESSAGE_CENTRAL_AUTH_TOKEN: &str = "";

/// Approved Message Central Sender ID / Shortcode (Optional)
const MESSAGE_CENTRAL_SENDER_ID: &str = "";

/// Destination phone number for testing (E.164 format e.g. "+2348012345678")
const TEST_RECIPIENT: &str = "+2348012345678";

// ============================================================================
// Credential Helper
// ============================================================================

fn resolve_credential(constant_val: &str, env_var: &str) -> String {
    if !constant_val.trim().is_empty() {
        constant_val.trim().to_string()
    } else {
        env::var(env_var).unwrap_or_default().trim().to_string()
    }
}

// ============================================================================
// Use-Case 1: Sendar Provider
// ============================================================================

async fn demo_sendar(to: &str) -> Result<(), Box<dyn Error>> {
    println!("\n=======================================================");
    println!(" [USE-CASE 1] Sendar Provider (Transactional SMS & Status)");
    println!("=======================================================");

    let api_key = resolve_credential(SENDAR_API_KEY, "SENDAR_API_KEY");
    let sender_id = resolve_credential(SENDAR_SENDER_ID, "SENDAR_SENDER_ID");

    if api_key.is_empty() || sender_id.is_empty() {
        println!("ℹ️  Skipping live Sendar request (SENDAR_API_KEY / SENDAR_SENDER_ID not set).");
        println!("   Code architecture pattern:");
        println!(
            r#"   let provider = SendarProvider::builder()
       .api_key(api_key)
       .sender_id(sender_id)
       .wallet_type("{SENDAR_WALLET_TYPE}")
       .build()?;

   // 1. Send transactional SMS
   let response = provider.send_sms(to, "Your transfer of NGN 5,000 was successful.").await?;

   // 2. Query real-time delivery status
   let status = provider.get_message_status(&response.message_id).await?;"#
        );
        return Ok(());
    }

    // 1. Initialize SendarProvider with builder
    let sendar = SendarProvider::builder()
        .api_key(api_key)
        .sender_id(sender_id)
        .wallet_type(SENDAR_WALLET_TYPE)
        .build()?;

    println!(
        "✓ SendarProvider initialized [Sender: '{}', Wallet: '{}']",
        sendar.sender_id(),
        sendar.wallet_type()
    );

    // 2. Dispatch SMS
    println!("→ Dispatching transactional SMS to {}...", to);
    let response = sendar
        .send_sms(
            to,
            "Your transfer of NGN 5,000 was successful. Ref: TXN998811.",
        )
        .await?;

    println!(
        "✓ Delivered! Provider: {}, Message ID: {}",
        response.provider, response.message_id
    );

    // 3. Query Message Delivery Status
    println!(
        "→ Fetching real-time delivery status for message ID '{}'...",
        response.message_id
    );
    match sendar.get_message_status(&response.message_id).await {
        Ok(status) => {
            println!(
                "✓ Sendar Delivery Status: ID={}, status={}, to={:?}",
                status.message_id, status.status, status.to
            );
        }
        Err(err) => {
            println!(
                "⚠️ Status check returned: {} (status may still be queued)",
                err
            );
        }
    }

    Ok(())
}

// ============================================================================
// Use-Case 2: Termii Provider
// ============================================================================

async fn demo_termii(to: &str) -> Result<(), Box<dyn Error>> {
    let api_key = resolve_credential(TERMII_API_KEY, "TERMII_API_KEY");
    let sender_id = resolve_credential(TERMII_SENDER_ID, "TERMII_SENDER_ID");

    // 1. Initialize TermiiProvider with builder
    let termii = TermiiProvider::builder()
        .api_key(api_key)
        .sender_id(sender_id)
        .channel(TERMII_CHANNEL)
        .build()?;

    println!(
        "✓ TermiiProvider initialized [Sender: '{}', Channel: '{}']",
        termii.sender_id(),
        termii.channel()
    );

    let response = termii
        .send_sms(
            to,
            "Your security alert code is 998811. Do not share this with anyone.",
        )
        .await?;

    println!(
        "OTP Delivered! Provider: {}, Message ID: {}",
        response.provider, response.message_id
    );

    // 3. Dispatch Dynamic SMS Token via Termii OTP API
    println!("→ Dispatching dynamic OTP token via Termii Send Token API...");
    let token_req = TermiiSendTokenRequest {
        pin_type: TermiiPinType::Numeric,
        to: to.to_string(),
        from: None, // Uses configured sender ID
        channel: Some(TERMII_CHANNEL.to_string()),
        pin_attempts: 3,
        pin_time_to_live: 10,
        pin_length: 6,
        pin_placeholder: "< 123456 >".to_string(),
        message_text: "Your login PIN is < 123456 >. Valid for 10 minutes.".to_string(),
    };

    match termii.send_token(token_req).await {
        Ok(token_res) => {
            println!(
                "✓ Termii OTP Dispatched! Pin ID: {}, SMS Status: {:?}",
                token_res.pin_id, token_res.sms_status
            );
        }
        Err(err) => {
            println!("⚠️ Termii OTP token dispatch: {}", err);
        }
    }

    // 4. Generate In-App Token (for mobile apps or embedded validation)
    println!("→ Generating in-app token (zero SMS dispatch charges)...");
    let in_app_req = TermiiGenerateTokenRequest {
        pin_type: TermiiPinType::Numeric,
        phone_number: to.to_string(),
        pin_attempts: 3,
        pin_time_to_live: 5,
        pin_length: 6,
    };

    match termii.generate_in_app_token(in_app_req).await {
        Ok(in_app_res) => {
            println!(
                "✓ Termii In-App Token Generated! Pin ID: {}, OTP: {}",
                in_app_res.pin_id, in_app_res.otp
            );
        }
        Err(err) => {
            println!("⚠️ Termii in-app token generation: {}", err);
        }
    }

    Ok(())
}

// ============================================================================
// Use-Case 3: Message Central Provider
// ============================================================================

async fn demo_message_central(to: &str) -> Result<(), Box<dyn Error>> {
    let customer_id =
        resolve_credential(MESSAGE_CENTRAL_CUSTOMER_ID, "MESSAGE_CENTRAL_CUSTOMER_ID");
    let auth_token = resolve_credential(MESSAGE_CENTRAL_AUTH_TOKEN, "MESSAGE_CENTRAL_AUTH_TOKEN");
    let sender_id = resolve_credential(MESSAGE_CENTRAL_SENDER_ID, "MESSAGE_CENTRAL_SENDER_ID");

    // 1. Initialize MessageCentralProvider with builder
    let mut builder = MessageCentralProvider::builder().auth_token(auth_token);

    if !customer_id.is_empty() {
        builder = builder.customer_id(customer_id);
    }
    if !sender_id.is_empty() {
        builder = builder.sender_id(sender_id);
    }

    let mc = builder.default_country_code("234").build()?;
    let response = mc
        .send_sms(to, "Your verification code is 482910. Valid for 5 minutes.")
        .await?;

    println!(
        "Delivered! Provider: {}, Message ID: {}",
        response.provider, response.message_id
    );

    // 2. Verify Now V3: Send OTP
    match mc.send_otp(to, Some("SMS")).await {
        Ok(otp_res) => {
            println!(
                "  Response Code: {:?}, Mobile: {:?}",
                otp_res.response_code, otp_res.mobile_number
            );

            // 3. Validate OTP (Sample demonstration call)
            // Assuming the user received the OTP "482910" and is submits that.
            match mc
                .validate_otp(&otp_res.verification_id, "482910", Some("SMS"))
                .await
            {
                Ok(valid) => println!("OTP Verification Result: valid = {}", valid),
                Err(err) => println!("OTP validation response: {}", err),
            }
        }
        Err(err) => {
            println!("Message Central OTP dispatch: {}", err);
        }
    }

    Ok(())
}

async fn demo_multi_client_fallback(to: &str) -> Result<(), Box<dyn Error>> {
    let sendar_key = resolve_credential(SENDAR_API_KEY, "SENDAR_API_KEY");
    let sendar_sender = resolve_credential(SENDAR_SENDER_ID, "SENDAR_SENDER_ID");
    let termii_key = resolve_credential(TERMII_API_KEY, "TERMII_API_KEY");
    let termii_sender = resolve_credential(TERMII_SENDER_ID, "TERMII_SENDER_ID");

    println!("MultiSmsClient architecture pattern:");
    println!(" - Primary Gateway:   Sendar (fast JSON REST)");
    println!(" - Fallback Gateway:  Termii (reliable carrier DND routing)");
    println!(
        " - Logic: Retries on 5xx / connection timeout; Aborts immediately on 400 Bad Request."
    );

    let primary = SendarProvider::new(
        if sendar_key.is_empty() {
            "placeholder-sendar-key"
        } else {
            &sendar_key
        },
        if sendar_sender.is_empty() {
            "PrimarySender"
        } else {
            &sendar_sender
        },
    );

    let fallback = TermiiProvider::new(
        if termii_key.is_empty() {
            "placeholder-termii-key"
        } else {
            &termii_key
        },
        if termii_sender.is_empty() {
            "FallbackSender"
        } else {
            &termii_sender
        },
    );

    let client = MultiSmsClient::builder()
        .primary(primary)
        .with_fallback(fallback)
        .build()?;

    println!(
        "✓ MultiSmsClient configured: Primary='{}', Fallback='{:?}'",
        client.primary_name(),
        client.fallback_name()
    );

    if sendar_key.is_empty() && termii_key.is_empty() {
        println!("ℹ️  Set live credentials to test automated failover end-to-end.");
    } else {
        println!("→ Attempting dispatch with secondary fallback redundancy...");
        match client
            .send_sms(to, "Security Notice: Login attempt from Lagos, NG.")
            .await
        {
            Ok(response) => {
                println!(
                    "✓ Message successfully dispatched via provider '{}' (Message ID: {})",
                    response.provider, response.message_id
                );
            }
            Err(SmsError::AllProvidersFailed { primary, fallback }) => {
                println!("⚠️ Both gateways failed:");
                println!("   Primary error:  {}", primary);
                if let Some(fb) = fallback {
                    println!("   Fallback error: {}", fb);
                }
            }
            Err(err) => {
                println!("⚠️ Non-retriable error (failover avoided): {}", err);
            }
        }
    }

    Ok(())
}

// ============================================================================
// Use-Case 5: LLM Agent Tools & Function Calling
// ============================================================================

async fn demo_agent_tool_execution(to: &str) -> Result<(), Box<dyn Error>> {
    println!("\n=======================================================");
    println!(" [USE-CASE 5] LLM Agent Tool Dispatching (afri_sms::tools)");
    println!("=======================================================");

    // 1. Tool JSON Schema Generation (for registering with OpenAI/Gemini/Anthropic/Rig)
    let schema = send_sms_tool_definition();
    println!("✓ Generated Tool Definition for Agent function calling:");
    println!("  Tool name: {}", schema["name"]);
    println!("  Tool description: {}", schema["description"]);
    println!("  Required fields: {:?}", schema["parameters"]["required"]);

    // 2. Simulating an incoming Agent function-call input (parsed from LLM JSON)
    let agent_input = SendSmsToolInput::new(to, "Delivery update: Your rider is at your gate.")
        .with_provider(ProviderChoice::Sendar);

    println!("\n→ Simulated LLM Tool Call Arguments:");
    println!("  Target:   {}", agent_input.to);
    println!("  Message:  {}", agent_input.message);
    println!("  Provider: {:?}", agent_input.provider);

    // 3. Executing the tool using MultiSmsClient
    let primary = SendarProvider::new("demo-api-key", "TestBrand");
    let client = MultiSmsClient::new(primary);

    println!("→ Executing agent tool via execute_tool(&client, agent_input)...");
    let tool_output = execute_tool(&client, agent_input).await;

    println!("✓ Tool Execution Output (ready to return to LLM context):");
    println!("  success:      {}", tool_output.success);
    println!("  message_id:   {:?}", tool_output.message_id);
    println!("  delivered_by: {:?}", tool_output.delivered_by);
    println!("  details:      {}", tool_output.details);

    Ok(())
}

// ============================================================================
// Main Entrypoint
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=======================================================");
    println!("       afri-sms Multi-Provider Showcase & Demos        ");
    println!("=======================================================");

    let recipient = if TEST_RECIPIENT != "+2348012345678" {
        TEST_RECIPIENT
    } else {
        &env::var("TEST_PHONE_NUMBER").unwrap_or_else(|_| TEST_RECIPIENT.to_string())
    };

    println!("Target Test Recipient: {}\n", recipient);

    // 1. Sendar
    demo_sendar(recipient).await?;

    // 2. Termii
    demo_termii(recipient).await?;

    // 3. Message Central
    demo_message_central(recipient).await?;

    // 4. MultiSmsClient Fallback Engine
    demo_multi_client_fallback(recipient).await?;

    // 5. LLM Agent Tools
    demo_agent_tool_execution(recipient).await?;

    println!("\n=======================================================");
    println!("             All Demo Scenarios Completed              ");
    println!("=======================================================");

    Ok(())
}
