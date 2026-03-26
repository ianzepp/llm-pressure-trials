use std::path::PathBuf;

use super::expand_run_request;
use crate::domain::scenario::{PromptDefinition, ScenarioDefinition};
use crate::loader::studies::LoadedStudy;

#[test]
fn expand_run_request_rejects_missing_selection() {
    let study = fixture_study();

    let error = expand_run_request(&study, None, None, &["model".to_owned()]).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("exactly one of scenario_id or experiment_id must be set")
    );
}

#[test]
fn expand_run_request_rejects_ambiguous_selection() {
    let study = fixture_study();

    let error = expand_run_request(
        &study,
        Some("scenario"),
        Some("experiment"),
        &["model".to_owned()],
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("exactly one of scenario_id or experiment_id must be set")
    );
}

fn fixture_study() -> LoadedStudy {
    LoadedStudy {
        id: "study".to_owned(),
        root: PathBuf::from("studies/study"),
        scenario: ScenarioDefinition {
            id: "scenario".to_owned(),
            title: "Scenario".to_owned(),
            family: "family".to_owned(),
            description: "description".to_owned(),
            hypothesis: None,
            tags: Vec::new(),
            risk_level: None,
            response_mode: None,
            output_formats: Vec::new(),
            prompt: PromptDefinition {
                system: None,
                user: "prompt".to_owned(),
                context: None,
            },
            variants: Vec::new(),
            conditions: Vec::new(),
            attached_rubrics: Vec::new(),
            metadata: serde_yaml::Value::Null,
        },
        experiments: Vec::new(),
        rubrics: Vec::new(),
    }
}
