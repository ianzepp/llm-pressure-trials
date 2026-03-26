use anyhow::{Result, bail};

use crate::domain::scenario::{ScenarioCondition, ScenarioDefinition, ScenarioVariant};
use crate::domain::trial::{ResolvedPrompt, TrialUnit};
use crate::loader::studies::LoadedStudy;

pub fn expand_experiment(
    study: &LoadedStudy,
    experiment_id: &str,
    models: &[String],
) -> Result<Vec<TrialUnit>> {
    let experiment = study
        .experiments
        .iter()
        .find(|item| item.id == experiment_id)
        .ok_or_else(|| anyhow::anyhow!("unknown experiment `{experiment_id}`"))?;

    let mut units = Vec::new();

    for model_id in models {
        for unit in &experiment.units {
            let condition = unit
                .condition
                .as_deref()
                .and_then(|condition_id| find_condition(&study.scenario, condition_id));
            let variant = unit
                .variant
                .as_deref()
                .map(|variant_id| find_variant(&study.scenario, variant_id))
                .transpose()?
                .or_else(|| condition.and_then(|item| item.variant.as_deref()).and_then(|variant_id| find_variant(&study.scenario, variant_id).ok()));

            let prompt = resolve_prompt(&study.scenario, variant.as_ref());
            let condition_label = unit.label.clone();
            let output_format = unit
                .output_format
                .clone()
                .or_else(|| condition.and_then(|item| item.output_format.clone()))
                .or_else(|| variant.as_ref().and_then(|item| item.changes.output_format.clone()));

            units.push(TrialUnit {
                id: format!("{experiment_id}:{condition_label}:{model_id}"),
                study_id: study.id.clone(),
                scenario_id: study.scenario.id.clone(),
                experiment_id: Some(experiment.id.clone()),
                condition_id: unit.condition.clone(),
                condition_label,
                variant_id: variant.map(|item| item.id),
                model_id: model_id.clone(),
                output_format,
                prompt,
                repetition_index: 0,
            });
        }
    }

    Ok(units)
}

pub fn expand_scenario(
    study: &LoadedStudy,
    scenario_id: &str,
    models: &[String],
) -> Result<Vec<TrialUnit>> {
    if study.scenario.id != scenario_id {
        bail!("unknown scenario `{scenario_id}`");
    }

    let mut units = Vec::new();
    for model_id in models {
        units.push(TrialUnit {
            id: format!("{scenario_id}:default:{model_id}"),
            study_id: study.id.clone(),
            scenario_id: study.scenario.id.clone(),
            experiment_id: None,
            condition_id: None,
            condition_label: "default".to_owned(),
            variant_id: None,
            model_id: model_id.clone(),
            output_format: None,
            prompt: resolve_prompt(&study.scenario, None),
            repetition_index: 0,
        });
    }

    Ok(units)
}

fn find_condition<'a>(scenario: &'a ScenarioDefinition, condition_id: &str) -> Option<&'a ScenarioCondition> {
    scenario.conditions.iter().find(|item| item.id == condition_id)
}

fn find_variant(scenario: &ScenarioDefinition, variant_id: &str) -> Result<ScenarioVariant> {
    scenario
        .variants
        .iter()
        .find(|item| item.id == variant_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("unknown variant `{variant_id}`"))
}

fn resolve_prompt(scenario: &ScenarioDefinition, variant: Option<&ScenarioVariant>) -> ResolvedPrompt {
    let mut user = scenario.prompt.user.clone();
    if let Some(suffix) = variant.and_then(|item| item.changes.prompt_suffix.as_deref()) {
        if !user.ends_with('\n') {
            user.push('\n');
        }
        user.push('\n');
        user.push_str(suffix.trim_end());
    }

    ResolvedPrompt {
        system: scenario.prompt.system.clone(),
        user,
        context: scenario.prompt.context.clone(),
    }
}
