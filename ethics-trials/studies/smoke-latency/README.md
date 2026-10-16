# Smoke Latency

`smoke-latency` is a tiny low-token study package for scheduler, transport, and
parallelism testing.

It is not intended to answer substantive behavioral questions. Its purpose is
to let the runner exercise:

- rapid request launch
- paced backpressure
- per-model sequencing
- artifact writing
- SQLite indexing
- JSONL event streaming

with minimal prompt and completion cost.

Current contents:

- one scenario
- one ABAB experiment
- no rubric files yet
- run artifacts under `runs/`
- a study-local `sqlite.db`

## Cheap Paid Cohort

The repository now includes a curated paid-model cohort in:

- `cheap-paid-models.txt`

This list is intended for scale-up runs where free models are a bad fit because
OpenRouter often throttles them more aggressively.

The current cohort favors:

- low cached prompt and completion pricing
- small or mid-sized instruct models
- multiple providers instead of one-provider concentration
- plain text and JSON-following models suitable for the `READY` smoke prompt

It intentionally excludes:

- `:free` models
- routing aliases such as `openrouter/auto`
- obviously non-smoke targets such as image/audio/search models

## Recommended Run Path

Dry-run the full paid cohort:

```bash
bash scripts/run_smoke_latency_paid_cohort.sh --dry-run --json
```

Launch the cohort live with JSONL events:

```bash
bash scripts/run_smoke_latency_paid_cohort.sh --jsonl
```

If you want to tune the scheduler without editing the script, pass overrides at
the end. The last flag wins with Clap, so this works:

```bash
bash scripts/run_smoke_latency_paid_cohort.sh \
  --high-water 64 \
  --low-water 32 \
  --launch-interval-ms 50 \
  --jsonl
```

## Cohort Notes

The cohort file stores cached per-token pricing from `.cache/openrouter/models.json`.

That means:

- it is good enough for near-term repo-local planning
- it is not a contractual price lock
- rerunning `cargo run -- sync` may change the relative ordering later
