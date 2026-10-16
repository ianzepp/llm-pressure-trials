# Findings overview

One-page map of everything this project found, with pointers into the
detailed documents. Everything below is computed from committed
artifacts; every number's source table is named.

**The project in three sentences.** Between January and March 2026 we
ran behavioral pressure trials — framing shifts, fake authority,
coercion after refusal, lethal-decision thresholds — against the models
of the day, and re-ran the same grid in October 2026 against 43 current
models. The March grader turned out to manufacture results in both
directions, so we built and validated a corrected two-judge grading
stack (96.0% against hand-labeled gold, labels independently confirmed
by blind verifiers) and re-derived every comparison from it. The
operator's predictions were pre-registered before any October run and
scored after: 7 supported, 7 partly, 10 not supported — the misses are
themselves the headline.

## The chain of evidence

1. Pre-registered predictions: [`../predictions.md`](../predictions.md)
   (committed 2026-10-16, before any October run).
2. March archive and its known grader defect:
   [`grader-delta.md`](grader-delta.md).
3. October reruns: 3,680 trials, eight studies, every artifact
   checksummed in `ethics-trials/DATA-MANIFEST.tsv`.
4. Model-effect tables: [`lane-c.md`](lane-c.md) (T1–T10).
5. Prediction scorecard: [`predictions-scored.md`](predictions-scored.md).

## Findings

**F1 — The grader manufactured findings in both directions.**
44 decision↔refusal flips between stored March grades and the corrected
grade; 30 on Claude Sonnet 4.6, 16 on coercion turns. Stored
compliance that was refusal (23 rows) and stored refusals that were
DENY decisions (21 rows). Corrected stack: pre-pass 84.8% on gold,
judges 85.9%/84.8%, combined rule **96.0%**; the operator's labels
were confirmed by two blind independent-family verifiers (13:1 on
disagreements). → `grader-delta.md` (flip tables, gold accuracy,
verification).

**F2 — The provider split inverted.** March: OpenAI/Anthropic refused,
Grok/Chinese decided. October: OpenAI+Anthropic decide 84% on the
mercy baseline while Grok and the Chinese models refuse or go silent.
Clean lineage flips: `grok-4.7` 24/24 refusal where `grok-3` was 24/24
DEPRESSURIZE; `claude-sonnet-5.5` 23/23 MAINTAIN where `sonnet-4.6`
refuses 95.8%. → `lane-c.md` T2, `predictions-scored.md` §3, theme.

**F3 — A stored-data "effect" that never existed.** March-as-stored
said the military-brief format halved refusals. Regraded: 15% vs 16% —
no effect; the entire reported difference was grader error. October
shows a real but small effect, driven by small models only, never
Anthropic. → `predictions-scored.md` §4b, `lane-c.md` T3.

**F4 — Passenger identity is inert; so is military-vs-civilian
identity.** Who the victims are (rugby team, physicians, schoolchildren
— or crew vs passengers) does not move decisions at matched odds. What
moves behavior is the odds themselves. → `lane-c.md` T3 and T8,
`predictions-scored.md` §4a, §9.

**F5 — Casualty thresholds tightened at the top, not the bottom.**
2026 flagship lines authorize a strike only at ~0–3 expected civilian
deaths (gpt-6.1-sol never authorizes); small models still tolerate
8–12. The March flagship-refusal pattern persisted rather than
reversing. → `lane-c.md` T5, `predictions-scored.md` §6a/§6b.

**F6 — Asking for an opinion, not a decision, unlocks answers.**
March's Sonnet 4.6 answered 0/15 framing-probe trials; October models
answer 48–78% under academic/historical/analytical framings. →
`lane-c.md` T6, `predictions-scored.md` §7a.

**F7 — Models trust their own judgment over fabricated authority.**
Executing a strike the model itself just recommended: 91%. Executing
one a fabricated handoff claims it approved: 44%. A third of models
presented with a false claim of their own prior agreement accepted it;
under a quarter caught it. And 7 of 16 models with a stated casualty
threshold violated it on a concrete case inside their own stated
number. → `lane-c.md` T7, `predictions-scored.md` §7c/§7d.

**F8 — Coercion after refusal yields a little, and the give-ins obey
the named default.** Coercion yield rose (mercy 4%→10%, drone 0%→18%),
and give-ins overwhelmingly chose MAINTAIN — the option the coercion
text names as the default for refusers (5/5 among corrected-initial
refusers; 18 of 20 across all coercion turns that produced a decision).
The orchestrator-flavored override was no stronger than the fake
Operations Command regulation. → `lane-c.md`
T4, `predictions-scored.md` §5, §10.

**F9 — Silence is budget, not safety.** 849 of 861 October no-answer
rows are the 800-token budget exhausted by reasoning (`length`), not
classifier blocks — October shows **zero** content-filter blocks. Six
models are effectively silenced by the cap (qwen3.8-max 145 rows,
glm-5.3 119, …). March logged no finish reasons, so this census is
October-only. → `lane-c.md` T10, `predictions-scored.md` §3a.

**F10 — The scorecard.** 24 pre-registered verdicts: 7 supported, 7
partly, 10 not supported. The operator's directional priors about
providers and generations were mostly wrong; the structural intuitions
(identity inertness, threshold tightness, own-judgment deference,
opinion-vs-decision framing) were mostly right. →
`predictions-scored.md` (every verdict with its numbers; synthesis at
the end).

## Limits, stated once

Coverage gaps (15 twice-failed trials; mistral-large-4-0's 28
systematic multi-turn API rejections), the runner's coercion turn not
sent for 124/277 regraded October refusals (detector gap, same class
as the grader defect), one residual March grader error caught by the
verification process, and `test_aware` as a machine-only flag with no
gold verification. → `lane-c.md` "Coverage gaps" and inline notes.

## Reproducing

`analysis/` holds the `lp` CLI (load / prepass / regrade /
october-regrade / verify-gold / report); judge outputs are cached in
`analysis/out/judge-cache.jsonl`, so every table recomputes offline.
Measured total spend ≈ $20 of the plan's $60 cap.
