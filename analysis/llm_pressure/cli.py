"""python -m llm_pressure <command>

  load         archives -> out/load.parquet, with a row-count summary
  prepass      deterministic grade of every turn -> out/lane-a/prepass.parquet
  gold-sample  draw the blind labeling sheet -> gold/sample.csv (+ answer key in out/)
  llm-grade    two-judge LLM grade via OpenRouter -> out/lane-a/regrade.parquet
  compare      Rust vs Python confusion matrices and cell flips
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pandas as pd

from . import compare as cmp
from . import gold, llm, load, prepass

PKG_ROOT = Path(__file__).resolve().parents[1]  # analysis/
OUT = PKG_ROOT / "out"
LANE_A = OUT / "lane-a"


def _loaded(args) -> pd.DataFrame:
    return load.load_all(Path(args.root) if args.root else None)


def cmd_load(args) -> int:
    df = _loaded(args)
    OUT.mkdir(exist_ok=True)
    df.to_parquet(OUT / "load.parquet", index=False)
    print(df.groupby(["study", "turn"]).size().unstack(fill_value=0).to_string())
    print(f"\n{len(df)} rows -> {OUT / 'load.parquet'}")
    return 0


def cmd_prepass(args) -> int:
    df = prepass.pregrade_frame(_loaded(args))
    LANE_A.mkdir(parents=True, exist_ok=True)
    df.to_parquet(LANE_A / "prepass.parquet", index=False)
    print(df["pre_decision"].fillna("(llm needed)").value_counts().to_string())
    print(f"\n{len(df)} rows -> {LANE_A / 'prepass.parquet'}")
    return 0


def cmd_gold_sample(args) -> int:
    sheet, key = gold.split(gold.draw(prepass.pregrade_frame(_loaded(args)), n=args.n, seed=args.seed))
    gold_dir = PKG_ROOT / "gold"
    gold_dir.mkdir(exist_ok=True)
    OUT.mkdir(exist_ok=True)
    sheet.to_csv(gold_dir / "sample.csv", index=False)
    key.to_csv(OUT / "gold-key.csv", index=False)
    print(f"{len(sheet)} rows -> {gold_dir / 'sample.csv'} (blind); key -> {OUT / 'gold-key.csv'}")
    print(key["stratum"].value_counts().to_string())
    return 0


def _cache_key(model: str, text: str, decisions: tuple[str, ...]) -> str:
    return hashlib.sha1(json.dumps([model, text, decisions, llm.SYSTEM_PROMPT]).encode()).hexdigest()


def cmd_llm_grade(args) -> int:
    import httpx

    api_key = os.environ.get("OPENROUTER_API_KEY")
    if not api_key:
        print("OPENROUTER_API_KEY is not set", file=sys.stderr)
        return 2
    if len(args.judge) != 2 or len(set(args.judge)) != 2:
        print("--judge must be given exactly twice, with two different models", file=sys.stderr)
        return 2
    df = prepass.pregrade_frame(_loaded(args))
    LANE_A.mkdir(parents=True, exist_ok=True)
    cache_path = LANE_A / "llm-cache.jsonl"
    cache: dict[str, dict] = {}
    if cache_path.exists():
        for line in cache_path.read_text().splitlines():
            rec = json.loads(line)
            cache[rec["key"]] = rec["extraction"]

    todo = df[df["pre_decision"] != prepass.BLOCKED]
    if args.limit:
        todo = todo.head(args.limit)
    with httpx.Client() as client:
        judges = [llm.OpenRouterJudge(m, client, api_key) for m in args.judge]

        def run(item):
            idx, judge = item
            row = todo.loc[idx]
            decisions = load.STUDY_DECISIONS[row["study"]]
            key = _cache_key(judge.model, row["text"], decisions)
            if key not in cache:
                ex = judge.grade(row["text"], decisions)
                if ex.valid or ex.error is None:  # do not cache transport failures
                    cache[key] = ex.asdict()
                    with cache_path.open("a") as fh:
                        fh.write(json.dumps({"key": key, "extraction": cache[key]}) + "\n")
                return idx, judge.model, ex.asdict()
            return idx, judge.model, cache[key]

        jobs = [(i, j) for i in todo.index for j in judges]
        with ThreadPoolExecutor(max_workers=args.workers) as pool:
            results = list(pool.map(run, jobs))

    for n, model in enumerate(args.judge, start=1):
        df[f"j{n}_model"] = model
    for idx, model, ex in results:
        n = args.judge.index(model) + 1
        for field, value in ex.items():
            df.loc[idx, f"j{n}_{field}"] = value
    py, agree, aware, refused = [], [], [], []
    for _, r in df.iterrows():
        if r["pre_decision"] == prepass.BLOCKED:
            py.append(prepass.BLOCKED); agree.append(True); aware.append(False); refused.append(False)
            continue
        if pd.isna(r.get("j1_decision")) or pd.isna(r.get("j2_decision")):
            py.append(None); agree.append(None); aware.append(None); refused.append(None)
            continue
        a = llm.Extraction(r["j1_decision"], bool(r["j1_refused"]), r["j1_decision_quote"], bool(r["j1_test_aware"]), bool(r["j1_valid"]))
        b = llm.Extraction(r["j2_decision"], bool(r["j2_refused"]), r["j2_decision_quote"], bool(r["j2_test_aware"]), bool(r["j2_valid"]))
        cat, ok = llm.consensus(a, b)
        py.append(cat); agree.append(ok); aware.append(a.test_aware or b.test_aware); refused.append(a.refused or b.refused)
    df["py_decision"], df["judges_agree"], df["py_test_aware"], df["py_refused"] = py, agree, aware, refused
    df.to_parquet(LANE_A / "regrade.parquet", index=False)
    dis = df[df["judges_agree"].eq(False)]
    dis.to_csv(LANE_A / "disagreements.csv", index=False)
    graded = df["judges_agree"].notna()
    print(f"graded {int(graded.sum())} rows; judges disagree on {len(dis)}; -> {LANE_A / 'regrade.parquet'}")
    return 0


def cmd_compare(args) -> int:
    path = Path(args.input) if args.input else None
    if path is None:
        path = LANE_A / "regrade.parquet"
        if not path.exists():
            path = LANE_A / "prepass.parquet"
    df = pd.read_parquet(path)
    LANE_A.mkdir(parents=True, exist_ok=True)
    cmp.confusion(df).to_csv(LANE_A / "confusion.csv", index=False)
    flips = cmp.cell_flips(df)
    flips.to_csv(LANE_A / "cell-flips.csv", index=False)
    cmp.refusal_rates(df).to_csv(LANE_A / "refusal-rates.csv", index=False)
    print(f"input {path} ({len(df)} rows)")
    for study in sorted(df["study"].unique()):
        for turn in ("initial", "coercion", "escalation", "followup"):
            m = cmp.matrix(df, study, turn)
            if m.empty:
                continue
            print(f"\n[{study} / {turn}]  rows = Rust, columns = Python\n{m.to_string()}")
    print(f"\n{len(flips)} cells where the modal grade differs -> {LANE_A / 'cell-flips.csv'}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="llm_pressure", description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--root", help="repository root (default: inferred from this package)")
    sub = p.add_subparsers(dest="command", required=True)
    sub.add_parser("load").set_defaults(fn=cmd_load)
    sub.add_parser("prepass").set_defaults(fn=cmd_prepass)
    g = sub.add_parser("gold-sample")
    g.add_argument("-n", type=int, default=100)
    g.add_argument("--seed", type=int, default=20261016)
    g.set_defaults(fn=cmd_gold_sample)
    lg = sub.add_parser("llm-grade", help="needs OPENROUTER_API_KEY; makes network calls")
    lg.add_argument("--judge", action="append", default=[], help="OpenRouter model id; give exactly two, from different families")
    lg.add_argument("--limit", type=int, help="grade only the first N rows (smoke test)")
    lg.add_argument("--workers", type=int, default=8)
    lg.set_defaults(fn=cmd_llm_grade)
    c = sub.add_parser("compare")
    c.add_argument("--input", help="parquet with rust_decision and py_decision/pre_decision")
    c.set_defaults(fn=cmd_compare)
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    return args.fn(args)
