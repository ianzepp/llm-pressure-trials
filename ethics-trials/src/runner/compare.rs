use std::collections::HashMap;

use crate::domain::experiment::ExperimentDefinition;
use crate::domain::result::{ComparisonResult, TrialResult};

pub fn compare_results(
    experiment: Option<&ExperimentDefinition>,
    results: &[TrialResult],
) -> Vec<ComparisonResult> {
    let Some(experiment) = experiment else {
        return Vec::new();
    };

    let mut indexed = HashMap::new();
    for result in results {
        indexed.insert(
            (result.model_id.clone(), result.condition_label.clone()),
            result,
        );
    }

    let mut comparisons = Vec::new();

    for model_id in results
        .iter()
        .map(|item| item.model_id.clone())
        .collect::<std::collections::BTreeSet<_>>()
    {
        for comparison in &experiment.comparisons {
            let left = indexed.get(&(model_id.clone(), comparison.left.clone()));
            let right = indexed.get(&(model_id.clone(), comparison.right.clone()));

            if let (Some(left), Some(right)) = (left, right) {
                comparisons.push(ComparisonResult {
                    model_id: model_id.clone(),
                    comparison_id: comparison.id.clone(),
                    left_label: comparison.left.clone(),
                    right_label: comparison.right.clone(),
                    metric: comparison.metric.clone(),
                    left_decision: left.normalized.decision.clone(),
                    right_decision: right.normalized.decision.clone(),
                    decision_shift: left.normalized.decision != right.normalized.decision,
                    refusal_shift: left.grading.refusal_present != right.grading.refusal_present,
                    hedging_delta: right.grading.hedging_level as i16
                        - left.grading.hedging_level as i16,
                    specificity_delta: right.grading.harmful_specificity as i16
                        - left.grading.harmful_specificity as i16,
                });
            }
        }
    }

    comparisons
}
