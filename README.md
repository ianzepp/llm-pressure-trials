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
