# TUI Direction

## Goal

`ethics-trials` should include a terminal UI that makes behavioral experiments easy to configure, launch, observe, and inspect without requiring hand-built CLI invocations.

The TUI is not decorative. It should be an operational surface for running scenario-based experiments across many models in parallel.

## Core User Flow

The first useful TUI flow should be:

1. load the TUI
2. browse or search predefined scenarios from YAML files in the project
3. select one scenario or experiment definition
4. select which LLMs should participate in the run
5. launch the run
6. execute the selected LLMs in parallel
7. observe live run progress and per-model status
8. inspect completed results and behavioral deltas

The exact mechanics can be refined later, but this flow should anchor the design.

## Initial Product Requirement

At minimum, the TUI must support:

- loading scenario definitions from project YAML
- presenting a selectable scenario list
- presenting a selectable model list
- launching a run across multiple selected models
- running those selected models in parallel
- showing live progress while the run is active
- preserving artifacts for later review

## What We Learned From `gofolio-cli`

The sibling [`gofolio-cli`](/Users/ianzepp/github/ianzepp/gofolio-cli) project is a useful reference for eval TUI direction, but not a drop-in design.

Useful ideas to carry forward:

- terminal-first eval dashboard
- live per-case progress rows
- detail modal per finished case
- fixture-backed and live execution modes
- replay support
- suite/case/model separation
- key-pool driven bounded concurrency

Important issues not to repeat:

- do not tie parallel execution to whether the TUI is enabled
- do not let per-case artifacts overwrite one another in multi-model runs
- do not let key-pool accounting claim per-key limits that are not actually enforced
- do not let docs drift away from the implemented command surface

## TUI Responsibilities

The TUI should be responsible for orchestration and visibility, not hidden business logic.

It should:

- load scenario and experiment definitions
- load available model targets
- allow selection and launch
- subscribe to runner events
- render progress, status, and details
- open completed run summaries and case-level artifacts

It should not:

- contain grading logic
- define scenario semantics
- own storage rules
- duplicate runner logic that should live in the execution engine

## Proposed Screens

### 1. Scenario Picker

Purpose:

- browse available scenarios and experiments from YAML
- filter by family, tags, risk level, or output format

Desired information:

- scenario id
- scenario title
- family
- tags
- risk level
- available variants or experiment modes

### 2. Model Picker

Purpose:

- choose one or more LLMs for the run

Desired information:

- provider
- model id
- human label
- expected cost tier
- whether keys are currently available

The model picker should support multi-select cleanly.

### 3. Launch / Run Summary Panel

Purpose:

- confirm what is about to run
- show scenario, selected models, execution mode, and concurrency

This can remain simple in the first version.

### 4. Live Run Dashboard

Purpose:

- show active parallel execution
- render one row per model or per model-condition unit
- show running, passed, failed, errored, or cancelled states
- show elapsed time and queue state

Useful live signals:

- model id
- condition label
- current phase
- latency so far
- tokens so far
- key-pool usage

### 5. Detail View

Purpose:

- inspect one completed result in depth

Should eventually show:

- raw response
- normalized response
- deterministic checks
- behavioral signals
- rubric results
- metadata

### 6. Run History View

Purpose:

- reopen prior runs
- inspect artifacts without rerunning
- compare runs at a high level

This can come after the first launchable TUI exists.

## Parallel Execution Requirements

Parallelism is a core requirement, not an enhancement.

The runner behind the TUI should:

- execute selected models concurrently
- use bounded concurrency
- support provider-specific key pools
- support per-key leasing
- expose live events back to the TUI

The TUI should display:

- key count
- max concurrent leases
- active leases
- queued work

The runner and TUI must be decoupled enough that:

- runs can execute with or without the TUI
- batch and CI execution use the same parallel engine
- the TUI is only one frontend over the runner

## Event Model

The TUI should consume structured runner events.

Examples:

- run started
- scenario selected
- model started
- model finished
- condition started
- condition finished
- artifact written
- run finished
- run failed

This event model should be explicit and stable. It will help keep the TUI thin.

## Artifact Expectations

A TUI-driven run must still produce durable artifacts.

Each launched run should persist:

- run metadata
- selected scenario or experiment id
- selected model set
- execution timestamps
- per-model result files
- summary data

Do not collapse multi-model runs into a single artifact name that can overwrite prior outputs.

## Initial Scope

The first TUI version does not need to solve everything.

It only needs to solve:

- scenario selection
- model selection
- parallel launch
- live progress display
- completed result inspection

That is enough to make the system operational.

## Non-Goals For V1

- full experiment editing inside the TUI
- advanced rubric authoring in the UI
- complex cross-run comparison dashboards
- large-scale visualization
- every possible workflow from day one

## Recommended Design Constraint

Build the runner first, then let the TUI sit on top of its event stream.

If the TUI becomes the place where execution logic lives, the system will get brittle quickly.

## Near-Term Implementation Target

The first `ethics-trials` TUI should let a user:

- start the app
- select a predefined scenario from YAML
- select multiple LLMs
- press run
- watch those models execute in parallel
- inspect each result when complete

That is the bar for the first real version.
