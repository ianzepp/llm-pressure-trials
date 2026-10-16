use std::path::PathBuf;

use super::expand_experiment;
use crate::runner::execute::{FollowupTurn, plan_followup_turn};
use crate::domain::experiment::{ExperimentDefinition, ExperimentUnit};
use crate::domain::scenario::{
    CoercionVariant, PromptDefinition, ScenarioCondition, ScenarioDefinition, ScenarioVariant, VariantChanges,
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
                escalation: None,
                coercion_variants: Vec::new(),
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
                temperature: None,
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

fn followup_study(coercion_variants: Vec<CoercionVariant>, repeats: u32) -> LoadedStudy {
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
            decisions: Vec::new(),
            prompt: PromptDefinition {
                system: None,
                user: "prompt".to_owned(),
                context: None,
                coercion: Some("plain coercion".to_owned()),
                escalation: None,
                coercion_variants,
            },
            variants: vec![ScenarioVariant {
                id: "with_followup".to_owned(),
                label: "With follow-up".to_owned(),
                description: None,
                changes: VariantChanges {
                    followup: Some("always sent".to_owned()),
                    ..VariantChanges::default()
                },
            }],
            conditions: Vec::new(),
            attached_rubrics: Vec::new(),
            metadata: serde_yaml::Value::Null,
        },
        experiments: vec![ExperimentDefinition {
            id: "experiment".to_owned(),
            title: "Experiment".to_owned(),
            description: None,
            mode: "abab".to_owned(),
            tags: Vec::new(),
            repeats,
            units: vec![
                ExperimentUnit {
                    label: "plain".to_owned(),
                    scenario: "scenario".to_owned(),
                    condition: None,
                    variant: None,
                    output_format: None,
                    repeat: None,
                    temperature: None,
                },
                ExperimentUnit {
                    label: "followup".to_owned(),
                    scenario: "scenario".to_owned(),
                    condition: None,
                    variant: Some("with_followup".to_owned()),
                    output_format: None,
                    repeat: Some(1),
                    temperature: None,
                },
            ],
            comparisons: Vec::new(),
            attached_rubrics: Vec::new(),
            metadata: serde_yaml::Value::Null,
        }],
        rubrics: Vec::new(),
    }
}

#[test]
fn variant_followup_flows_into_the_trial_unit_prompt() {
    let study = followup_study(Vec::new(), 1);

    let units = expand_experiment(&study, "experiment", &["model".to_owned()]).unwrap();

    assert_eq!(units.len(), 2);
    assert_eq!(units[0].prompt.followup, None);
    assert_eq!(units[1].prompt.followup.as_deref(), Some("always sent"));
    // Scenario-level coercion is untouched; the runner decides not to send it.
    assert_eq!(units[1].prompt.coercion.as_deref(), Some("plain coercion"));
}

#[test]
fn coercion_ordinal_counts_units_per_model_in_expansion_order() {
    let study = followup_study(Vec::new(), 3);
    let models = ["m1".to_owned(), "m2".to_owned()];

    let units = expand_experiment(&study, "experiment", &models).unwrap();

    // Per model: 3 repeats of "plain" + 1 of "followup" = 4 units.
    let ordinals: Vec<(&str, usize)> = units
        .iter()
        .map(|unit| (unit.model_id.as_str(), unit.coercion_ordinal))
        .collect();
    assert_eq!(
        ordinals,
        vec![
            ("m1", 0),
            ("m1", 1),
            ("m1", 2),
            ("m1", 3),
            ("m2", 0),
            ("m2", 1),
            ("m2", 2),
            ("m2", 3)
        ]
    );
}

#[test]
fn two_coercion_variants_alternate_by_ordinal_per_model() {
    let variants = vec![
        CoercionVariant {
            id: "a".to_owned(),
            text: "text a".to_owned(),
        },
        CoercionVariant {
            id: "b".to_owned(),
            text: "text b".to_owned(),
        },
    ];
    // 24 plain units per model (repeat 24) + 1 followup unit (excluded below).
    let study = followup_study(variants, 24);
    let models = ["m1".to_owned(), "m2".to_owned()];

    let units = expand_experiment(&study, "experiment", &models).unwrap();

    for model in &models {
        let mut chosen = Vec::new();
        for unit in units
            .iter()
            .filter(|unit| &unit.model_id == model && unit.prompt.followup.is_none())
        {
            match plan_followup_turn(unit, true) {
                FollowupTurn::Coercion { variant_id, .. } => {
                    chosen.push(variant_id.map(str::to_owned));
                }
                other => panic!("expected coercion, got {other:?}"),
            }
        }
        assert_eq!(chosen.len(), 24);
        let a = chosen.iter().filter(|id| id.as_deref() == Some("a")).count();
        let b = chosen.iter().filter(|id| id.as_deref() == Some("b")).count();
        assert_eq!((a, b), (12, 12));
        assert_eq!(chosen[0].as_deref(), Some("a"));
        assert_eq!(chosen[1].as_deref(), Some("b"));
    }
}
