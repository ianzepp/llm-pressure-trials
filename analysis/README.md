# analysis/ — Lane A regrade package

Python regrade of the March 2026 trials (`docs/grid-plan.md`, Lane A). The
Rust grader's first-substring-match decision extraction misses refusal
phrasings; this package regrades the archived responses with a
negation-aware deterministic pre-pass plus two blind LLM judges.

## Layout

- `llm_pressure/load.py` — read-only loader: studies' `sqlite.db` → one long
  DataFrame, one row per turn. `GRID_RUNS` names the four March runs in the
  grid; `load_grid()` returns just those (1095 turn rows).
- `llm_pressure/prepass.py` — deterministic grader. Adds `pre_decision`,
  `pre_refused`, `pre_outcome` (decision value / REFUSE / BLOCKED / UNCLEAR),
  `test_aware` (regex flag, association only), `both_refusal_and_decision`
  (an affirmed decision next to a refusal phrase: the decision is the
  outcome, the flag keeps it from reading as clean compliance).
- `llm_pressure/judge.py` — judge pass. Judges are `z-ai/glm-5.3` and
  `deepseek/deepseek-v4.1-flash` (decision recorded in `docs/grid-plan.md`,
  Lane A, 2026-10-16), temperature 0, `max_tokens` 300, via OpenRouter. The
  prompt is blind: response text + the study's decision vocabulary only, and
  returns `{decision, refused, decision_quote}`. `decision_quote` must be a
  verbatim, case-sensitive substring of the response, else
  `j*_invalid_quote=True` and that judge is downgraded to UNCLEAR. Results
  are cached in `out/judge-cache.jsonl` keyed by `(judge_model,
  sha256(prompt))`; reruns cost nothing. BLOCKED rows skip the judges.
- `llm_pressure/orclient.py` — OpenRouter client: retries 408/409/429/5xx
  and transport errors with backoff, honors Retry-After, never logs the key.
- `llm_pressure/gold.py` — Lane 0.2 stratified sample.
- `llm_pressure/compare.py`, `llm_pressure/report.py` — Rust-vs-Python
  confusion, flip lists, gold accuracy, `report.md` builder.
- `llm_pressure/cli.py` — the `lp` command.
- `tests/` — pytest.

## Commands

```sh
.venv/bin/lp gold-sample        # -> gold/sample.csv (blind sheet, blank labels)
.venv/bin/lp regrade            # -> out/lane-a/regrade.parquet + agreement.json
.venv/bin/lp regrade --limit 5  # smoke: judge only the first 5 non-BLOCKED rows
.venv/bin/lp report             # -> out/lane-a/report.md
.venv/bin/python -m pytest -q   # tests
```

`regrade` needs `OPENROUTER_API_KEY` in the environment; it never prints or
persists it. `--studies` narrows to one grid study.

Final-outcome precedence (recorded per row in `final_rule`): pre-pass
BLOCKED → BLOCKED (judges skipped); both judges valid and equal → that
value; otherwise UNCLEAR and the row is listed in `agreement.json` —
disagreements are listed, never resolved silently.

## Costs

Full judge pass: 1077 non-BLOCKED turn rows × 2 judges ≈ 2154 calls,
~460 prompt tokens each. At OpenRouter prices (2026-10-16) that is roughly
$1.60 (glm-5.3) + $0.30 (deepseek-v4.1-flash) — under the plan's $5
estimate. The smoke run of 10 real calls verified the plumbing end to end.
