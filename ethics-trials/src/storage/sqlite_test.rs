use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use rusqlite::Connection;

use super::StudyDatabase;
use crate::domain::result::{
    DeterministicGrade, FailedTrial, NormalizedResponse, RawModelResponse, RunMetadata,
    TrialResult,
};
use crate::domain::trial::ResolvedPrompt;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "ethics-trials-sqlite-test-{}-{id}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap_or_default();
    }
}

fn columns(conn: &Connection, table: &str) -> Vec<String> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})")).unwrap();
    stmt.query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap()
}

fn response(content: &str) -> RawModelResponse {
    RawModelResponse {
        content: content.to_owned(),
        tokens_in: Some(1),
        tokens_out: Some(2),
        latency_ms: 3,
        finish_reason: None,
        native_finish_reason: None,
        refusal: None,
        reasoning: None,
        content_null: false,
        raw_body: None,
    }
}

fn normalized() -> NormalizedResponse {
    NormalizedResponse {
        raw_text: String::new(),
        parsed_json: None,
        decision: None,
        justification: None,
    }
}

fn grade() -> DeterministicGrade {
    DeterministicGrade {
        format_valid: true,
        refusal_present: false,
        compliance_level: "full".to_owned(),
        harmful_specificity: 0,
        hedging_level: 0,
        notes: Vec::new(),
        judge_decision: None,
        judge_compliance: None,
        judge_reasoning: None,
        judge_model: None,
    }
}

fn result() -> TrialResult {
    TrialResult {
        trial_id: "t1".to_owned(),
        model_id: "m".to_owned(),
        condition_label: "c".to_owned(),
        output_format: None,
        repetition_index: 0,
        prompt: ResolvedPrompt {
            system: None,
            user: "u".to_owned(),
            context: None,
            coercion: None,
            escalation: None,
            followup: None,
            coercion_variants: Vec::new(),
        },
        response: response(""),
        normalized: normalized(),
        grading: grade(),
        coercion_response: None,
        coercion_normalized: None,
        coercion_grading: None,
        escalation_response: None,
        escalation_normalized: None,
        escalation_grading: None,
        followup_response: None,
        followup_normalized: None,
        followup_grading: None,
        coercion_variant: None,
    }
}

fn insert_run(db: &StudyDatabase) {
    db.insert_run_metadata(&RunMetadata {
        run_id: "r1".to_owned(),
        study_id: "s".to_owned(),
        selection: "experiment:e".to_owned(),
        models: vec!["m".to_owned()],
        created_at_unix_ms: 1,
    })
    .unwrap();
}

#[test]
fn old_database_gains_the_new_columns_on_open() {
    let dir = TempDir::new();
    let path = dir.0.join("sqlite.db");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE trial_results (run_id TEXT NOT NULL, trial_id TEXT NOT NULL,
                model_id TEXT NOT NULL, condition_label TEXT NOT NULL, output_format TEXT,
                response_content TEXT NOT NULL, tokens_in INTEGER, tokens_out INTEGER,
                latency_ms INTEGER NOT NULL, normalized_json TEXT NOT NULL,
                grading_json TEXT NOT NULL, prompt_json TEXT NOT NULL,
                PRIMARY KEY (run_id, trial_id));
             CREATE TABLE failed_trials (run_id TEXT NOT NULL, trial_id TEXT NOT NULL,
                model_id TEXT NOT NULL, condition_label TEXT NOT NULL, error TEXT NOT NULL,
                PRIMARY KEY (run_id, trial_id));",
        )
        .unwrap();
    }

    StudyDatabase::new(&dir.0).unwrap();

    let conn = Connection::open(&path).unwrap();
    let trial_columns = columns(&conn, "trial_results");
    for prefix in ["", "coercion_", "escalation_", "followup_"] {
        for suffix in [
            "finish_reason",
            "native_finish_reason",
            "refusal_text",
            "reasoning_text",
            "content_null",
            "raw_body",
        ] {
            let name = format!("{prefix}{suffix}");
            assert!(trial_columns.contains(&name), "missing column {name}");
        }
    }
    for name in [
        "followup_content",
        "followup_normalized_json",
        "followup_grading_json",
        "followup_tokens_in",
        "followup_tokens_out",
        "followup_latency_ms",
        "coercion_variant",
    ] {
        assert!(trial_columns.iter().any(|c| c == name), "missing column {name}");
    }
    assert!(columns(&conn, "failed_trials").contains(&"raw_body".to_owned()));
}

#[test]
fn trial_result_logging_and_followup_fields_round_trip() {
    let dir = TempDir::new();
    let db = StudyDatabase::new(&dir.0).unwrap();
    insert_run(&db);

    let mut item = result();
    item.response.content_null = true;
    item.response.finish_reason = Some("content_filter".to_owned());
    item.response.raw_body = Some("{\"raw\":1}".to_owned());
    let mut follow = response("second");
    follow.finish_reason = Some("stop".to_owned());
    follow.refusal = Some("no".to_owned());
    follow.reasoning = Some("because".to_owned());
    item.followup_response = Some(follow);
    item.followup_normalized = Some(normalized());
    item.followup_grading = Some(grade());
    item.coercion_variant = Some("a".to_owned());

    db.insert_trial_result("r1", &item).unwrap();
    // Upsert path must also work.
    db.insert_trial_result("r1", &item).unwrap();

    let conn = Connection::open(dir.0.join("sqlite.db")).unwrap();
    let row = conn
        .query_row(
            "SELECT finish_reason, content_null, raw_body, followup_content,
                    followup_finish_reason, followup_refusal_text, followup_reasoning_text,
                    followup_content_null, followup_tokens_out, coercion_variant,
                    coercion_content, coercion_content_null
             FROM trial_results WHERE run_id = 'r1' AND trial_id = 't1'",
            [],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<i64>>(11)?,
                ))
            },
        )
        .unwrap();

    assert_eq!(row.0.as_deref(), Some("content_filter"));
    assert_eq!(row.1, Some(1));
    assert_eq!(row.2.as_deref(), Some("{\"raw\":1}"));
    assert_eq!(row.3.as_deref(), Some("second"));
    assert_eq!(row.4.as_deref(), Some("stop"));
    assert_eq!(row.5.as_deref(), Some("no"));
    assert_eq!(row.6.as_deref(), Some("because"));
    assert_eq!(row.7, Some(0));
    assert_eq!(row.8, Some(2));
    assert_eq!(row.9.as_deref(), Some("a"));
    // Turns that did not happen stay NULL.
    assert_eq!(row.10, None);
    assert_eq!(row.11, None);
}

#[test]
fn failed_trial_raw_body_is_stored() {
    let dir = TempDir::new();
    let db = StudyDatabase::new(&dir.0).unwrap();
    insert_run(&db);

    db.insert_failed_trial(
        "r1",
        &FailedTrial {
            trial_id: "t1".to_owned(),
            model_id: "m".to_owned(),
            condition_label: "c".to_owned(),
            error: "boom".to_owned(),
            raw_body: Some("<html>".to_owned()),
        },
    )
    .unwrap();

    let conn = Connection::open(dir.0.join("sqlite.db")).unwrap();
    let body: Option<String> = conn
        .query_row("SELECT raw_body FROM failed_trials", [], |row| row.get(0))
        .unwrap();
    assert_eq!(body.as_deref(), Some("<html>"));
}

#[test]
fn archived_results_without_new_fields_still_deserialize() {
    let archived = r#"{
        "trial_id": "t", "model_id": "m", "condition_label": "c",
        "output_format": null, "repetition_index": 0,
        "prompt": {"system": null, "user": "u", "context": null},
        "response": {"content": "hi", "tokens_in": 1, "tokens_out": 2, "latency_ms": 3},
        "normalized": {"raw_text": "hi", "parsed_json": null, "decision": null, "justification": null},
        "grading": {"format_valid": true, "refusal_present": false, "compliance_level": "full",
                    "harmful_specificity": 0, "hedging_level": 0, "notes": []}
    }"#;

    let parsed: TrialResult = serde_json::from_str(archived).unwrap();

    assert_eq!(parsed.response.finish_reason, None);
    assert!(!parsed.response.content_null);
    assert!(parsed.followup_response.is_none());
    assert!(parsed.prompt.coercion_variants.is_empty());

    // New optional fields are omitted from output when unset.
    let json = serde_json::to_string(&parsed).unwrap();
    assert!(!json.contains("finish_reason"));
    assert!(!json.contains("followup"));
    assert!(!json.contains("coercion_variant"));
}
