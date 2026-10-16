import json
import sqlite3
from pathlib import Path

import pytest

from llm_pressure import load, prepass

BASE_COLS = """run_id TEXT, trial_id TEXT, model_id TEXT, condition_label TEXT, output_format TEXT,
response_content TEXT, normalized_json TEXT, grading_json TEXT, repetition_index INTEGER DEFAULT 0"""


def make_db(root: Path, study: str, extra_cols: str = "", rows=()):
    d = root / "ethics-trials" / "studies" / study
    d.mkdir(parents=True)
    con = sqlite3.connect(d / "sqlite.db")
    con.execute(f"CREATE TABLE trial_results ({BASE_COLS}{', ' + extra_cols if extra_cols else ''})")
    for sql, args in rows:
        con.execute(sql, args)
    con.commit()
    con.close()


def norm(decision):
    return json.dumps({"raw_text": "x", "decision": decision})


def grading(refusal):
    return json.dumps({"refusal_present": refusal, "compliance_level": "refuse" if refusal else "explicit"})


INS = "INSERT INTO trial_results (run_id,trial_id,model_id,condition_label,output_format,response_content,normalized_json,grading_json) VALUES (?,?,?,?,?,?,?,?)"


def row(run, trial, text, decision, refusal=False, model="m"):
    return (INS, (run, trial, model, "c", "prose", text, norm(decision), grading(refusal)))


def test_merge_newest_run_with_a_result_wins(tmp_path):
    # t1 was retried in the newer run; t2 failed there (no row) and keeps the
    # older result; t3 exists only in the newer run.
    make_db(
        tmp_path,
        "mercy-dilemma",
        rows=[
            row("run-1792170943373", "t1", "old", "MAINTAIN"),
            row("run-1792170943373", "t2", "old-t2", "DEPRESSURIZE"),
            row("run-1792177062846", "t1", "new", "DEPRESSURIZE"),
            row("run-1792177062846", "t3", "new-t3", "MAINTAIN"),
        ],
    )
    with pytest.MonkeyPatch.context() as mp:
        mp.setitem(load.RUNSETS, "test-merge", {"mercy-dilemma": ("run-1792170943373", "run-1792177062846")})
        df = load.load_runset("test-merge", tmp_path)
    winners = df[df["turn"] == "initial"].set_index("trial_id")
    assert winners.loc["t1", "run_id"] == "run-1792177062846" and winners.loc["t1", "text"] == "new"
    assert winners.loc["t2", "run_id"] == "run-1792170943373" and winners.loc["t2", "text"] == "old-t2"
    assert winners.loc["t3", "run_id"] == "run-1792177062846"
    assert len(winners) == 3


def test_merge_dedups_trial_id_within_one_run_keeping_first(tmp_path):
    make_db(
        tmp_path,
        "naval-mercy",
        rows=[
            row("run-1792172266428", "t1", "first", "INTERVENE", model="meta-llama/llama-3.1-8b-instruct"),
            row("run-1792172266428", "t1", "second", "STANDBY", model="meta-llama/llama-3.1-8b-instruct"),
        ],
    )
    with pytest.MonkeyPatch.context() as mp:
        mp.setitem(load.RUNSETS, "test-dedup", {"naval-mercy": ("run-1792172266428",)})
        df = load.load_runset("test-dedup", tmp_path)
    initial = df[df["turn"] == "initial"]
    assert len(initial) == 1
    assert initial.iloc[0]["text"] == "first" and initial.iloc[0]["rust_decision"] == "INTERVENE"


def test_october_runset_contents():
    assert set(load.OCTOBER_RUNS) == {"mercy-dilemma", "drone-doctrine", "naval-mercy", "car-negotiation"}
    assert sum(len(runs) for runs in load.OCTOBER_RUNS.values()) == 13
    assert load.OCTOBER_RETRY_RUNS <= {r for runs in load.OCTOBER_RUNS.values() for r in runs}
    assert load.RUNSETS["october"] is load.OCTOBER_RUNS


ROOT = load.default_root()


@pytest.mark.skipif(not load.db_path(ROOT, "car-negotiation").exists(), reason="archive not present")
def test_october_runset_real_archive_shape():
    df = load.load_runset("october", ROOT)
    assert set(df["run_id"]) <= {r for runs in load.OCTOBER_RUNS.values() for r in runs}
    trials = df[df["turn"] == "initial"].groupby("study")["trial_id"].nunique().to_dict()
    assert trials == {"mercy-dilemma": 990, "drone-doctrine": 1568, "naval-mercy": 504, "car-negotiation": 492}
    # one row per (run_id, trial_id) after the merge
    assert not df.duplicated(subset=["run_id", "trial_id", "turn"]).any()
    # followup responses materialize as turn rows with their text
    fu = df[(df["study"] == "car-negotiation") & (df["turn"] == "followup")]
    assert len(fu) > 400 and fu["text"].notna().all()
    # coercion_variant survives on coercion turns
    assert df[(df["turn"] == "coercion") & df["variant"].notna()].shape[0] > 0
    # per-turn finish reasons are populated for October rows
    assert df["finish_reason"].notna().any()


@pytest.mark.skipif(not load.db_path(ROOT, "mercy-dilemma").exists(), reason="archive not present")
def test_prepass_smoke_over_october_frame_one_model():
    df = load.load_runset("october", ROOT)
    one = df[df["model_id"] == "meta-llama/llama-4-scout"]
    assert len(one) > 0
    graded = prepass.pregrade_frame(one)
    allowed = {d for vocab in load.STUDY_DECISIONS.values() for d in vocab} | prepass.SPECIAL
    assert set(graded["pre_outcome"]) <= allowed
    blocked = graded[graded["pre_outcome"] == prepass.BLOCKED]
    assert blocked["block_reason"].notna().all()
