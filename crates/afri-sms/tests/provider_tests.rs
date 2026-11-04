use afri_sms::{
    MessageCentralProvider, SendarProvider, SmsError, SmsProvider, TermiiGenerateTokenRequest,
    TermiiPinType, TermiiProvider, TermiiSendTokenRequest,
};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_sendar_successful_dispatch() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "status": true,
        "total": 1,
        "queued": 1,
        "batch_id": "sendar_batch_001",
        "key_type": "live",
        "total_cost": 2.50,
        "data": [
            {
                "number": "2348012345678",
                "message_id": "sendar_msg_001",
                "status": "queued"
            }
        ]
    });

    Mock::given(method("POST"))
        .and(path("/sms/send"))
        .and(header("api-key", "sendar-secret-token"))
        .and(header("Content-Type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = SendarProvider::builder()
        .api_key("sendar-secret-token")
        .sender_id("FintechAlert")
        .base_url(mock_server.uri())
        .wallet_type("transactional")
        .build()
        .expect("builder succeeds");

    let res = provider
        .send_sms("+2348012345678", "Your OTP is 738291")
        .await
        .expect("send_sms should succeed");

    assert_eq!(res.message_id, "sendar_msg_001");
    assert_eq!(res.status, "queued");
    assert_eq!(res.provider, "sendar");
    assert!(res.raw_response.is_some());
}

#[tokio::test]
async fn test_sendar_delivery_status_check() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "message_id": "sendar_msg_001",
        "status": "delivered",
        "to": "2348012345678",
        "sender_id": "FintechAlert",
        "cost_per_unit": 6.5,
        "billing_event_id": "Txn_batch_1234",
        "gateway_message_id": "00000010-1234",
        "created_at": "2026-04-13T17:44:47.911324Z",
        "updated_at": "2026-04-13T17:44:47.911324Z"
    });

    Mock::given(method("GET"))
        .and(path("/get/sms/sendar_msg_001"))
        .and(header("api-key", "sendar-secret-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = SendarProvider::builder()
        .api_key("sendar-secret-token")
        .sender_id("FintechAlert")
        .base_url(mock_server.uri())
        .build()
        .expect("builder succeeds");

    let status = provider
        .get_message_status("sendar_msg_001")
        .await
        .expect("get_message_status succeeds");

    assert_eq!(status.message_id, "sendar_msg_001");
    assert_eq!(status.status, "delivered");
    assert_eq!(status.to.as_deref(), Some("2348012345678"));
    assert_eq!(status.cost_per_unit, Some(6.5));
}

#[tokio::test]
async fn test_sendar_invalid_phone_number_error_mapping() {
    let mock_server = MockServer::start().await;

    let error_body = serde_json::json!({
        "status": false,
        "message": "Invalid recipient phone number format"
    });

    Mock::given(method("POST"))
        .and(path("/sms/send"))
        .respond_with(ResponseTemplate::new(400).set_body_json(&error_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = SendarProvider::builder()
        .api_key("token")
        .sender_id("Alert")
        .base_url(mock_server.uri())
        .build()
        .expect("builder succeeds");

    let err = provider
        .send_sms("+9999999999", "Test")
        .await
        .expect_err("should return error");

    match err {
        SmsError::InvalidPhoneNumber { phone_number, .. } => {
            assert_eq!(phone_number, "+9999999999");
        }
        other => panic!("Expected InvalidPhoneNumber, got {other:?}"),
    }
}

#[tokio::test]
async fn test_sendar_authentication_failure_mapping() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/sms/send"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized API Key"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = SendarProvider::builder()
        .api_key("bad-token")
        .sender_id("Alert")
        .base_url(mock_server.uri())
        .build()
        .expect("builder succeeds");

    let err = provider
        .send_sms("+2348012345678", "Test")
        .await
        .expect_err("should fail with 401");

    assert!(matches!(err, SmsError::AuthenticationFailed(_)));
}

#[tokio::test]
async fn test_termii_successful_dispatch() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "code": "ok",
        "message_id": "3017544054459083819856413",
        "message": "Successfully Sent",
        "balance": 1500.50,
        "user": "Fintech Dev"
    });

    Mock::given(method("POST"))
        .and(path("/api/sms/send"))
        .and(header("Content-Type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = TermiiProvider::builder()
        .api_key("termii_api_secret_key")
        .sender_id("BrandAlert")
        .channel("dnd")
        .base_url(mock_server.uri())
        .build()
        .expect("builder succeeds");

    let res = provider
        .send_sms("+2348012345678", "Your login code is 123456")
        .await
        .expect("termii send should succeed");

    assert_eq!(res.message_id, "3017544054459083819856413");
    assert_eq!(res.status, "ok");
    assert_eq!(res.provider, "termii");
}

#[tokio::test]
async fn test_termii_send_token_dispatch() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "smsStatus": "Message Sent",
        "phone_number": "2347065250817",
        "pinId": "b1f2242a-44c5-4eed-94bc-8d37b67ef219",
        "status": "200"
    });

    Mock::given(method("POST"))
        .and(path("/api/sms/otp/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = TermiiProvider::builder()
        .api_key("termii_key")
        .sender_id("QwikWallet")
        .base_url(mock_server.uri())
        .build()
        .expect("builder succeeds");

    let req = TermiiSendTokenRequest {
        pin_type: TermiiPinType::Numeric,
        to: "+2347065250817".to_string(),
        from: None,
        channel: Some("generic".to_string()),
        pin_attempts: 3,
        pin_time_to_live: 5,
        pin_length: 6,
        pin_placeholder: "< 123456 >".to_string(),
        message_text: "Your code is < 123456 >".to_string(),
    };

    let res = provider.send_token(req).await.expect("send_token succeeds");
    assert_eq!(res.pin_id, "b1f2242a-44c5-4eed-94bc-8d37b67ef219");
    assert_eq!(res.sms_status.as_deref(), Some("Message Sent"));
}

#[tokio::test]
async fn test_termii_generate_in_app_token() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "phone_number": "2347065250817",
        "otp": "5635",
        "pin_id": "1f6294bc-8db4-48ee-864b-45cf9e3493d9"
    });

    Mock::given(method("POST"))
        .and(path("/api/sms/otp/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = TermiiProvider::builder()
        .api_key("termii_key")
        .sender_id("Brand")
        .base_url(mock_server.uri())
        .build()
        .expect("builder succeeds");

    let req = TermiiGenerateTokenRequest {
        pin_type: TermiiPinType::Numeric,
        phone_number: "+2347065250817".to_string(),
        pin_attempts: 3,
        pin_time_to_live: 5,
        pin_length: 4,
    };

    let res = provider
        .generate_in_app_token(req)
        .await
        .expect("generate_in_app_token succeeds");
    assert_eq!(res.otp, "5635");
    assert_eq!(res.pin_id, "1f6294bc-8db4-48ee-864b-45cf9e3493d9");
}

#[tokio::test]
async fn test_termii_insufficient_balance_in_body() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "message": "Insufficient balance to send message"
    });

    Mock::given(method("POST"))
        .and(path("/api/sms/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = TermiiProvider::builder()
        .api_key("key")
        .sender_id("ID")
        .base_url(mock_server.uri())
        .build()
        .expect("builder succeeds");

    let err = provider
        .send_sms("+2348012345678", "Test msg")
        .await
        .expect_err("should detect insufficient balance");

    assert!(matches!(err, SmsError::InsufficientBalance(_)));
}

#[tokio::test]
async fn test_message_central_successful_dispatch() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "responseCode": 200,
        "message": "SUCCESS",
        "data": {
            "verificationId": "v-7890",
            "transactionId": "tx-1c9c56ec",
            "mobileNumber": "8012345678"
        }
    });

    Mock::given(method("POST"))
        .and(path("/verification/v3/send"))
        .and(header("authtoken", "jwt-auth-token-xyz"))
        .and(query_param("countryCode", "234"))
        .and(query_param("mobileNumber", "8012345678"))
        .and(query_param("flowType", "SMS"))
        .and(query_param("senderId", "BrandID"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = MessageCentralProvider::builder()
        .auth_token("jwt-auth-token-xyz")
        .sender_id("BrandID")
        .default_country_code("234")
        .base_url(mock_server.uri())
        .build()
        .expect("builder succeeds");

    let res = provider
        .send_sms("+2348012345678", "Your code is 654321")
        .await
        .expect("mc send should succeed");

    assert_eq!(res.message_id, "tx-1c9c56ec");
    assert_eq!(res.status, "SUCCESS");
    assert_eq!(res.provider, "message_central");
}

#[tokio::test]
async fn test_message_central_send_and_validate_otp() {
    let mock_server = MockServer::start().await;

    let send_response = serde_json::json!({
        "responseCode": 200,
        "message": "SUCCESS",
        "data": {
            "verificationId": "v-9988-7766",
            "mobileNumber": "8123456789",
            "responseCode": "200",
            "errorMessage": null,
            "timeout": "60",
            "transactionId": "tx-otp-001"
        }
    });

    Mock::given(method("POST"))
        .and(path("/verification/v3/send"))
        .and(header("authtoken", "valid-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&send_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    let validate_response = serde_json::json!({
        "responseCode": 200,
        "message": "SUCCESS",
        "data": {
            "verificationId": "v-9988-7766"
        }
    });

    Mock::given(method("POST"))
        .and(path("/verification/v3/validateOtp"))
        .and(header("authtoken", "valid-token"))
        .and(query_param("verificationId", "v-9988-7766"))
        .and(query_param("code", "1234"))
        .and(query_param("flowType", "SMS"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&validate_response))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = MessageCentralProvider::builder()
        .auth_token("valid-token")
        .sender_id("Brand")
        .base_url(mock_server.uri())
        .build()
        .expect("builder succeeds");

    let send_res = provider
        .send_otp("+2348123456789", None)
        .await
        .expect("send_otp succeeds");
    assert_eq!(send_res.verification_id, "v-9988-7766");

    let is_valid = provider
        .validate_otp("v-9988-7766", "1234", None)
        .await
        .expect("validate succeeds");
    assert!(is_valid);
}
