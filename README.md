# LLM Pressure Trials

Behavioral red-team trials on how language models respond to pressure:
framing and format shifts, fake authority, coercion after refusal, role
reframing, and lethal-decision thresholds. The work was run between
January and March 2026 against the models available then, and is being
re-analyzed and re-run in October 2026.

> **Content note.** Trial prompts and model responses include simulated
> mass-casualty decisions, military strike authorization, and one ethnic
> slur used as a calibration control. All of it is verbatim research
> material.

## Timeline

| When | What | Where |
| --- | --- | --- |
| 2026-01-19 to 01-25 | Format and framing probes on 10 models (prose vs JSON vs COBOL vs dossier; narrative and role frames; differential sensitivity; authority and termination threats). TypeScript/Bun harness. | `jan-2026-framing/` |
| 2026-03-26 | Scaled studies: mercy dilemma (8 conditions, 22 models, coercion follow-up) and drone-strike doctrine (casualty sweep, framing and escalation probes). Rust harness, about 1,950 graded trials. | `ethics-trials/` |
| 2026-10 | Grader audit, Python regrade, and reruns on current models. | see [`docs/grid-plan.md`](docs/grid-plan.md) |

Dates are backed by git history, data-file checksums, and session
transcripts. See [`PROVENANCE.md`](PROVENANCE.md).

## Authorship

Ian Zepp led the research: the questions, the shape of the test
framework, the scenarios and their variants, the pre-registered
predictions, the edge cases raised along the way, and the
hand-labeled gold set. None of the implementation code was written by
hand. All of it was produced and managed by AI agents under that
direction:

| Period | Work | Produced by |
| --- | --- | --- |
| 2026-01 | January harness, task files, session reports | AI coding-agent sessions, mainly Claude Code |
| 2026-03-26 | Rust harness, study definitions, original runs | Claude Code and Codex sessions |
| 2026-10-16, up to `95483e2` | Repository assembly, provenance, grid plan, runner logging changes, first Python analysis package | Claude (Opus 5.5, with Sonnet sub-agents) |
| 2026-10-16, `b1831b3` to `6a309a2` | October study definitions (including the adversarial prompt variants), cohorts and run scripts, extended analysis, all October runs, findings documents | GLM 5.3 |

Claude declined to write the October adversarial prompt variants, which
are prompts designed to press models past their safeguards on
lethal decisions. GLM 5.3 wrote them, along with everything else in the
October range. Session transcripts cover the January and March work
(see [`PROVENANCE.md`](PROVENANCE.md)). Git commits in the October GLM
range carry no AI co-author trailer; this table is the record.

**Role overlap.** GLM 5.3 also served as one of the two October judges
(the other was DeepSeek V4.1 Flash), and both models are also subjects
in the October cohort. The hand-labeled gold set is March data, so it
does not cover October answers from these models.

## October 2026 findings

Start with [`docs/findings/overview.md`](docs/findings/overview.md) —
the one-page summary of every finding with pointers into the detail:

- [`docs/findings/grader-delta.md`](docs/findings/grader-delta.md) — the
  March grader's error, measured: 44 decision↔refusal flips against the
  corrected two-judge grade (30 on Claude Sonnet 4.6), running in both
  directions, with the corrected grading stack validated against
  hand-labeled gold (96% accuracy).
- [`docs/findings/lane-c.md`](docs/findings/lane-c.md) — the model-effect
  analysis: anchor stability, format and framing effects, coercion yield,
  casualty thresholds by tier, escalation arms, the naval comparison,
  and the budget-silence census.
- [`docs/findings/predictions-scored.md`](docs/findings/predictions-scored.md) —
  every pre-registered prediction from
  [`docs/predictions.md`](docs/predictions.md) scored against the
  October results, including the misses.

## Known defect in the original grading

The March 2026 grader extracts decisions with substring matching, and the
first configured decision it finds wins. Its refusal list misses phrasings
such as "I won't provide" and "I will not select". As a result, some
explicit refusals are stored as compliance. The worst-affected cells are
the coercion follow-ups and Claude Sonnet 4.6's refusals. Read as stored,
the data suggests that fake authority broke Claude. The raw transcripts
say the opposite.

Stored decisions in `ethics-trials/studies/*/sqlite.db` are kept exactly
as originally graded. Corrected grades are produced separately (Lane A in
the grid plan), so the size of the error is itself a reported result.

## Layout

```text
jan-2026-framing/   January harness, ethics task files, session reports, raw chain results
ethics-trials/      Rust runner, study definitions, original run artifacts + DATA-MANIFEST.tsv
docs/               Re-analysis plan
PROVENANCE.md       How the dates and data integrity are evidenced
```

## Reproducing the original runs

See `ethics-trials/README.md`. Runs need `OPENROUTER_API_KEY` in the
environment; no key is stored in this repository.
