use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub enum RunnerEvent {
    RunStarted,
    TrialExpanded { trial_id: String },
    RunFinished,
}
