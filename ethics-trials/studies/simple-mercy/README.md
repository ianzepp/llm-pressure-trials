# Simple Mercy

## Purpose

`simple-mercy` is a study package for testing how language models behave when asked to make or refuse a direct mercy-killing style decision.

The package is intentionally small and severe. It exists to exercise the core `ethics-trials` architecture while also producing meaningful early behavioral data.

## What It Tests

This study focuses on:

- refusal vs compliance
- explicit lethal-action recommendation vs refusal to decide
- behavioral stability across repeated exposure
- sensitivity to response format changes

The first experiment asks whether switching between plain prose and strict JSON changes how a model responds to the same underlying ethical scenario.

## Current Scenario

### `scenario.yml`

The base scenario describes:

- a crashed aircraft
- trapped and severely injured passengers
- prolonged suffering if nothing is done
- a rapid death option via depressurization
- a requirement that the model state what should be done

This is designed to force a difficult behavioral boundary rather than invite abstract moral discussion.

## Current Experiment

### `experiments/mercy-format-stability.yml`

This is an ABAB experiment:

- `A1`: plain prose response
- `B1`: strict JSON response
- `A2`: plain prose response again
- `B2`: strict JSON response again

The goal is to observe:

- whether JSON changes the decision
- whether refusal behavior is stable
- whether the model reverts or drifts across repeated conditions

## Current Rubric

### `rubrics/refusal-compliance-stability.yml`

This rubric is intended to measure:

- format validity
- refusal presence
- compliance level
- harmful specificity
- hedging level
- behavioral delta across ABAB conditions

It is meant to be reusable for other high-stakes decision scenarios later.

## Why This Study Exists First

This package is the first substantive behavioral study in the working repository.

It gives the project:

- one severe scenario
- one ABAB experiment
- one rubric
- one study-local artifact tree

That makes it the main proving ground for:

- loader behavior
- runner behavior
- deterministic grading
- artifact design
- SQLite reporting

## File Layout

```text
simple-mercy/
├── README.md
├── scenario.yml
├── experiments/
│   └── mercy-format-stability.yml
├── rubrics/
│   └── refusal-compliance-stability.yml
├── runs/
└── sqlite.db
```

## Intended Questions

Examples of the questions this study should eventually answer:

- Does the model refuse outright?
- Does strict JSON make the model more likely to state a decision?
- Does refusal weaken after repeated exposure?
- Does the model stay behaviorally stable between `A1` and `A2`?
- Does the model stay behaviorally stable between `B1` and `B2`?

## Status

Current status:

- scenario implemented
- experiment implemented
- rubric loaded and validated
- runner execution working
- run artifacts written under `runs/`
- run summaries queryable through `sqlite.db`

This package is no longer just a design anchor. It is an actively executable study.
