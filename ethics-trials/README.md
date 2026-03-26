# Ethics Trials

`ethics-trials` is a Rust research harness for studying how language models behave in ethically extreme, adversarial, coercive, and high-stakes scenarios.

The current implementation is centered on:

- study-local YAML definitions
- one shared runner for CLI and future TUI use
- OpenRouter execution through an OpenAI-compatible client adapter
- cached OpenRouter model catalog under `.cache/`
- durable JSON artifacts
- per-study SQLite indexing

## Current Status

Implemented today:

- Rust CLI scaffold
- study loading and validation
- experiment expansion into concrete trial units
- live serial execution against OpenRouter
- deterministic normalization and grading for `simple-mercy`
- study-local artifact writing under `studies/<study>/runs/<run-id>/`
- study-local SQLite index at `studies/<study>/sqlite.db`
- OpenRouter model catalog sync into `.cache/openrouter/models.json`
- run reporting from SQLite
- baseline unit coverage for runner/provider correctness edges
- hygiene ratchet coverage for panic/error-dropping debt patterns

Not implemented yet:

- bounded parallel execution
- replay
- TUI
- richer rubric-driven grading beyond the current deterministic first pass

## Requirements

- Rust toolchain
- `OPENROUTER_API_KEY` set in the shell environment

Example:

```bash
export OPENROUTER_API_KEY="your-key-here"
```

## Build

```bash
cargo check
```

## Validate

```bash
cargo test
```

```bash
cargo clippy --all-targets -- -D warnings
```

## Project Layout

```text
ethics-trials/
├── .cache/
│   └── openrouter/
│       └── models.json
├── studies/
│   └── simple-mercy/
│       ├── scenario.yml
│       ├── experiments/
│       ├── rubrics/
│       ├── runs/
│       └── sqlite.db
├── schemas/
├── src/
└── README.md
```

## Command Examples

### Validate definitions

Validate every study:

```bash
cargo run -- validate all
```

Validate one study:

```bash
cargo run -- validate all --study simple-mercy
```

### List project objects

List studies:

```bash
cargo run -- list studies
```

List studies as JSON:

```bash
cargo run -- list studies --json
```

List experiments for one study:

```bash
cargo run -- list experiments --study simple-mercy
```

List rubrics for one study:

```bash
cargo run -- list rubrics --study simple-mercy
```

List available model targets:

```bash
cargo run -- list models
```

List models as JSON:

```bash
cargo run -- list models --json
```

Filter cached models by full `provider/model` pattern:

```bash
cargo run -- list models --pattern 'openai/gpt-5*'
```

Filter by provider namespace:

```bash
cargo run -- list models --providers openai,anthropic
```

Combine provider and pattern filters:

```bash
cargo run -- list models \
  --providers openai \
  --pattern 'openai/gpt-5*'
```

### Sync the OpenRouter model catalog

Fetch the current OpenRouter model catalog into the local cache:

```bash
cargo run -- sync
```

JSON output:

```bash
cargo run -- sync --json
```

This writes:

```text
.cache/openrouter/models.json
```

`list models` prefers this cache when present and falls back to the small built-in list otherwise.

### Dry-run an experiment

Expand the `simple-mercy` ABAB experiment without calling a model:

```bash
cargo run -- run \
  --study simple-mercy \
  --experiment mercy-format-stability \
  --model openai/gpt-4o-mini \
  --dry-run
```

JSON output:

```bash
cargo run -- run \
  --study simple-mercy \
  --experiment mercy-format-stability \
  --model openai/gpt-4o-mini \
  --dry-run \
  --json
```

Dry-run across every cached model matching a `provider/model` pattern:

```bash
cargo run -- run \
  --study simple-mercy \
  --experiment mercy-format-stability \
  --pattern 'openai/*' \
  --dry-run
```

Limit the matched model set before expansion:

```bash
cargo run -- run \
  --study simple-mercy \
  --experiment mercy-format-stability \
  --pattern 'openai/*' \
  --limit 3 \
  --dry-run
```

Filter by provider namespace and pattern together:

```bash
cargo run -- run \
  --study simple-mercy \
  --experiment mercy-format-stability \
  --providers openai \
  --pattern 'openai/gpt-5*' \
  --limit 5 \
  --dry-run
```

Rules:

- `--model` and `--pattern` are mutually exclusive
- `--providers` filters the namespace before the slash in ids like `openai/gpt-5-mini`
- `--limit` truncates the selected model list before trial expansion

### Execute a live run

Run the `simple-mercy` ABAB experiment live against OpenRouter:

```bash
cargo run -- run \
  --study simple-mercy \
  --experiment mercy-format-stability \
  --model openai/gpt-4o-mini
```

This writes artifacts to:

```text
studies/simple-mercy/runs/<run-id>/
```

Files currently written:

- `run.json`
- `trial_units.json`
- `trial_results.jsonl`
- `comparisons.json`
- `summary.json`

It also updates:

```text
studies/simple-mercy/sqlite.db
```

### Report a run

Report the latest run in a study:

```bash
cargo run -- report --study simple-mercy
```

Report a specific run:

```bash
cargo run -- report \
  --study simple-mercy \
  --run-id run-1774549655939
```

JSON report output:

```bash
cargo run -- report --study simple-mercy --json
```

The report currently includes:

- aggregate results for the run
- per-`provider/model` breakdown
- format-valid counts
- refusal-present counts
- compliance buckets
- decision buckets
- average hedging and harmful-specificity
- ABAB comparison totals and shift counts

## Notes On Current Grading

The current grading layer is intentionally narrow and deterministic.

For `simple-mercy`, it currently tracks:

- strict JSON validity
- refusal marker detection
- coarse compliance level
- inferred decision
- hedging level
- harmful specificity

This is enough for a first behavioral pass, but it is not yet a full rubric engine.

## Example End-To-End Flow

```bash
# 1. Validate the study
cargo run -- validate all --study simple-mercy

# 2. Inspect the available experiment
cargo run -- list experiments --study simple-mercy

# 3. Sync the OpenRouter model catalog
cargo run -- sync

# 4. Dry-run the ABAB expansion
cargo run -- run \
  --study simple-mercy \
  --experiment mercy-format-stability \
  --providers openai \
  --pattern 'openai/gpt-5*' \
  --limit 3 \
  --dry-run

# 5. Execute the live run
cargo run -- run \
  --study simple-mercy \
  --experiment mercy-format-stability \
  --model openai/gpt-4o-mini

# 6. Summarize the latest run from SQLite
cargo run -- report --study simple-mercy
```

## Design Rule

There is one runner with one set of semantics.

The CLI is the current frontend. A future TUI should subscribe to the same runner behavior rather than reimplement execution logic.
