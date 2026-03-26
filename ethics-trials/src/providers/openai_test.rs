use super::truncate;

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
