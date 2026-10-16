# Project Status

## Purpose

`ethics-trials` is a Rust research harness for running study-scoped LLM behavior experiments, storing durable artifacts, and summarizing results from a study-local SQLite index.

The project is no longer at the pure planning stage. The current repository contains a working CLI, runner, study packages, artifact storage, and reporting.

## Current Implementation

Implemented now:

- Rust CLI with `list`, `validate`, `run`, `report`, and `sync`
- study-local YAML loading from `studies/<study>/`
- study validation for scenario, experiment, condition, variant, and rubric references
- scenario and experiment expansion into executable trial units
- live execution through an OpenRouter-compatible OpenAI client
- bounded scheduling controls via launch interval and high/low watermarks
- deterministic normalization and grading
- run artifact writing under `studies/<study>/runs/<run-id>/`
- study-local SQLite indexing in `studies/<study>/sqlite.db`
- comparison generation for ABAB-style experiments
- JSON output and JSONL event streaming
- targeted correctness tests and a hygiene ratchet

Implemented studies:

- `simple-mercy`
- `smoke-latency`

## Current Repository Shape

```text
ethics-trials/
├── README.md
├── CLI.md
├── PROJECT.md
├── RUNNER.md
├── SCENARIOS.md
├── studies/
│   ├── simple-mercy/
│   │   ├── README.md
│   │   ├── scenario.yml
│   │   ├── experiments/
│   │   ├── rubrics/
│   │   ├── runs/
│   │   └── sqlite.db
│   └── smoke-latency/
│       ├── README.md
│       ├── scenario.yml
│       ├── experiments/
│       ├── runs/
│       └── sqlite.db
├── schemas/
├── src/
└── tests/
```

## Data Model In Practice

The current implementation revolves around these concrete objects:

- `ScenarioDefinition`
- `ExperimentDefinition`
- `TrialUnit`
- `TrialResult`
- `FailedTrial`
- `ComparisonResult`
- `RunMetadata`
- `RunSummary`

The system already supports:

- one scenario per study
- multiple experiments per study
- multiple rubrics per study
- multiple models per run
- per-condition comparison output for experiments

## Analysis Surface

The current report path can summarize one stored run by:

- aggregate counts across the run
- per-model counts
- compliance buckets
- decision buckets
- average hedging
- average harmful specificity
- comparison totals and shift counts

This is still intentionally narrow. It is enough to inspect early behavioral differences, but not yet enough for broad rubric-driven interpretation.

## What Is Still Missing

Not implemented yet:

- replay
- TUI
- run listing command
- direct artifact retrieval command
- richer rubric execution beyond the current deterministic grader
- provider-key lease management
- cost accounting and pricing-aware reporting
- cross-study reporting

## Design Direction

The direction remains the same, but it should be stated in terms of what exists:

- one shared runner
- study-local definitions and artifacts
- durable JSON artifacts plus SQLite indexing
- deterministic first-pass grading before any judge-model layer
- CLI-first workflows with a future TUI on the same runner

## Reality Check

Older docs in this repo described Python as the likely implementation language and treated bounded parallelism and the runner itself as future work.

Those statements are obsolete.

The current codebase is Rust, the runner exists, and bounded scheduling controls are already implemented.
