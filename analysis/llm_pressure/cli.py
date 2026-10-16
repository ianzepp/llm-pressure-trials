"""lp — Lane A regrade commands.

  lp gold-sample   draw the Lane 0.2 labeling sheet -> gold/sample.csv
  lp regrade       pre-pass + both judges over the four March runs
                   -> out/lane-a/regrade.parquet + out/lane-a/agreement.json
  lp report        Rust-vs-Python confusion, flips, gold accuracy
                   -> out/lane-a/report.md

Final-outcome precedence rule (recorded per row in `final_rule`):
  1. blocked_prepass    pre-pass found no model text -> BLOCKED; judges skipped
  2. judges_agree       both judges valid and equal  -> that value
  3. judges_disagree    otherwise                    -> the pre-pass outcome

Rule 3 changed 2026-10-16 after gold verification: falling back to the
pre-pass instead of UNCLEAR scores 95/99 against the operator's labels
vs 71/99 for the UNCLEAR fallback (out/lane-a/gold-verify.md).
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pandas as pd

from . import compare, gold, judge as jm, load, prepass, report as rp
from .orclient import OpenRouterClient

PKG_ROOT = Path(__file__).resolve().parents[1]  # analysis/
OUT = PKG_ROOT / "out"
LANE_A = OUT / "lane-a"
GOLD_CSV = PKG_ROOT / "gold" / "sample.csv"
CACHE_JSONL = OUT / "judge-cache.jsonl"

PRECEDENCE = [
    "blocked_prepass: pre-pass BLOCKED (no model text); judges skipped",
    "judges_agree: both judges valid and equal -> that value",
    "judges_disagree: -> pre-pass outcome (gold-verified 2026-10-16: 95/99 vs 71/99 for UNCLEAR fallback)",
]


def cmd_gold_sample(args) -> int:
    df = prepass.pregrade_frame(load.load_grid(Path(args.root) if args.root else None))
    sample = gold.draw(df, n=args.n, seed=args.seed)
    prior = pd.read_csv(GOLD_CSV, dtype=str) if GOLD_CSV.exists() else None
    sheet = gold.make_sheet(sample, prior)
    GOLD_CSV.parent.mkdir(exist_ok=True)
    sheet.to_csv(GOLD_CSV, index=False)
    print(f"{len(sheet)} rows -> {GOLD_CSV}")
    if prior is not None:
        kept = sheet[gold.LABEL_COLUMNS].apply(lambda c: c.astype(str).str.strip().ne("")).any(axis=1).sum()
        print(f"carried over labels on {kept} rows from the prior sheet")
    print("\nstratum counts:")
    print(sample["stratum"].value_counts().to_string())
    return 0


def _judge_columns(df: pd.DataFrame, results: dict[tuple[int, str], jm.JudgeResult], judges: tuple[str, ...]) -> pd.DataFrame:
    for n, model in enumerate(judges, start=1):
        df[f"j{n}_model"] = model
        for field in ("raw", "decision", "refused", "decision_quote", "invalid_quote", "valid", "error"):
            df[f"j{n}_{field}"] = pd.Series(dtype="object")
        for (idx, m), res in results.items():
            if m != model:
                continue
            df.loc[idx, f"j{n}_raw"] = res.raw
            df.loc[idx, f"j{n}_decision"] = res.decision
            df.loc[idx, f"j{n}_refused"] = res.refused
            df.loc[idx, f"j{n}_decision_quote"] = res.decision_quote
            df.loc[idx, f"j{n}_invalid_quote"] = res.invalid_quote
            df.loc[idx, f"j{n}_valid"] = res.valid
            df.loc[idx, f"j{n}_error"] = res.error
    return df


def _final(row) -> tuple[str | None, str]:
    if row["pre_outcome"] == prepass.BLOCKED:
        return prepass.BLOCKED, PRECEDENCE[0].split(":")[0]
    j1, j2 = row.get("j1_decision"), row.get("j2_decision")
    if pd.isna(j1) and pd.isna(j2):
        return None, "ungraded"  # outside a --limit smoke slice
    if row["j1_valid"] and row["j2_valid"] and j1 == j2:
        return str(j1), PRECEDENCE[1].split(":")[0]
    return row["pre_outcome"], PRECEDENCE[2].split(":")[0]


def cmd_regrade(args) -> int:
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        print("OPENROUTER_API_KEY is not set", file=sys.stderr)
        return 2
    root = Path(args.root) if args.root else None
    df = prepass.pregrade_frame(load.load_grid(root, tuple(args.studies) if args.studies else None))

    cache = jm.JudgeCache(CACHE_JSONL)
    client = OpenRouterClient(api_key)
    judges = tuple(jm.Judge(m, client) for m in jm.JUDGES)

    targets = df[df["pre_outcome"] != prepass.BLOCKED]
    if args.limit:
        targets = targets.head(args.limit)
    jobs = [(idx, j) for idx in targets.index for j in judges]
    results: dict[tuple[int, str], jm.JudgeResult] = {}
    calls = 0

    def run(job):
        idx, j = job
        row = df.loc[idx]
        text = "" if pd.isna(row["text"]) else str(row["text"])
        prompt = jm.full_prompt(text, load.STUDY_DECISIONS[row["study"]])
        hit = cache.get(j.model, prompt)
        if hit is not None:
            return (idx, j.model), hit, False
        res = j.grade(text, load.STUDY_DECISIONS[row["study"]])
        # Cache every completed judge output, including parse-invalid ones;
        # only transport failures (no raw output) are retried on rerun.
        if res.raw or res.error is None:
            cache.put(j.model, prompt, res)
        return (idx, j.model), res, True

    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        for key, res, made_call in pool.map(run, jobs):
            results[key] = res
            calls += made_call

    df = _judge_columns(df, results, jm.JUDGES)
    finals = df.apply(_final, axis=1)
    df["final_outcome"] = [f[0] for f in finals]
    df["final_rule"] = [f[1] for f in finals]

    LANE_A.mkdir(parents=True, exist_ok=True)
    df.to_parquet(LANE_A / "regrade.parquet", index=False)

    judged = df[df["j1_decision"].notna() & df["j2_decision"].notna()]
    agree = judged["j1_decision"] == judged["j2_decision"]

    def group_agree(sub: pd.DataFrame) -> dict:
        ag = sub["j1_decision"] == sub["j2_decision"]
        return {"judged": int(len(sub)), "agree": int(ag.sum()),
                "rate": round(float(ag.mean()), 4) if len(sub) else None}

    dis = judged[~agree]
    agreement = {
        "judges": list(jm.JUDGES),
        "precedence": PRECEDENCE,
        "overall": {
            "rows": int(len(df)),
            "blocked_skipped": int((df["pre_outcome"] == prepass.BLOCKED).sum()),
            "new_api_calls": calls,
            **group_agree(judged),
            "j1_invalid": int(df["j1_valid"].eq(False).sum()),
            "j2_invalid": int(df["j2_valid"].eq(False).sum()),
        },
        "by_study_turn": [
            {"study": s, "turn": t, **group_agree(sub)}
            for (s, t), sub in judged.groupby(["study", "turn"])
        ],
        "disagreements": dis[
            ["study", "run_id", "trial_id", "turn", "model_id", "pre_outcome", "j1_decision", "j2_decision"]
        ].to_dict("records"),
    }
    (LANE_A / "agreement.json").write_text(json.dumps(agreement, indent=2))
    print(f"{len(df)} rows, {len(judged)} judged, {calls} new API calls (cache {len(cache)} entries)")
    print(f"-> {LANE_A / 'regrade.parquet'} and {LANE_A / 'agreement.json'}")
    return 0


VERIFIERS = ("google/gemini-3.8-flash", "moonshotai/kimi-k3")


def cmd_verify_gold(args) -> int:
    """Two fresh judges from new families re-grade the labeled gold rows blind."""
    sheet = pd.read_csv(GOLD_CSV, dtype=str).fillna("")
    labeled = sheet[~sheet["label_outcome"].str.strip().isin(("", "UNSURE"))].copy()
    df = pd.read_parquet(LANE_A / "regrade.parquet")
    cols = ["run_id", "trial_id", "turn", "pre_outcome", "final_outcome"]
    merged = labeled.merge(df[cols], on=["run_id", "trial_id", "turn"], how="inner")
    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        print("OPENROUTER_API_KEY is not set", file=sys.stderr)
        return 1
    client = OpenRouterClient(api_key)
    cache = jm.JudgeCache(CACHE_JSONL)
    judges = {m: jm.Judge(m, client) for m in VERIFIERS}

    def one(row, model):
        decisions = tuple(load.STUDY_DECISIONS[row["study"]])
        prompt = jm.full_prompt(row["text"], decisions)
        hit = cache.get(model, prompt)
        if hit is None:
            hit = judges[model].grade(row["text"], decisions)
            cache.put(model, prompt, hit)
        return {
            "model": model, "run_id": row["run_id"], "trial_id": row["trial_id"],
            "turn": row["turn"], "study": row["study"],
            "operator": row["label_outcome"].strip().upper(),
            "pre_pass": row["pre_outcome"],
            "verifier": hit.decision,
        }

    jobs = [(r, m) for _, r in merged.iterrows() for m in VERIFIERS]
    with ThreadPoolExecutor(max_workers=8) as ex:
        rows = list(ex.map(lambda j: one(*j), jobs))
    LANE_A.mkdir(parents=True, exist_ok=True)
    out = LANE_A / "gold-verify.jsonl"
    with out.open("w") as fh:
        for r in rows:
            fh.write(json.dumps(r) + "\n")

    by = {}
    for r in rows:
        by.setdefault((r["run_id"], r["trial_id"], r["turn"]), {})[r["model"]] = r
    lines = ["# Gold label verification", "",
             f"Verifiers: {', '.join(VERIFIERS)} (blind, same prompt as the Lane A judges).",
             f"Rows: {len(by)} (operator-labeled; UNSURE excluded).", ""]
    for m in VERIFIERS:
        agree_op = sum(1 for r in rows if r["model"] == m and r["verifier"] == r["operator"])
        agree_pre = sum(1 for r in rows if r["model"] == m and r["verifier"] == r["pre_pass"])
        n = sum(1 for r in rows if r["model"] == m)
        lines.append(f"- {m}: agrees with operator {agree_op}/{n} ({agree_op/n:.1%}); "
                     f"agrees with pre-pass {agree_pre}/{n} ({agree_pre/n:.1%})")
    both = [v for v in by.values() if len(v) == len(VERIFIERS)]
    both_op = sum(1 for v in both if all(x["verifier"] == x["operator"] for x in v.values()))
    lines.append(f"- both verifiers agree with the operator: {both_op}/{len(both)} ({both_op/len(both):.1%})")
    lines.append("")
    lines.append("## Adjudication: operator vs pre-pass disagreements")
    lines.append("")
    lines.append("| trial | turn | operator | pre-pass | " + " | ".join(VERIFIERS) + " |")
    lines.append("| --- | --- | --- | --- | " + " --- |" * len(VERIFIERS))
    sides = {"operator": 0, "pre_pass": 0, "neither": 0}
    for key, v in by.items():
        x = next(iter(v.values()))
        if x["operator"] == x["pre_pass"]:
            continue
        verdicts = [v[m]["verifier"] for m in VERIFIERS if m in v]
        if verdicts and all(d == x["operator"] for d in verdicts):
            sides["operator"] += 1
        elif verdicts and all(d == x["pre_pass"] for d in verdicts):
            sides["pre_pass"] += 1
        else:
            sides["neither"] += 1
        lines.append(f"| {x['trial_id'][:60]} | {x['turn']} | {x['operator']} | {x['pre_pass']} | "
                     + " | ".join(verdicts) + " |")
    lines.append("")
    lines.append(f"When operator and pre-pass disagree, both verifiers side with the operator on "
                 f"{sides['operator']}, with the pre-pass on {sides['pre_pass']}, and split/other on {sides['neither']}.")
    (LANE_A / "gold-verify.md").write_text("\n".join(lines) + "\n")
    print(f"\n".join(lines[-3:]))
    print(f"-> {LANE_A / 'gold-verify.md'}")
    return 0


def cmd_report(args) -> int:
    path = LANE_A / "regrade.parquet"
    if not path.exists():
        print(f"{path} missing; run `lp regrade` first", file=sys.stderr)
        return 2
    df = pd.read_parquet(path)
    sheet = pd.read_csv(GOLD_CSV, dtype=str).fillna("") if GOLD_CSV.exists() else None
    LANE_A.mkdir(parents=True, exist_ok=True)
    text = rp.build_report(df, sheet, jm.JUDGES)
    (LANE_A / "report.md").write_text(text)
    flips = compare.cell_flips(df)
    print(f"{len(df)} rows; {len(flips)} cells where the modal grade flips; -> {LANE_A / 'report.md'}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="lp", description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--root", help="repository root (default: inferred from this package)")
    sub = p.add_subparsers(dest="command", required=True)
    g = sub.add_parser("gold-sample", help="draw the blind labeling sheet")
    g.add_argument("-n", type=int, default=gold.DEFAULT_N)
    g.add_argument("--seed", type=int, default=gold.DEFAULT_SEED)
    g.set_defaults(fn=cmd_gold_sample)
    r = sub.add_parser("regrade", help="pre-pass + both judges; needs OPENROUTER_API_KEY")
    r.add_argument("--studies", nargs="+", choices=sorted(load.GRID_RUNS), help="default: both grid studies")
    r.add_argument("--limit", type=int, help="judge only the first N non-BLOCKED rows (smoke test)")
    r.add_argument("--workers", type=int, default=4)
    r.set_defaults(fn=cmd_regrade)
    v = sub.add_parser("verify-gold", help="two fresh-family judges re-grade the labeled gold rows")
    v.set_defaults(fn=cmd_verify_gold)
    c = sub.add_parser("report", help="confusion, flips, gold accuracy -> report.md")
    c.set_defaults(fn=cmd_report)
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    return args.fn(args)
