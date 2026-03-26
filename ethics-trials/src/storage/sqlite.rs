use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::domain::model::ModelPricing;
use crate::domain::result::{ComparisonResult, FailedTrial, RunMetadata, RunSummary, TrialResult};

pub struct StudyDatabase {
    path: PathBuf,
}

impl StudyDatabase {
    pub fn new(study_root: &Path) -> Result<Self> {
        let path = study_root.join("sqlite.db");
        let db = Self { path };
        db.init()?;
        Ok(db)
    }

    pub fn insert_run_metadata(&self, metadata: &RunMetadata) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            r#"
            INSERT INTO runs (
                run_id, study_id, selection, models_json, created_at_unix_ms,
                total_trials, successful_trials, failure_count, comparison_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, NULL, NULL, NULL)
            ON CONFLICT(run_id) DO UPDATE SET
                study_id = excluded.study_id,
                selection = excluded.selection,
                models_json = excluded.models_json,
                created_at_unix_ms = excluded.created_at_unix_ms
            "#,
            params![
                metadata.run_id,
                metadata.study_id,
                metadata.selection,
                serde_json::to_string(&metadata.models)?,
                to_i64(metadata.created_at_unix_ms)?,
            ],
        )
        .context("failed to insert run metadata into sqlite.db")?;

        Ok(())
    }

    pub fn insert_run_pricing(&self, run_id: &str, pricing: &[ModelPricing]) -> Result<()> {
        let mut conn = self.open()?;
        let tx = conn
            .transaction()
            .context("failed to start sqlite pricing transaction")?;

        for p in pricing {
            tx.execute(
                r#"
                INSERT OR REPLACE INTO model_pricing (
                    run_id, model_id, prompt_cost_per_token, completion_cost_per_token
                ) VALUES (?1, ?2, ?3, ?4)
                "#,
                params![
                    run_id,
                    p.model_id,
                    p.prompt_cost_per_token,
                    p.completion_cost_per_token,
                ],
            )
            .context("failed to insert model pricing into sqlite.db")?;
        }

        tx.commit()
            .context("failed to commit sqlite pricing transaction")?;
        Ok(())
    }

    pub fn insert_trial_result(&self, run_id: &str, result: &TrialResult) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            r#"
            INSERT INTO trial_results (
                run_id, trial_id, model_id, condition_label, output_format,
                response_content, tokens_in, tokens_out, latency_ms,
                normalized_json, grading_json, prompt_json, repetition_index,
                coercion_content, coercion_normalized_json, coercion_grading_json,
                coercion_tokens_in, coercion_tokens_out, coercion_latency_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                      ?14, ?15, ?16, ?17, ?18, ?19)
            ON CONFLICT(run_id, trial_id) DO UPDATE SET
                model_id = excluded.model_id,
                condition_label = excluded.condition_label,
                output_format = excluded.output_format,
                response_content = excluded.response_content,
                tokens_in = excluded.tokens_in,
                tokens_out = excluded.tokens_out,
                latency_ms = excluded.latency_ms,
                normalized_json = excluded.normalized_json,
                grading_json = excluded.grading_json,
                prompt_json = excluded.prompt_json,
                repetition_index = excluded.repetition_index,
                coercion_content = excluded.coercion_content,
                coercion_normalized_json = excluded.coercion_normalized_json,
                coercion_grading_json = excluded.coercion_grading_json,
                coercion_tokens_in = excluded.coercion_tokens_in,
                coercion_tokens_out = excluded.coercion_tokens_out,
                coercion_latency_ms = excluded.coercion_latency_ms
            "#,
            params![
                run_id,
                result.trial_id,
                result.model_id,
                result.condition_label,
                result.output_format,
                result.response.content,
                result.response.tokens_in.map(i64::from),
                result.response.tokens_out.map(i64::from),
                to_i64(result.response.latency_ms)?,
                serde_json::to_string(&result.normalized)?,
                serde_json::to_string(&result.grading)?,
                serde_json::to_string(&result.prompt)?,
                result.repetition_index,
                result.coercion_response.as_ref().map(|r| r.content.clone()),
                result.coercion_normalized.as_ref().map(|n| serde_json::to_string(n)).transpose()?,
                result.coercion_grading.as_ref().map(|g| serde_json::to_string(g)).transpose()?,
                result.coercion_response.as_ref().and_then(|r| r.tokens_in.map(i64::from)),
                result.coercion_response.as_ref().and_then(|r| r.tokens_out.map(i64::from)),
                result.coercion_response.as_ref().map(|r| r.latency_ms as i64),
            ],
        )
        .context("failed to insert trial result into sqlite.db")?;

        Ok(())
    }

    pub fn insert_failed_trial(&self, run_id: &str, failure: &FailedTrial) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            r#"
            INSERT INTO failed_trials (
                run_id, trial_id, model_id, condition_label, error
            ) VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(run_id, trial_id) DO UPDATE SET
                model_id = excluded.model_id,
                condition_label = excluded.condition_label,
                error = excluded.error
            "#,
            params![
                run_id,
                failure.trial_id,
                failure.model_id,
                failure.condition_label,
                failure.error,
            ],
        )
        .context("failed to insert failed trial into sqlite.db")?;

        Ok(())
    }

    pub fn replace_comparisons(
        &self,
        run_id: &str,
        comparisons: &[ComparisonResult],
    ) -> Result<()> {
        let mut conn = self.open()?;
        let tx = conn
            .transaction()
            .context("failed to start sqlite comparison transaction")?;

        tx.execute("DELETE FROM comparisons WHERE run_id = ?1", params![run_id])
            .context("failed to clear existing run comparisons")?;

        for comparison in comparisons {
            tx.execute(
                r#"
                INSERT INTO comparisons (
                    run_id, model_id, comparison_id, left_label, right_label, metric,
                    left_decision, right_decision, decision_shift, refusal_shift,
                    hedging_delta, specificity_delta
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
                params![
                    run_id,
                    comparison.model_id,
                    comparison.comparison_id,
                    comparison.left_label,
                    comparison.right_label,
                    comparison.metric,
                    comparison.left_decision,
                    comparison.right_decision,
                    bool_to_i64(comparison.decision_shift),
                    bool_to_i64(comparison.refusal_shift),
                    i64::from(comparison.hedging_delta),
                    i64::from(comparison.specificity_delta),
                ],
            )
            .context("failed to insert comparison into sqlite.db")?;
        }

        tx.commit()
            .context("failed to commit sqlite comparison transaction")?;
        Ok(())
    }

    pub fn update_run_summary(&self, summary: &RunSummary) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            r#"
            UPDATE runs
            SET total_trials = ?2,
                successful_trials = ?3,
                failure_count = ?4,
                comparison_count = ?5,
                models_json = ?6,
                selection = ?7
            WHERE run_id = ?1
            "#,
            params![
                summary.run_id,
                to_i64(summary.total_trials as u128)?,
                to_i64(summary.successful_trials as u128)?,
                to_i64(summary.failed_trials as u128)?,
                to_i64(summary.comparisons as u128)?,
                serde_json::to_string(&summary.models)?,
                summary.selection,
            ],
        )
        .context("failed to update run summary in sqlite.db")?;

        Ok(())
    }

    pub fn latest_run_id(&self) -> Result<Option<String>> {
        let conn = self.open()?;
        let run_id = conn
            .query_row(
                "SELECT run_id FROM runs ORDER BY created_at_unix_ms DESC LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("failed to query latest run id from sqlite.db")?;
        Ok(run_id)
    }

    pub fn run_selection(&self, run_id: &str) -> Result<Option<String>> {
        let conn = self.open()?;
        let selection = conn
            .query_row(
                "SELECT selection FROM runs WHERE run_id = ?1",
                params![run_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("failed to query run selection from sqlite.db")?;
        Ok(selection)
    }

    pub fn failed_trial_count(&self, run_id: &str) -> Result<usize> {
        let conn = self.open()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM failed_trials WHERE run_id = ?1",
                params![run_id],
                |row| row.get(0),
            )
            .context("failed to count failed trials from sqlite.db")?;
        Ok(count as usize)
    }

    pub fn build_run_report(&self, run_id: &str) -> Result<RunReport> {
        let conn = self.open()?;
        let run_meta = conn
            .query_row(
                "SELECT run_id, study_id, selection, models_json FROM runs WHERE run_id = ?1",
                params![run_id],
                |row| {
                    let models_json: String = row.get(3)?;
                    let models: Vec<String> =
                        serde_json::from_str(&models_json).map_err(to_sql_error)?;
                    Ok(RunMetaRow {
                        run_id: row.get(0)?,
                        study_id: row.get(1)?,
                        selection: row.get(2)?,
                        models,
                    })
                },
            )
            .optional()
            .context("failed to query run metadata from sqlite.db")?
            .ok_or_else(|| anyhow::anyhow!("run `{run_id}` not found"))?;

        let mut stmt = conn.prepare(
            r#"
            SELECT model_id, normalized_json, grading_json, tokens_in, tokens_out, latency_ms,
                   condition_label, coercion_grading_json, coercion_normalized_json
            FROM trial_results
            WHERE run_id = ?1
            ORDER BY model_id, trial_id
            "#,
        )?;
        let trial_rows = stmt
            .query_map(params![run_id], |row| {
                let normalized_json: String = row.get(1)?;
                let grading_json: String = row.get(2)?;
                let coercion_grading_json: Option<String> = row.get(7)?;
                let coercion_normalized_json: Option<String> = row.get(8)?;
                Ok(TrialReportRow {
                    model_id: row.get(0)?,
                    normalized: serde_json::from_str(&normalized_json).map_err(to_sql_error)?,
                    grading: serde_json::from_str(&grading_json).map_err(to_sql_error)?,
                    tokens_in: row.get(3)?,
                    tokens_out: row.get(4)?,
                    latency_ms: row.get(5)?,
                    condition_label: row.get(6)?,
                    coercion_grading: coercion_grading_json
                        .as_deref()
                        .map(|s| serde_json::from_str(s).map_err(to_sql_error))
                        .transpose()?,
                    coercion_decision: coercion_normalized_json
                        .as_deref()
                        .and_then(|s| match serde_json::from_str::<StoredNormalized>(s) {
                            Ok(n) => n.decision,
                            Err(_) => None,
                        }),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to query trial results for report")?;

        let mut comparison_stmt = conn.prepare(
            r#"
            SELECT model_id, decision_shift, refusal_shift
            FROM comparisons
            WHERE run_id = ?1
            ORDER BY model_id, comparison_id
            "#,
        )?;
        let comparison_rows = comparison_stmt
            .query_map(params![run_id], |row| {
                Ok(ComparisonReportRow {
                    model_id: row.get(0)?,
                    decision_shift: row.get::<_, i64>(1)? != 0,
                    refusal_shift: row.get::<_, i64>(2)? != 0,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to query comparisons for report")?;

        let mut pricing_stmt = conn.prepare(
            r#"
            SELECT model_id, prompt_cost_per_token, completion_cost_per_token
            FROM model_pricing
            WHERE run_id = ?1
            "#,
        )?;
        let pricing_map: std::collections::HashMap<String, PricingRow> = pricing_stmt
            .query_map(params![run_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    PricingRow {
                        prompt_cost_per_token: row.get(1)?,
                        completion_cost_per_token: row.get(2)?,
                    },
                ))
            })?
            .collect::<rusqlite::Result<_>>()
            .context("failed to query model pricing for report")?;

        let aggregate =
            AggregateStats::from_rows(&trial_rows, &comparison_rows, &pricing_map, None);
        let mut model_ids = trial_rows
            .iter()
            .map(|row| row.model_id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        for row in &comparison_rows {
            model_ids.insert(row.model_id.clone());
        }

        let by_model = model_ids
            .into_iter()
            .map(|model_id| {
                let votes = compute_votes(&trial_rows, &model_id);
                ModelReport {
                    model_id: model_id.clone(),
                    stats: AggregateStats::from_rows(
                        &trial_rows,
                        &comparison_rows,
                        &pricing_map,
                        Some(&model_id),
                    ),
                    votes,
                }
            })
            .collect::<Vec<_>>();

        Ok(RunReport {
            run_id: run_meta.run_id,
            study_id: run_meta.study_id,
            selection: run_meta.selection,
            models: run_meta.models,
            aggregate,
            by_model,
        })
    }

    fn init(&self) -> Result<()> {
        let conn = self.open()?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS runs (
                run_id TEXT PRIMARY KEY,
                study_id TEXT NOT NULL,
                selection TEXT NOT NULL,
                models_json TEXT NOT NULL,
                created_at_unix_ms INTEGER NOT NULL,
                total_trials INTEGER,
                successful_trials INTEGER,
                failure_count INTEGER,
                comparison_count INTEGER
            );

            CREATE TABLE IF NOT EXISTS trial_results (
                run_id TEXT NOT NULL,
                trial_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                condition_label TEXT NOT NULL,
                output_format TEXT,
                response_content TEXT NOT NULL,
                tokens_in INTEGER,
                tokens_out INTEGER,
                latency_ms INTEGER NOT NULL,
                normalized_json TEXT NOT NULL,
                grading_json TEXT NOT NULL,
                prompt_json TEXT NOT NULL,
                PRIMARY KEY (run_id, trial_id),
                FOREIGN KEY (run_id) REFERENCES runs(run_id)
            );

            CREATE TABLE IF NOT EXISTS comparisons (
                run_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                comparison_id TEXT NOT NULL,
                left_label TEXT NOT NULL,
                right_label TEXT NOT NULL,
                metric TEXT,
                left_decision TEXT,
                right_decision TEXT,
                decision_shift INTEGER NOT NULL,
                refusal_shift INTEGER NOT NULL,
                hedging_delta INTEGER NOT NULL,
                specificity_delta INTEGER NOT NULL,
                PRIMARY KEY (run_id, model_id, comparison_id),
                FOREIGN KEY (run_id) REFERENCES runs(run_id)
            );

            CREATE TABLE IF NOT EXISTS failed_trials (
                run_id TEXT NOT NULL,
                trial_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                condition_label TEXT NOT NULL,
                error TEXT NOT NULL,
                PRIMARY KEY (run_id, trial_id),
                FOREIGN KEY (run_id) REFERENCES runs(run_id)
            );

            CREATE TABLE IF NOT EXISTS model_pricing (
                run_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                prompt_cost_per_token REAL NOT NULL,
                completion_cost_per_token REAL NOT NULL,
                PRIMARY KEY (run_id, model_id),
                FOREIGN KEY (run_id) REFERENCES runs(run_id)
            );

            CREATE INDEX IF NOT EXISTS idx_trial_results_run_id ON trial_results(run_id);
            CREATE INDEX IF NOT EXISTS idx_trial_results_model_id ON trial_results(model_id);
            CREATE INDEX IF NOT EXISTS idx_comparisons_run_id ON comparisons(run_id);
            CREATE INDEX IF NOT EXISTS idx_comparisons_model_id ON comparisons(model_id);
            CREATE INDEX IF NOT EXISTS idx_failed_trials_run_id ON failed_trials(run_id);
            "#,
        )
        .context("failed to initialize study sqlite.db schema")?;

        ensure_column(&conn, "runs", "successful_trials", "INTEGER")?;
        ensure_column(&conn, "runs", "failure_count", "INTEGER")?;
        ensure_column(
            &conn,
            "trial_results",
            "repetition_index",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        ensure_column(&conn, "trial_results", "coercion_content", "TEXT")?;
        ensure_column(&conn, "trial_results", "coercion_normalized_json", "TEXT")?;
        ensure_column(&conn, "trial_results", "coercion_grading_json", "TEXT")?;
        ensure_column(&conn, "trial_results", "coercion_tokens_in", "INTEGER")?;
        ensure_column(&conn, "trial_results", "coercion_tokens_out", "INTEGER")?;
        ensure_column(&conn, "trial_results", "coercion_latency_ms", "INTEGER")?;

        Ok(())
    }

    fn open(&self) -> Result<Connection> {
        let conn = Connection::open(&self.path)
            .with_context(|| format!("failed to open {}", self.path.display()))?;
        conn.busy_timeout(std::time::Duration::from_secs(3))
            .context("failed to set sqlite busy_timeout")?;
        Ok(conn)
    }
}

#[derive(Debug, Serialize)]
pub struct RunReport {
    pub run_id: String,
    pub study_id: String,
    pub selection: String,
    pub models: Vec<String>,
    pub aggregate: AggregateStats,
    pub by_model: Vec<ModelReport>,
}

#[derive(Debug, Serialize)]
pub struct ModelReport {
    pub model_id: String,
    #[serde(flatten)]
    pub stats: AggregateStats,
    pub votes: Vec<VoteSummary>,
}

#[derive(Debug, Default, Serialize)]
pub struct AggregateStats {
    pub total_trials: usize,
    pub format_valid: usize,
    pub refusal_present: usize,
    pub compliance_counts: Vec<(String, usize)>,
    pub decision_counts: Vec<(String, usize)>,
    pub avg_hedging_level: f64,
    pub avg_harmful_specificity: f64,
    pub comparison_count: usize,
    pub decision_shift_count: usize,
    pub refusal_shift_count: usize,
    pub coercion_attempts: usize,
    pub coercion_overrides: usize,
    pub judged_count: usize,
    pub judged_decision_counts: Vec<(String, usize)>,
    pub judged_compliance_counts: Vec<(String, usize)>,
    pub usage: UsageStats,
}

#[derive(Debug, Default, Serialize)]
pub struct UsageStats {
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub latency_min_ms: Option<u64>,
    pub latency_max_ms: Option<u64>,
    pub latency_avg_ms: f64,
    pub latency_stddev_ms: f64,
    pub estimated_cost_usd: f64,
}

#[derive(Debug)]
struct RunMetaRow {
    run_id: String,
    study_id: String,
    selection: String,
    models: Vec<String>,
}

#[derive(Debug)]
struct TrialReportRow {
    model_id: String,
    condition_label: String,
    normalized: StoredNormalized,
    grading: StoredGrading,
    tokens_in: Option<i64>,
    tokens_out: Option<i64>,
    latency_ms: i64,
    coercion_grading: Option<StoredGrading>,
    coercion_decision: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VoteSummary {
    pub model_id: String,
    pub condition_label: String,
    pub winning_decision: String,
    pub vote_count: usize,
    pub total_votes: usize,
    pub unanimous: bool,
}

struct PricingRow {
    prompt_cost_per_token: f64,
    completion_cost_per_token: f64,
}

#[derive(Debug)]
struct ComparisonReportRow {
    model_id: String,
    decision_shift: bool,
    refusal_shift: bool,
}

#[derive(Debug, serde::Deserialize)]
struct StoredNormalized {
    decision: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct StoredGrading {
    format_valid: bool,
    refusal_present: bool,
    compliance_level: String,
    harmful_specificity: u8,
    hedging_level: u8,
    #[serde(default)]
    judge_decision: Option<String>,
    #[serde(default)]
    judge_compliance: Option<String>,
}

impl AggregateStats {
    fn from_rows(
        trial_rows: &[TrialReportRow],
        comparison_rows: &[ComparisonReportRow],
        pricing_map: &std::collections::HashMap<String, PricingRow>,
        model_filter: Option<&str>,
    ) -> Self {
        let mut format_valid = 0usize;
        let mut refusal_present = 0usize;
        let mut hedging_sum = 0usize;
        let mut specificity_sum = 0usize;
        let mut compliance_counts = std::collections::BTreeMap::<String, usize>::new();
        let mut decision_counts = std::collections::BTreeMap::<String, usize>::new();
        let mut total_trials = 0usize;

        let mut tokens_in_sum = 0u64;
        let mut tokens_out_sum = 0u64;
        let mut latency_min: Option<u64> = None;
        let mut latency_max: Option<u64> = None;
        let mut latency_sum = 0u64;
        let mut latency_welford_mean = 0.0f64;
        let mut latency_welford_m2 = 0.0f64;
        let mut cost_sum = 0.0f64;
        let mut coercion_attempts = 0usize;
        let mut coercion_overrides = 0usize;
        let mut judged_count = 0usize;
        let mut judged_decisions = std::collections::BTreeMap::<String, usize>::new();
        let mut judged_compliances = std::collections::BTreeMap::<String, usize>::new();

        for row in trial_rows
            .iter()
            .filter(|row| model_filter.is_none_or(|id| row.model_id == id))
        {
            total_trials += 1;
            if row.grading.format_valid {
                format_valid += 1;
            }
            if row.grading.refusal_present {
                refusal_present += 1;
            }
            hedging_sum += usize::from(row.grading.hedging_level);
            specificity_sum += usize::from(row.grading.harmful_specificity);
            *compliance_counts
                .entry(row.grading.compliance_level.clone())
                .or_insert(0) += 1;
            *decision_counts
                .entry(
                    row.normalized
                        .decision
                        .clone()
                        .unwrap_or_else(|| "NONE".to_owned()),
                )
                .or_insert(0) += 1;

            let t_in = row.tokens_in.unwrap_or(0).max(0) as u64;
            let t_out = row.tokens_out.unwrap_or(0).max(0) as u64;
            tokens_in_sum += t_in;
            tokens_out_sum += t_out;

            let lat = row.latency_ms.max(0) as u64;
            latency_sum += lat;
            latency_min = Some(latency_min.map_or(lat, |prev| prev.min(lat)));
            latency_max = Some(latency_max.map_or(lat, |prev| prev.max(lat)));

            let delta = lat as f64 - latency_welford_mean;
            latency_welford_mean += delta / total_trials as f64;
            let delta2 = lat as f64 - latency_welford_mean;
            latency_welford_m2 += delta * delta2;

            if let Some(p) = pricing_map.get(&row.model_id) {
                cost_sum += t_in as f64 * p.prompt_cost_per_token
                    + t_out as f64 * p.completion_cost_per_token;
            }

            if let Some(cg) = &row.coercion_grading {
                coercion_attempts += 1;
                if !cg.refusal_present {
                    coercion_overrides += 1;
                }
            }

            if let Some(jd) = &row.grading.judge_decision {
                judged_count += 1;
                *judged_decisions.entry(jd.clone()).or_insert(0) += 1;
            }
            if let Some(jc) = &row.grading.judge_compliance {
                *judged_compliances.entry(jc.clone()).or_insert(0) += 1;
            }
        }

        let filtered_comparisons = comparison_rows
            .iter()
            .filter(|row| model_filter.is_none_or(|id| row.model_id == id))
            .collect::<Vec<_>>();

        let comparison_count = filtered_comparisons.len();
        let decision_shift_count = filtered_comparisons
            .iter()
            .filter(|row| row.decision_shift)
            .count();
        let refusal_shift_count = filtered_comparisons
            .iter()
            .filter(|row| row.refusal_shift)
            .count();

        Self {
            total_trials,
            format_valid,
            refusal_present,
            compliance_counts: compliance_counts.into_iter().collect(),
            decision_counts: decision_counts.into_iter().collect(),
            avg_hedging_level: average(hedging_sum, total_trials),
            avg_harmful_specificity: average(specificity_sum, total_trials),
            comparison_count,
            decision_shift_count,
            refusal_shift_count,
            coercion_attempts,
            coercion_overrides,
            judged_count,
            judged_decision_counts: judged_decisions.into_iter().collect(),
            judged_compliance_counts: judged_compliances.into_iter().collect(),
            usage: UsageStats {
                total_tokens_in: tokens_in_sum,
                total_tokens_out: tokens_out_sum,
                latency_min_ms: latency_min,
                latency_max_ms: latency_max,
                latency_avg_ms: if total_trials > 0 {
                    latency_sum as f64 / total_trials as f64
                } else {
                    0.0
                },
                latency_stddev_ms: if total_trials > 1 {
                    (latency_welford_m2 / (total_trials - 1) as f64).sqrt()
                } else {
                    0.0
                },
                estimated_cost_usd: cost_sum,
            },
        }
    }
}

fn to_i64(value: u128) -> Result<i64> {
    i64::try_from(value).context("numeric value overflow while writing sqlite.db")
}

fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let existing = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    if existing.iter().any(|name| name == column) {
        return Ok(());
    }

    conn.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
        [],
    )
    .with_context(|| format!("failed to add column `{column}` to `{table}`"))?;

    Ok(())
}

fn average(sum: usize, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        sum as f64 / count as f64
    }
}

fn compute_votes(trial_rows: &[TrialReportRow], model_id: &str) -> Vec<VoteSummary> {
    let mut by_condition =
        std::collections::BTreeMap::<String, std::collections::BTreeMap<String, usize>>::new();

    for row in trial_rows
        .iter()
        .filter(|row| row.model_id == model_id)
    {
        // Prefer judge decision > coercion decision > primary decision
        let decision = row
            .grading
            .judge_decision
            .clone()
            .or_else(|| row.coercion_decision.clone())
            .or_else(|| row.normalized.decision.clone())
            .unwrap_or_else(|| "NONE".to_owned());
        *by_condition
            .entry(row.condition_label.clone())
            .or_default()
            .entry(decision)
            .or_insert(0) += 1;
    }

    by_condition
        .into_iter()
        .map(|(condition_label, tallies)| {
            let total_votes: usize = tallies.values().sum();
            let (winning_decision, vote_count) = tallies
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .unwrap_or_else(|| ("NONE".to_owned(), 0));
            VoteSummary {
                model_id: model_id.to_owned(),
                condition_label,
                winning_decision,
                unanimous: vote_count == total_votes,
                vote_count,
                total_votes,
            }
        })
        .collect()
}

fn to_sql_error(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}
