# CLI

## Current Surface

`ethics-trials` currently exposes five commands:

- `list`
- `validate`
- `run`
- `report`
- `sync`

There is no implemented `tui`, `replay`, `get`, or `list runs` command yet.

The CLI is the only production frontend today.

## Command Reference

### `ethics-trials list`

Lists one class of project objects.

Supported targets:

- `studies`
- `scenarios`
- `experiments`
- `rubrics`
- `models`

Current behavior:

- `list studies` discovers study directories under `studies/`
- `list scenarios --study <id>` returns the single scenario id for that study
- `list experiments --study <id>` returns experiment ids from `studies/<id>/experiments/`
- `list rubrics --study <id>` returns rubric ids from `studies/<id>/rubrics/`
- `list models` prefers `.cache/openrouter/models.json` and falls back to the small built-in model list

Options:

- `--study <STUDY>`
- `--pattern <PATTERN>`
- `--providers <PROVIDERS>`
- `--json`

### `ethics-trials validate`

Validates study definitions and reference integrity.

Current target set:

- `validate all`

Current behavior:

- with `--study <id>`, validates one study
- without `--study`, validates every discovered study
- prints one `study: ok` line per validated study unless `--json` is used

Options:

- `--study <STUDY>`
- `--json`

### `ethics-trials run`

Runs one scenario or one experiment within a study.

Required:

- `--study <STUDY>`
- exactly one of `--scenario <SCENARIO>` or `--experiment <EXPERIMENT>`

Model selection:

- one or more `--model <MODEL>`
- or `--pattern <PATTERN>`
- optional `--providers <PROVIDERS>`
- optional `--limit <LIMIT>`

Execution options:

- `--launch-interval-ms <N>`
- `--high-water <N>`
- `--low-water <N>`
- `--continue-on-error`
- `--dry-run`
- `--json`
- `--jsonl`

Current behavior:

- `--dry-run` expands trial units without calling the provider
- live runs use the shared runner and write artifacts under `studies/<study>/runs/<run-id>/`
- live runs update the study-local SQLite index at `studies/<study>/sqlite.db`
- `--jsonl` streams runner events during live execution only
- `--json` prints the dry-run summary or final live-run summary

Constraint rules enforced today:

- `--scenario` and `--experiment` are mutually exclusive
- `--model` and `--pattern` are mutually exclusive
- `--json` and `--jsonl` are mutually exclusive
- `--jsonl` is rejected for dry runs
- `--low-water` must be less than `--high-water`

### `ethics-trials report`

Builds a summary from the study-local SQLite index.

Required:

- `--study <STUDY>`

Optional:

- `--run-id <RUN_ID>`
- `--json`

Current behavior:

- with `--run-id`, reports that run
- without `--run-id`, reports the latest run in the study
- aggregate output includes trial counts, format-valid counts, refusal counts, compliance buckets, decision buckets, average hedging, average harmful specificity, and comparison totals
- per-model output includes the same counters scoped to one model id

### `ethics-trials sync`

Refreshes the cached OpenRouter model catalog.

Options:

- `--json`

Current behavior:

- fetches the model list through the OpenRouter-compatible client
- writes `.cache/openrouter/models.json`
- prints cache path, model count, and sync timestamp unless `--json` is used

## Output Modes

The current CLI supports two machine-usable output styles:

- `--json` for structured summaries from `list`, `validate`, `run`, `report`, and `sync`
- `--jsonl` for live `run` event streaming

Human-readable text output remains the default.

## Runner Relationship

The CLI does not implement its own execution logic.

Today it shares:

- study loading
- validation
- experiment and scenario expansion
- provider execution
- deterministic grading
- artifact writing
- SQLite indexing

with the same runner modules a future TUI would need to call.

## Implemented vs Planned

Implemented now:

- CLI-first workflow
- study-scoped lookup
- dry-run expansion
- live execution
- bounded scheduler controls
- JSON and JSONL outputs
- run reporting from SQLite
- model catalog syncing

Planned but not implemented:

- TUI command
- replay
- direct artifact retrieval
- run listing command
- provider-aware key leasing beyond the current scheduler watermarks
