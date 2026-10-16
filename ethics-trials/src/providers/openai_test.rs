use super::{parse_completion_body, truncate};
use crate::providers::types::raw_body_of;

#[test]
fn truncate_respects_utf8_boundaries() {
    let value = "Aé🙂Z";

    let truncated = truncate(value, 4);

    assert_eq!(truncated, "Aé...");
}

#[test]
fn truncate_returns_ellipsis_when_limit_splits_first_character() {
    let truncated = truncate("🙂rest", 1);

    assert_eq!(truncated, "...");
}

#[test]
fn null_content_is_a_successful_response_with_logging_fields() {
    let body = r#"{
        "choices": [{
            "message": {"role": "assistant", "content": null},
            "finish_reason": "content_filter",
            "native_finish_reason": "SAFETY"
        }],
        "usage": {"prompt_tokens": 12, "completion_tokens": 0}
    }"#;

    let response = parse_completion_body(body, 7).unwrap();

    assert_eq!(response.content, "");
    assert!(response.content_null);
    assert_eq!(response.finish_reason.as_deref(), Some("content_filter"));
    assert_eq!(response.native_finish_reason.as_deref(), Some("SAFETY"));
    assert_eq!(response.raw_body.as_deref(), Some(body));
    assert_eq!(response.tokens_in, Some(12));
    assert_eq!(response.latency_ms, 7);
}

#[test]
fn missing_content_key_is_treated_as_null() {
    let body = r#"{"choices": [{"message": {"role": "assistant"}, "finish_reason": "length"}]}"#;

    let response = parse_completion_body(body, 0).unwrap();

    assert!(response.content_null);
    assert_eq!(response.finish_reason.as_deref(), Some("length"));
    assert!(response.raw_body.is_some());
}

#[test]
fn refusal_and_reasoning_are_captured() {
    let body = r#"{
        "choices": [{
            "message": {
                "content": "",
                "refusal": "I can't help with that.",
                "reasoning": "thinking it over"
            },
            "finish_reason": "stop"
        }]
    }"#;

    let response = parse_completion_body(body, 0).unwrap();

    assert_eq!(response.refusal.as_deref(), Some("I can't help with that."));
    assert_eq!(response.reasoning.as_deref(), Some("thinking it over"));
    assert!(!response.content_null);
    // Empty content is anomalous even with finish_reason "stop".
    assert!(response.raw_body.is_some());
}

#[test]
fn reasoning_content_is_used_when_reasoning_is_absent() {
    let body = r#"{"choices": [{"message": {"content": "ok", "reasoning_content": "chain"}, "finish_reason": "stop"}]}"#;

    let response = parse_completion_body(body, 0).unwrap();

    assert_eq!(response.reasoning.as_deref(), Some("chain"));
}

#[test]
fn normal_stop_response_does_not_keep_raw_body() {
    let body = r#"{"choices": [{"message": {"content": "hello", "refusal": null}, "finish_reason": "stop"}]}"#;

    let response = parse_completion_body(body, 0).unwrap();

    assert_eq!(response.content, "hello");
    assert!(!response.content_null);
    assert_eq!(response.refusal, None);
    assert_eq!(response.raw_body, None);
}

#[test]
fn non_stop_finish_reason_keeps_raw_body() {
    let body = r#"{"choices": [{"message": {"content": "cut off"}, "finish_reason": "length"}]}"#;

    let response = parse_completion_body(body, 0).unwrap();

    assert_eq!(response.content, "cut off");
    assert_eq!(response.raw_body.as_deref(), Some(body));
}

#[test]
fn missing_choices_error_carries_raw_body() {
    let body = r#"{"error": {"code": 502, "message": "upstream failed"}}"#;

    let error = parse_completion_body(body, 0).unwrap_err();

    assert_eq!(error.raw_body.as_deref(), Some(body));
    assert!(error.message.contains("upstream failed"));
}

#[test]
fn unparseable_body_error_carries_raw_body() {
    let body = "<html>bad gateway</html>";

    let error = parse_completion_body(body, 0).unwrap_err();

    assert_eq!(error.raw_body.as_deref(), Some(body));
    let wrapped = anyhow::Error::from(error);
    assert_eq!(raw_body_of(&wrapped).as_deref(), Some(body));
}
