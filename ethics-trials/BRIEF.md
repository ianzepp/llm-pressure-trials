# Ethics Trials

## Goal

Build a research-oriented evaluation harness for studying the behavioral responses of LLMs to ethically extreme, adversarial, coercive, manipulative, and high-stakes scenarios.

This project is not primarily about factual accuracy, coding correctness, or benchmark-style task completion. Its core purpose is to measure how models behave under pressure, across framing changes, and across repeated intervention sequences such as ABAB trials.

## Core Question

How do different models change their behavior when scenario framing, output format, conversational structure, authority pressure, emotional tone, or prior interaction history is altered?

## Primary Use Cases

- Compare OpenRouter models on ethically difficult scenarios.
- Run ABAB-style intervention sequences to detect behavioral drift, persistence, and reversibility.
- Measure framing sensitivity across prompt variants.
- Test how output format constraints alter refusal, compliance, hedging, or redirection behavior.
- Study behavioral differences across model families, sizes, and providers.
- Capture not just whether a model refused or complied, but how it reasoned, redirected, escalated, or destabilized.

## What This Project Must Do

- Treat behavior as the thing being evaluated.
- Support structured scenario definitions with variants and controlled perturbations.
- Support experimental designs such as A/B comparisons, ABAB sequences, and repeated-trial cohorts.
- Store full artifacts for each run, including raw response, normalized response, condition metadata, and derived behavioral signals.
- Separate deterministic checks from rubric-based interpretation.
- Enable cross-model and cross-condition comparison after runs complete.
- Be designed from the start for research traceability and reproducibility.

## Initial Scope

The first version should focus on text-only experiments against remote chat models, especially OpenRouter-accessible models.

Initial scenario classes should include:

- ethical refusal and harmful-request handling
- authority pressure and coercion
- urgency and crisis framing
- emotional manipulation
- policy-boundary probing
- self-preservation or self-importance framing
- delegation and responsibility shifting
- ambiguity under high stakes

Initial output-format variants should include:

- plain prose
- bullet list
- strict JSON
- fixed schema answer blocks
- short-answer vs extended justification

## Initial Experimental Focus

The first serious experimental mode should be ABAB trials:

- A1: baseline condition
- B1: intervention condition
- A2: return to baseline
- B2: re-intervention

This should allow the harness to measure:

- compliance drift
- refusal stability
- intervention persistence
- behavioral reversion
- sensitivity to framing changes
- variance within the same model under repeated exposure

## Non-Goals

- building another generic prompt runner
- rebuilding a broad agent eval platform before the behavioral core exists
- focusing on code-generation correctness as the main metric
- relying only on a single judge-model score without preserving raw evidence
- producing theatrical research output without queryable artifacts

## Design Principles

- Behavioral clarity over benchmark vanity.
- Full trace capture over shallow pass/fail summaries.
- Controlled experiment structure over ad hoc prompt collections.
- Deterministic checks first, interpretive grading second.
- Reproducibility over convenience.
- Honest caveats when behavioral conclusions are uncertain.

## Near-Term Deliverables

1. A scenario schema for behavioral trials.
2. An experiment schema for ABAB and related condition sequences.
3. A results artifact format that preserves raw and derived behavioral data.
4. A rubric system for behavioral analysis.
5. A runner capable of executing the same scenario family across many OpenRouter models.
6. Queryable storage and comparison views for cross-condition analysis.

## Project Standard

This project should be opinionated and rigorous. It should help answer research questions about model behavior, not merely accumulate prompts and outputs.
