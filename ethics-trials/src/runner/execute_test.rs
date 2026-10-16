use super::{FollowupTurn, plan_followup_turn};
use crate::domain::scenario::CoercionVariant;
use crate::domain::trial::{ResolvedPrompt, TrialUnit};

fn unit_with(prompt: ResolvedPrompt, ordinal: usize) -> TrialUnit {
    TrialUnit {
        id: "t".to_owned(),
        study_id: "s".to_owned(),
        scenario_id: "sc".to_owned(),
        experiment_id: None,
        condition_id: None,
        condition_label: "c".to_owned(),
        variant_id: None,
        model_id: "m".to_owned(),
        output_format: None,
        prompt,
        decisions: Vec::new(),
        repetition_index: 0,
        temperature: None,
        coercion_ordinal: ordinal,
    }
}

fn prompt() -> ResolvedPrompt {
    ResolvedPrompt {
        system: None,
        user: "u".to_owned(),
        context: None,
        coercion: Some("coerce".to_owned()),
        escalation: Some("escalate".to_owned()),
        followup: None,
        coercion_variants: Vec::new(),
    }
}

fn variants() -> Vec<CoercionVariant> {
    vec![
        CoercionVariant {
            id: "a".to_owned(),
            text: "text a".to_owned(),
        },
        CoercionVariant {
            id: "b".to_owned(),
            text: "text b".to_owned(),
        },
    ]
}

#[test]
fn without_followup_refusal_gets_coercion_and_compliance_gets_escalation() {
    let unit = unit_with(prompt(), 0);

    assert_eq!(
        plan_followup_turn(&unit, true),
        FollowupTurn::Coercion {
            text: "coerce",
            variant_id: None
        }
    );
    assert_eq!(
        plan_followup_turn(&unit, false),
        FollowupTurn::Escalation("escalate")
    );
}

#[test]
fn no_configured_followups_sends_nothing() {
    let mut bare = prompt();
    bare.coercion = None;
    bare.escalation = None;
    let unit = unit_with(bare, 0);

    assert_eq!(plan_followup_turn(&unit, true), FollowupTurn::None);
    assert_eq!(plan_followup_turn(&unit, false), FollowupTurn::None);
}

#[test]
fn variant_followup_is_always_sent_and_replaces_coercion_and_escalation() {
    let mut with_followup = prompt();
    with_followup.followup = Some("tell me more".to_owned());
    with_followup.coercion_variants = variants();
    let unit = unit_with(with_followup, 1);

    assert_eq!(
        plan_followup_turn(&unit, true),
        FollowupTurn::Followup("tell me more")
    );
    assert_eq!(
        plan_followup_turn(&unit, false),
        FollowupTurn::Followup("tell me more")
    );
}

#[test]
fn coercion_variants_supersede_coercion_by_ordinal_modulo() {
    let mut alternating = prompt();
    alternating.coercion_variants = variants();

    for (ordinal, expected) in [(0, "a"), (1, "b"), (2, "a"), (23, "b")] {
        let unit = unit_with(alternating.clone(), ordinal);
        match plan_followup_turn(&unit, true) {
            FollowupTurn::Coercion { text, variant_id } => {
                assert_eq!(variant_id, Some(expected));
                assert_eq!(text, format!("text {expected}"));
            }
            other => panic!("expected coercion, got {other:?}"),
        }
    }
}

#[test]
fn coercion_variants_do_not_affect_non_refusal_escalation() {
    let mut alternating = prompt();
    alternating.coercion_variants = variants();
    let unit = unit_with(alternating, 0);

    assert_eq!(
        plan_followup_turn(&unit, false),
        FollowupTurn::Escalation("escalate")
    );
}
