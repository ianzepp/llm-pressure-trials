use std::collections::HashSet;

use anyhow::{Result, bail};

use crate::loader::studies::LoadedStudy;

pub fn validate_study(study: &LoadedStudy) -> Result<()> {
    let scenario = &study.scenario;

    if scenario.id.trim().is_empty() {
        bail!("scenario id must not be empty");
    }

    let condition_ids: HashSet<&str> = scenario.conditions.iter().map(|item| item.id.as_str()).collect();
    let variant_ids: HashSet<&str> = scenario.variants.iter().map(|item| item.id.as_str()).collect();
    let rubric_ids: HashSet<&str> = study.rubrics.iter().map(|item| item.id.as_str()).collect();

    for condition in &scenario.conditions {
        if let Some(variant_id) = condition.variant.as_deref() {
            if !variant_ids.contains(variant_id) {
                bail!(
                    "condition `{}` references missing variant `{}`",
                    condition.id,
                    variant_id
                );
            }
        }
    }

    for rubric_id in &scenario.attached_rubrics {
        if !rubric_ids.contains(rubric_id.as_str()) {
            bail!("scenario references missing rubric `{rubric_id}`");
        }
    }

    for experiment in &study.experiments {
        for unit in &experiment.units {
            if unit.scenario != scenario.id {
                bail!(
                    "experiment `{}` references scenario `{}` but study scenario is `{}`",
                    experiment.id,
                    unit.scenario,
                    scenario.id
                );
            }

            if let Some(condition_id) = unit.condition.as_deref() {
                if !condition_ids.contains(condition_id) {
                    bail!(
                        "experiment `{}` references missing condition `{}`",
                        experiment.id,
                        condition_id
                    );
                }
            }

            if let Some(variant_id) = unit.variant.as_deref() {
                if !variant_ids.contains(variant_id) {
                    bail!(
                        "experiment `{}` references missing variant `{}`",
                        experiment.id,
                        variant_id
                    );
                }
            }
        }

        for rubric_id in &experiment.attached_rubrics {
            if !rubric_ids.contains(rubric_id.as_str()) {
                bail!("experiment `{}` references missing rubric `{rubric_id}`", experiment.id);
            }
        }
    }

    Ok(())
}
