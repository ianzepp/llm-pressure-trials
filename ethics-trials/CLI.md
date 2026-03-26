# CLI Direction

## Goal

`ethics-trials` should expose a first-class CLI for launching, inspecting, replaying, and reporting behavioral experiments without requiring the terminal UI.

This CLI is required for:

- automation
- scripting
- CI
- reproducible batch runs
- direct use by another LLM or agent

The CLI is not a secondary fallback. It is one of the primary interfaces to the system.

## Design Rule

The CLI and the TUI must both sit on top of the same execution engine.

That means:

- same runner
- same scenario loading
- same model resolution
- same parallel execution
- same grading
- same artifact writing

The CLI must not be a separate implementation with different semantics.

## Core Principles

- explicit commands over hidden defaults
- machine-usable output where appropriate
- deterministic exit behavior
- stable argument surfaces
- compatibility with both human operators and LLM-driven invocation

## Primary Command Surface

The CLI should have a compact but capable surface.

### `ethics-trials tui`

Launch the interactive terminal UI.

Purpose:

- human-operated scenario selection
- model selection
- live run monitoring

### `ethics-trials run`

Run a scenario or experiment directly.

Purpose:

- batch execution
- scripted execution
- agent-driven execution

The `run` command should become the main automation entrypoint.

It should support study-scoped execution so a single named directory can own:

- scenario definition
- experiment definitions
- rubric definitions
- run artifacts
- result artifacts

### `ethics-trials replay`

Replay or re-grade a previous run using stored artifacts.

Purpose:

- rubric evolution
- grader evolution
- regression analysis

### `ethics-trials report`

Summarize a prior run or a set of runs.

Purpose:

- quick inspection
- aggregate behavioral reporting
- machine-readable summaries

### `ethics-trials get`

Open a stored run artifact or a single case result.

Purpose:

- direct artifact retrieval
- agent inspection of prior outputs

### `ethics-trials list`

List available project objects.

Subcommands should include:

- `list scenarios`
- `list experiments`
- `list models`
- `list rubrics`
- `list runs`

### `ethics-trials validate`

Validate project definitions.

Subcommands should include:

- `validate scenario <id>`
- `validate experiment <id>`
- `validate rubric <id>`
- `validate all`

## The `run` Command

This is the most important command for LLM use.

It should support:

- selecting a scenario directly
- selecting an experiment directly
- selecting one or more models
- selecting output format or condition variants
- selecting live or fixture-backed execution
- controlling concurrency
- controlling output formatting

### Minimum Inputs

At least one of:

- `--scenario <id>`
- `--experiment <id>`

And one of:

- `--model <id>`
- `--models <id1,id2,...>`
- `--models-file <path>`

### Useful Flags

Recommended flags:

- `--study <id>`
- `--scenario <id>`
- `--experiment <id>`
- `--model <id>`
- `--models <comma-separated>`
- `--models-file <path>`
- `--provider <id>`
- `--output-format <id>`
- `--variant <id>`
- `--parallel`
- `--max-parallel <n>`
- `--fixture <name>`
- `--live`
- `--output <path>`
- `--json`
- `--no-tui`
- `--dry-run`

The exact list can change, but this is the right class of interface.

### Study Resolution

For scalability, the CLI should support a study-first lookup model.

Examples:

- `ethics-trials run --study simple-mercy --scenario scenario`
- `ethics-trials run --study simple-mercy --experiment mercy-format-stability`
- `ethics-trials report --study simple-mercy --run-id ...`

This allows a study directory such as `studies/simple-mercy/` to contain all
related definitions and artifacts.

## Agent-Friendly Requirements

If another LLM is going to use this CLI, the CLI must be predictable.

That means:

- exit code `0` on success
- nonzero exit code on validation or execution failure
- optional JSON output for machine consumption
- commands that can discover project state before execution
- no need to scrape colorful human-only terminal output

Important agent-oriented commands:

- `ethics-trials list scenarios --json`
- `ethics-trials list experiments --json`
- `ethics-trials list models --json`
- `ethics-trials run ... --json`
- `ethics-trials report --run-id <id> --json`
- `ethics-trials get <path> --json`

## Output Modes

The CLI should support at least two output modes.

### Human Output

Readable terminal output for operators.

Includes:

- concise progress
- run summary
- failure details
- result location

### JSON Output

Structured output for automation and LLM use.

Should be available via `--json`.

This should expose:

- run id
- selected scenario or experiment
- selected models
- status
- output artifact path
- pass/fail or analysis summary
- error metadata when relevant

## Parallel Execution

The CLI must be able to launch parallel runs directly.

This is especially important for:

- large model sweeps
- repeated experiments
- agent-directed experimentation

Requirements:

- bounded concurrency
- provider-aware key pools
- per-key leasing
- progress reporting even in non-TUI mode

The critical design constraint is:

Parallel execution must not depend on the TUI being active.

The same parallel runner should work for:

- `ethics-trials run --json`
- `ethics-trials run --no-tui`
- `ethics-trials tui`

## Replay and Reporting

The CLI should make artifact reuse easy.

### Replay

Examples:

- replay an entire run
- replay using current graders
- compare original pass/fail vs replayed pass/fail

### Report

Examples:

- summarize one run
- summarize all runs for a scenario
- summarize behavior by model family
- summarize ABAB deltas

These commands should become the non-interactive analysis surface.

## Example Command Shapes

These are directional examples, not final syntax.

### Run One Scenario Across Multiple Models

```bash
ethics-trials run \
  --study simple-mercy \
  --scenario mercy/simple-mercy-json \
  --models openrouter/claude-haiku,openrouter/gpt-5-mini,openrouter/grok-3-mini \
  --parallel
```

### Run One Experiment

```bash
ethics-trials run \
  --study simple-mercy \
  --experiment abab/authority-mercy-sweep \
  --models-file config/model_sets/high-risk.yml \
  --parallel \
  --json
```

### List Available Scenarios

```bash
ethics-trials list scenarios --json
```

### Report A Prior Run

```bash
ethics-trials report --run-id run-2026-03-26-001 --json
```

### Replay A Prior Run

```bash
ethics-trials replay --run-id run-2026-03-26-001
```

## Recommended Internal Separation

The CLI layer should do only:

- argument parsing
- command dispatch
- output formatting

The runner layer should do:

- loading
- expansion
- execution
- grading
- artifact writing

This separation will keep the CLI stable and make it safe for agents to use.

## V1 CLI Priorities

The first useful version only needs:

- `tui`
- `run`
- `list scenarios`
- `list models`
- `report`
- `replay`

That is enough to support both human and LLM workflows.

## Standard For Success

The CLI is good enough when:

- a human can run a scenario sweep without touching code
- an LLM can discover scenarios and models, launch a run, and inspect results
- parallel execution works the same way with or without the TUI
- prior runs can be replayed and reported without rerunning models
