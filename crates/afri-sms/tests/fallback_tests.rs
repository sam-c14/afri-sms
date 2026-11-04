use afri_sms::{MultiSmsClient, SendarProvider, SmsError, TermiiProvider};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_fallback_engine_switches_to_secondary_on_primary_500() {
    // 1. Set up primary mock server returning HTTP 500 Internal Server Error
    let primary_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sms/send"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_string("Internal Gateway Error: Telecom provider unreachable"),
        )
        .expect(1)
        .mount(&primary_mock)
        .await;

    // 2. Set up fallback mock server returning HTTP 200 OK
    let fallback_mock = MockServer::start().await;
    let fallback_json = serde_json::json!({
        "code": "ok",
        "message_id": "fallback_tx_998877",
        "message": "Successfully Delivered via Secondary Route"
    });
    Mock::given(method("POST"))
        .and(path("/api/sms/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&fallback_json))
        .expect(1)
        .mount(&fallback_mock)
        .await;

    // 3. Configure the MultiSmsClient
    let primary = SendarProvider::builder()
        .api_key("sk_live_test_sendar_key")
        .sender_id("BankAlert")
        .base_url(primary_mock.uri())
        .build()
        .expect("primary builds");

    let fallback = TermiiProvider::builder()
        .api_key("termii_key")
        .sender_id("BankAlert")
        .base_url(fallback_mock.uri())
        .build()
        .expect("fallback builds");

    let client = MultiSmsClient::builder()
        .primary(primary)
        .with_fallback(fallback)
        .build()
        .expect("client builds");

    // 4. Dispatch SMS and assert fallback delivery
    let response = client
        .send_sms("+2348012345678", "Your OTP is 112233")
        .await
        .expect("should successfully deliver through fallback");

    assert_eq!(response.provider, "termii");
    assert_eq!(response.message_id, "fallback_tx_998877");
    assert_eq!(response.status, "ok");
}

#[tokio::test]
async fn test_fallback_engine_aborts_without_retry_on_client_400_error() {
    let primary_mock = MockServer::start().await;
    let fallback_mock = MockServer::start().await;

    // Primary returns 400 Bad Request
    let bad_request_json = serde_json::json!({
        "status": false,
        "message": "Invalid recipient phone number format"
    });
    Mock::given(method("POST"))
        .and(path("/sms/send"))
        .respond_with(ResponseTemplate::new(400).set_body_json(&bad_request_json))
        .expect(1)
        .mount(&primary_mock)
        .await;

    // Fallback should NEVER be invoked!
    Mock::given(method("POST"))
        .and(path("/api/sms/send"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&fallback_mock)
        .await;

    let primary = SendarProvider::builder()
        .api_key("sk_live_sendar_key")
        .sender_id("Alert")
        .base_url(primary_mock.uri())
        .build()
        .expect("primary builds");

    let fallback = TermiiProvider::builder()
        .api_key("termii_key")
        .sender_id("Alert")
        .base_url(fallback_mock.uri())
        .build()
        .expect("fallback builds");

    let client = MultiSmsClient::builder()
        .primary(primary)
        .with_fallback(fallback)
        .build()
        .expect("client builds");

    let err = client
        .send_sms("+111", "Hello")
        .await
        .expect_err("should return error without fallback");

    assert!(matches!(err, SmsError::InvalidPhoneNumber { .. }));
}

#[tokio::test]
async fn test_fallback_engine_returns_all_providers_failed_when_both_fail() {
    let primary_mock = MockServer::start().await;
    let fallback_mock = MockServer::start().await;

    // Both servers return 500
    Mock::given(method("POST"))
        .and(path("/sms/send"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Primary Down"))
        .expect(1)
        .mount(&primary_mock)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/sms/send"))
        .respond_with(ResponseTemplate::new(503).set_body_string("Fallback Unavailable"))
        .expect(1)
        .mount(&fallback_mock)
        .await;

    let primary = SendarProvider::builder()
        .api_key("k1")
        .sender_id("A")
        .base_url(primary_mock.uri())
        .build()
        .expect("builds");

    let fallback = TermiiProvider::builder()
        .api_key("k2")
        .sender_id("A")
        .base_url(fallback_mock.uri())
        .build()
        .expect("builds");

    let client = MultiSmsClient::builder()
        .primary(primary)
        .with_fallback(fallback)
        .build()
        .expect("builds");

    let err = client
        .send_sms("+2348000000000", "Hello")
        .await
        .expect_err("should return AllProvidersFailed");

    match err {
        SmsError::AllProvidersFailed { primary, fallback } => {
            assert!(matches!(
                *primary,
                SmsError::ProviderError {
                    status_code: 500,
                    ..
                }
            ));
            assert!(matches!(
                fallback.map(|f| *f),
                Some(SmsError::ProviderError {
                    status_code: 503,
                    ..
                })
            ));
        }
        other => panic!("Expected AllProvidersFailed, got {other:?}"),
    }
}
