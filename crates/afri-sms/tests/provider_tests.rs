use afri_sms::{MessageCentralProvider, SendarProvider, SmsError, SmsProvider, TermiiProvider};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_sendar_successful_dispatch() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "status": "success",
        "message_id": "sendar_msg_001",
        "cost": "2.50",
        "currency": "NGN"
    });

    Mock::given(method("POST"))
        .and(path("/v2/sms/send"))
        .and(header("Authorization", "Bearer sendar-secret-token"))
        .and(header("Content-Type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = SendarProvider::builder()
        .api_key("sendar-secret-token")
        .sender_id("FintechAlert")
        .base_url(mock_server.uri())
        .route("transactional")
        .build()
        .expect("builder succeeds");

    let res = provider
        .send_sms("+2348012345678", "Your OTP is 738291")
        .await
        .expect("send_sms should succeed");

    assert_eq!(res.message_id, "sendar_msg_001");
    assert_eq!(res.status, "success");
    assert_eq!(res.provider, "sendar");
    assert!(res.raw_response.is_some());
}

#[tokio::test]
async fn test_sendar_invalid_phone_number_error_mapping() {
    let mock_server = MockServer::start().await;

    let error_body = serde_json::json!({
        "status": "error",
        "message": "Invalid recipient phone number format"
    });

    Mock::given(method("POST"))
        .and(path("/v2/sms/send"))
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
        .and(path("/v2/sms/send"))
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
