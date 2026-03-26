# Ethics Trials Project Plan

## Purpose

`ethics-trials` is a behavioral research harness for studying how language models respond to ethically extreme and high-pressure scenarios under controlled experimental conditions.

The project exists to make behavioral comparison rigorous, repeatable, and queryable.

## Project Shape

This should be built as a research system with five first-class layers:

1. `scenario definitions`
2. `experiment definitions`
3. `execution engine`
4. `grading and analysis`
5. `artifacts and storage`

The system should be able to express a scenario once, run it across many models and condition sequences, and produce artifacts that support later analysis without rerunning the model.

## Core Concepts

### Scenario

A scenario is the behavioral situation being tested.

Examples:

- user requests harmful instructions
- user applies emotional coercion
- user invokes authority pressure
- user frames the request as urgent or exceptional
- user shifts responsibility to the model

A scenario should define:

- the baseline prompt frame
- perturbation hooks
- expected output format options
- scenario tags
- risk level
- analysis rubrics to apply

### Condition

A condition is a particular presentation of a scenario.

Examples:

- neutral framing
- coercive framing
- apologetic framing
- strict JSON output
- transcript with prior refusal

Conditions are what allow structured experimentation rather than one-off prompting.

### Experiment

An experiment defines how conditions are sequenced and compared.

Examples:

- A/B
- ABAB
- repeated baseline
- format sweep
- model cohort sweep

The experiment layer should own:

- sequence structure
- repetition count
- model selection
- randomization or ordering rules
- analysis grouping keys

### Run

A run is one execution of one experiment across one or more models.

It should produce durable artifacts and enough metadata to support replay, regrading, and cohort analysis.

### Behavioral Signal

A behavioral signal is an extracted feature from a response.

Examples:

- refusal strength
- compliance level
- harmful specificity
- hedging intensity
- moral framing
- redirection quality
- authority deference
- emotional mirroring
- escalation to human oversight

Signals should be separate from raw output and separate from final interpretation.

## Proposed Directory Layout

The project should default to a scenario-package layout rather than scattering
definitions across top-level folders.

Each major scenario family or study should live in one named directory that
contains its definitions and artifacts together.

```text
ethics-trials/
├── BRIEF.md
├── PROJECT.md
├── README.md
├── pyproject.toml
├── config/
│   ├── models.yml
│   ├── providers.yml
│   └── defaults.yml
├── schemas/
│   ├── scenario.schema.yml
│   ├── experiment.schema.yml
│   └── rubric.schema.yml
├── studies/
│   ├── simple-mercy/
│   │   ├── scenario.yml
│   │   ├── experiments/
│   │   │   └── mercy-format-stability.yml
│   │   ├── rubrics/
│   │   │   └── refusal-compliance-stability.yml
│   │   ├── fixtures/
│   │   ├── runs/
│   │   └── results/
│   ├── resonance/
│   └── dynamics/
├── src/ethics_trials/
│   ├── cli.py
│   ├── types.py
│   ├── loader.py
│   ├── runner.py
│   ├── orchestrators/
│   ├── providers/
│   ├── graders/
│   ├── analyzers/
│   ├── storage/
│   └── reporting/
└── tests/
```

## Runner Architecture

The runner should be structured around explicit phases.

### Phase 1: Load

Load:

- scenario definitions
- experiment definitions
- model registry
- default execution settings
- rubric definitions

### Phase 2: Expand

Compile experiment definitions into concrete trial units.

A trial unit should represent:

- model
- scenario
- condition label
- condition content
- output format
- repetition index
- experiment ID
- ordering metadata

This expansion layer is critical. It is what turns research intent into executable work.

### Phase 3: Execute

For each trial unit:

- construct the final prompt or transcript
- call the model
- record raw request and raw response
- capture timing and token usage

The execution layer should be pure enough that it can later support replay and provider substitution cleanly.

### Phase 4: Normalize

Normalize raw responses into machine-comparable forms.

Examples:

- parse JSON when JSON is requested
- separate refusal preamble from actionable content
- extract explicit yes/no decisions
- normalize markdown and answer blocks

Normalization should not overwrite raw output. It should create derived structured data.

### Phase 5: Grade

Grading should be multi-layered.

#### Deterministic checks

- format adherence
- JSON validity
- required field presence
- refusal present or absent
- presence of actionable harmful steps
- verbosity or length constraints

#### Signal extraction

- compliance
- refusal strength
- redirection
- hedging
- policy invocation
- deference markers
- justification style

#### Rubric interpretation

Optional judge-model or rule-based interpretation for:

- ethical stability
- framing sensitivity
- behavioral shift
- intervention persistence

### Phase 6: Store

Persist:

- raw artifacts
- normalized artifacts
- deterministic grading output
- extracted behavioral signals
- rubric scores
- run metadata

Storage must support later comparison without rerunning models.

### Phase 7: Compare

The system should be able to compare:

- A1 vs B1
- A1 vs A2
- B1 vs B2
- model vs model
- family vs family
- format vs format
- scenario family vs scenario family

## Study Package Layout

The preferred organizational unit should be a named study directory.

For example:

```text
studies/simple-mercy/
├── scenario.yml
├── experiments/
├── rubrics/
├── fixtures/
├── runs/
└── results/
```

This gives the project:

- locality of context
- easier migration and archival
- simpler human browsing
- cleaner runner resolution
- per-study ownership of experiments and artifacts

The runner should still support shared/global config for models and providers,
but behavioral definitions and their resulting artifacts should live together
whenever practical.

## Artifact Model

Each executed trial should produce one primary artifact record.

That record should include:

- run id
- experiment id
- scenario id
- condition id
- sequence label such as `A1` or `B2`
- model id
- provider id
- prompt artifact
- raw response artifact
- normalized response artifact
- deterministic checks
- extracted behavioral signals
- rubric scores
- tokens
- latency
- timestamp

ABAB analysis should produce derived comparison artifacts as well.

Examples:

- delta between A1 and B1
- delta between A1 and A2
- persistence score between B1 and B2
- stability score across full sequence

## Data and Analysis Priorities

The first version should optimize for the following questions:

- Does the model refuse?
- Does refusal remain stable across repeated exposure?
- Does framing alter compliance?
- Does format alter compliance?
- Does a prior response condition later behavior?
- Which model families are most stable across ABAB sequences?

This means early analysis should privilege behavioral deltas over broad dashboards.

## Technology Direction

Python is a reasonable default for the first version because:

- `ai-trials` already provides nearby provider/config patterns
- YAML and SQLite tooling is straightforward
- iteration speed is high for research software

The project should use:

- YAML for human-authored scenarios and experiments
- SQLite for queryable local analysis
- JSONL or per-trial JSON for raw artifact persistence

The initial UI can be CLI-first. A richer terminal UI can come later once the artifact model is stable.

## Suggested Phases

### Phase 0: Foundations

- create package skeleton
- add config loading
- add provider abstraction
- add model registry
- add artifact writer

### Phase 1: Definitions

- define scenario schema
- define experiment schema
- define rubric schema
- add loaders and validation

### Phase 2: Single-turn Behavioral Runs

- run one scenario condition against many models
- capture raw and normalized outputs
- implement deterministic checks

### Phase 3: ABAB Engine

- implement sequence expansion
- implement condition labeling
- implement comparison artifacts
- implement stability and drift metrics

### Phase 4: Behavioral Signal Extraction

- refusal/compliance detection
- harmful-specificity detection
- hedging and deference extraction
- redirection quality markers

### Phase 5: Judge-Assisted Analysis

- rubric-based interpretation
- pairwise behavioral delta judgments
- cohort summaries

### Phase 6: Reporting

- CLI summaries
- query commands
- exportable reports
- later, TUI if warranted

## What To Borrow

Borrow from `ai-trials`:

- provider registry concepts
- model config shape
- lightweight result persistence instincts

Borrow from `faber-trials`:

- richer grading philosophy
- transition-oriented thinking
- broader experimental ambition
- metadata discipline

Do not inherit either project’s core task model directly.

## Failure Modes To Avoid

- becoming a bag of prompts without experimental structure
- overusing judge models before deterministic features exist
- collapsing behavioral analysis into one score
- losing raw evidence during normalization
- making the schema so abstract that writing scenarios becomes miserable
- optimizing for flashy reports before the data model is correct

## Standard For Progress

The project is moving in the right direction if:

- a scenario can be authored clearly
- an ABAB experiment can be declared without code changes
- runs produce reusable artifacts
- behavior can be compared across models and conditions
- conclusions can be traced back to raw evidence

That is the bar.
