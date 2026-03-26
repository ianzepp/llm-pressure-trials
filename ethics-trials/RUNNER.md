# Runner Architecture

## Goal

The `ethics-trials` runner is the core execution engine for the project.

It is responsible for turning scenario and experiment definitions into concrete trial executions, running those trials against selected models, grading and analyzing results, and writing durable artifacts.

The runner is the shared backend for:

- the CLI
- the TUI
- automation
- LLM-driven execution
- replay and reporting workflows

## Design Rule

There must be one runner with one set of semantics.

The TUI and CLI may differ in:

- input method
- progress rendering
- result presentation

They must not differ in:

- scenario loading
- experiment expansion
- model execution
- concurrency behavior
- grading behavior
- artifact persistence

## Responsibilities

The runner must own:

- loading definitions
- validating inputs
- expanding scenarios and experiments into executable trial units
- resolving model targets
- scheduling parallel work
- calling providers
- normalizing responses
- grading responses
- emitting structured events
- writing artifacts

The runner must not depend on:

- TUI widgets
- CLI formatting
- ad hoc shell orchestration

## Core Objects

### Scenario Definition

A scenario definition describes the behavioral situation under test.

The runner should treat this as static project input.

In the preferred layout, a scenario lives inside a named study directory such as:

- `studies/simple-mercy/scenario.yml`

### Experiment Definition

An experiment definition describes how one or more scenarios are sequenced and compared.

Examples:

- one-shot baseline
- A/B comparison
- ABAB sequence
- output-format sweep
- framing sweep

### Model Target

A model target describes one executable model choice.

It should include:

- provider
- model id
- human label
- cost metadata when available

### Trial Unit

A trial unit is the smallest executable unit of work.

It should include:

- run id
- scenario id
- experiment id if present
- condition id
- condition label such as `A1`, `B1`, or `json-variant`
- model target
- prompt or transcript payload
- output format
- repetition index

This is the object the scheduler should execute.

Trial units should carry study identity explicitly so artifacts and reports can
remain scoped to a single named study package.

### Trial Result

A trial result is the full outcome of one executed trial unit.

It should include:

- execution metadata
- raw response
- normalized response
- deterministic checks
- extracted signals
- rubric output
- cost/tokens/latency

## Runner Phases

The runner should be structured as explicit phases.

### Phase 1: Load

Load:

- scenarios
- experiments
- rubrics
- model registry
- provider config
- runner defaults

This phase should be deterministic and side-effect free.

### Phase 2: Validate

Validate:

- schema integrity
- reference integrity
- required fields
- selected scenario or experiment existence
- model target validity

Validation failures should stop the run before any provider call is made.

### Phase 3: Expand

Expand definitions into concrete trial units.

Examples:

- a single scenario plus three models becomes three trial units
- one ABAB experiment plus four models becomes sixteen trial units
- one format sweep plus five models becomes `conditions × models` trial units

Expansion is where the research design becomes execution work.

This phase must be explicit and inspectable.

### Phase 4: Schedule

The scheduler decides:

- what runs now
- what waits
- how provider-specific concurrency is bounded
- how keys are leased

Scheduling should operate on trial units, not on raw YAML.

### Phase 5: Execute

For each scheduled trial unit:

- assemble final prompt or transcript
- call the selected model
- capture raw request/response metadata
- emit execution events

Execution should be isolated enough that failures in one unit do not poison the whole run.

### Phase 6: Normalize

Normalize the raw model response into a structured form.

Examples:

- parse JSON
- split refusal from explanation
- extract decision fields
- normalize markdown blocks

Normalization should always preserve the original raw response.

### Phase 7: Grade

Apply deterministic checks first.

Examples:

- valid format
- required fields present
- forbidden fields absent
- refusal markers present or absent
- actionable harmful content detected or absent

Then apply extracted behavioral signals.

Then optionally apply rubric interpretation.

### Phase 8: Persist

Write:

- per-trial artifacts
- run summary artifacts
- queryable storage rows

Writes should happen incrementally so partial runs remain inspectable.

### Phase 9: Compare

When an experiment requires comparison, compute derived outputs.

Examples:

- A1 vs B1 delta
- A1 vs A2 stability
- B1 vs B2 persistence
- cross-model comparison for the same condition

Comparison should be a runner phase, not just a report-only concern.

## Parallel Execution

Parallelism is a core runner concern.

The runner must support:

- multi-model parallelism
- bounded concurrency
- provider-aware key pools
- per-key leasing

The concurrency engine must work identically whether the caller is:

- CLI
- TUI
- automation
- replay mode where live calls are skipped

### Concurrency Rule

The runner must not hide parallelism behind the TUI.

If a command is run via CLI in non-interactive mode, it should still be able to use the same bounded parallel execution engine.

### Key Leasing Rule

Key leasing must be real, not approximate.

If the system claims:

- N keys
- M leases per key

then no individual key may exceed M active leases.

This must be enforced structurally by the runner’s concurrency layer.

## Event Stream

The runner should emit a structured event stream.

This allows:

- TUI rendering
- CLI progress output
- logs
- future tracing integrations

### Example Event Types

- run started
- run validated
- trial queued
- trial started
- provider lease acquired
- provider lease released
- response received
- normalization complete
- grading complete
- artifact written
- trial failed
- trial finished
- comparison finished
- run finished

The event stream should be runner-native, not UI-native.

## Error Model

Errors should be separated into classes.

Examples:

- validation error
- configuration error
- provider error
- lease acquisition timeout
- normalization error
- grading error
- artifact persistence error

The runner should:

- fail fast on configuration and validation issues
- isolate per-trial execution errors where possible
- continue running sibling trials when one trial fails

## Artifact Model

The runner should write artifacts at multiple levels.

Artifacts should default to the study directory that owns the definitions.

For example:

- `studies/simple-mercy/experiments/...`
- `studies/simple-mercy/rubrics/...`
- `studies/simple-mercy/runs/...`
- `studies/simple-mercy/results/...`

### Trial Artifact

One file or record per trial unit.

Must include:

- trial identity
- scenario identity
- model identity
- prompt payload
- raw response
- normalized response
- deterministic checks
- behavioral signals
- rubric output
- tokens
- latency
- timestamp

### Run Artifact

One summary for the run.

Should include:

- selected scenario or experiment
- selected models
- total trial count
- pass/fail/error counts
- cost and token totals
- latency summaries
- artifact paths

### Derived Comparison Artifact

For experiments that compare conditions, write explicit delta outputs.

Examples:

- ABAB deltas
- model-to-model differences
- variant sensitivity summaries

## Study-Scoped Resolution

The runner should support a study-scoped lookup model.

Examples:

- load one study by id such as `simple-mercy`
- resolve its `scenario.yml`
- resolve experiments from `studies/simple-mercy/experiments/`
- resolve rubrics from `studies/simple-mercy/rubrics/`
- write run artifacts into `studies/simple-mercy/runs/` or `results/`

This keeps definitions and outputs co-located and avoids forcing the user to
mentally join scattered directories.

Global config should remain global:

- provider config
- model registry
- shared runner defaults

Study-specific behavioral assets should remain local to the study package.

## Replay Mode

Replay should reuse stored trial artifacts when possible.

Replay must support:

- re-grading old outputs with new deterministic rules
- re-running signal extraction
- re-running rubric interpretation
- reporting deltas between old and new grading results

Replay should not require fresh model execution unless explicitly requested.

## Runner API Shape

The internal API should be narrow and composable.

Examples of useful entrypoints:

- `run_scenario(...)`
- `run_experiment(...)`
- `replay_run(...)`
- `report_run(...)`

Internally, those should all pass through the same expansion, scheduling, grading, and persistence layers.

## Suggested Internal Modules

```text
src/ethics_trials/
├── runner/
│   ├── mod.py
│   ├── load.py
│   ├── validate.py
│   ├── expand.py
│   ├── schedule.py
│   ├── execute.py
│   ├── normalize.py
│   ├── grade.py
│   ├── compare.py
│   ├── persist.py
│   └── events.py
```

The exact file split can change, but these concerns should remain distinct.

## Initial V1 Runner Scope

The first useful runner version only needs to support:

- loading study packages from YAML
- selecting models
- expanding one scenario into `scenario × models`
- bounded parallel execution
- raw response capture
- simple normalization
- simple deterministic grading
- artifact writing
- event emission for CLI/TUI consumption

ABAB expansion can come immediately after that foundation exists.

## Phase Order Recommendation

Implement in this order:

1. load + validate
2. trial-unit expansion
3. execution engine
4. artifact writing
5. event stream
6. bounded parallel scheduling
7. normalization
8. deterministic grading
9. ABAB comparison layer
10. replay

That order preserves momentum without corrupting the architecture.

## Non-Goals For V1

- every possible experiment type
- distributed execution
- complex dynamic scheduling
- deep judge-model orchestration
- visualization-heavy comparison logic

The runner should first become correct, inspectable, and reusable.

## Standard For Success

The runner is good enough when:

- CLI and TUI can both invoke it cleanly
- one scenario can be run across multiple models in parallel
- artifacts are written incrementally and do not overwrite one another
- failures are isolated per trial
- replay can reuse stored outputs later
- ABAB and other multi-condition experiments can be added without redesigning the core
