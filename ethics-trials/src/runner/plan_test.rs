use std::path::PathBuf;

use super::expand_experiment;
use crate::domain::experiment::{ExperimentDefinition, ExperimentUnit};
use crate::domain::scenario::{
    PromptDefinition, ScenarioCondition, ScenarioDefinition, ScenarioVariant, VariantChanges,
};
use crate::loader::studies::LoadedStudy;

#[test]
fn expand_experiment_errors_when_condition_variant_is_missing() {
    let study = LoadedStudy {
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
            decisions: Vec::new(),
            prompt: PromptDefinition {
                system: None,
                user: "prompt".to_owned(),
                context: None,
                coercion: None,
            },
            variants: vec![ScenarioVariant {
                id: "present".to_owned(),
                label: "Present".to_owned(),
                description: None,
                changes: VariantChanges::default(),
            }],
            conditions: vec![ScenarioCondition {
                id: "condition".to_owned(),
                label: "Condition".to_owned(),
                variant: Some("missing".to_owned()),
                output_format: None,
                prompt_overrides: serde_yaml::Value::Null,
            }],
            attached_rubrics: Vec::new(),
            metadata: serde_yaml::Value::Null,
        },
        experiments: vec![ExperimentDefinition {
            id: "experiment".to_owned(),
            title: "Experiment".to_owned(),
            description: None,
            mode: "abab".to_owned(),
            tags: Vec::new(),
            repeats: 1,
            units: vec![ExperimentUnit {
                label: "unit".to_owned(),
                scenario: "scenario".to_owned(),
                condition: Some("condition".to_owned()),
                variant: None,
                output_format: None,
                repeat: None,
            }],
            comparisons: Vec::new(),
            attached_rubrics: Vec::new(),
            metadata: serde_yaml::Value::Null,
        }],
        rubrics: Vec::new(),
    };

    let error = expand_experiment(&study, "experiment", &["model".to_owned()]).unwrap_err();

    assert!(error.to_string().contains("unknown variant `missing`"));
}
