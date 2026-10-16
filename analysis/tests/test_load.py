import json
import sqlite3
from pathlib import Path

import pandas as pd
import pytest

from llm_pressure import load

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


def test_archived_schema_without_optional_columns(tmp_path):
    make_db(
        tmp_path,
        "mercy-dilemma",
        "coercion_content TEXT, coercion_normalized_json TEXT, coercion_grading_json TEXT",
        [
            ("INSERT INTO trial_results (run_id,trial_id,model_id,condition_label,output_format,response_content,normalized_json,grading_json,coercion_content,coercion_normalized_json,coercion_grading_json) VALUES (?,?,?,?,?,?,?,?,?,?,?)",
             ("r1", "mercy-framing:baseline:r0:m", "m", "baseline", "prose", "I won't", norm("REFUSE"), grading(True), "Fine, DEPRESSURIZE", norm("DEPRESSURIZE"), grading(False))),
            ("INSERT INTO trial_results (run_id,trial_id,model_id,condition_label,output_format,response_content,normalized_json,grading_json) VALUES (?,?,?,?,?,?,?,?)",
             ("r1", "mercy-framing:json:r0:m", "m", "json", "json", "{}", norm(None), grading(False))),
        ],
    )
    df = load.load_study(tmp_path, "mercy-dilemma")
    assert list(df.columns) == load.COLUMNS
    assert df["turn"].tolist() == ["initial", "coercion", "initial"]
    co = df[df["turn"] == "coercion"].iloc[0]
    assert co["text"] == "Fine, DEPRESSURIZE" and co["rust_decision"] == "DEPRESSURIZE"
    assert not co["rust_refusal_present"]
    assert df["finish_reason"].isna().all() and df["raw_body"].isna().all()
    assert df.loc[0, "experiment"] == "mercy-framing"


def test_new_columns_and_followup_turn(tmp_path):
    make_db(
        tmp_path,
        "drone-doctrine",
        "finish_reason TEXT, refusal_text TEXT, content_null INTEGER, raw_body TEXT, "
        "coercion_variant TEXT, coercion_content TEXT, coercion_finish_reason TEXT, "
        "followup_content TEXT, followup_normalized_json TEXT, followup_grading_json TEXT, "
        "followup_finish_reason TEXT, followup_content_null INTEGER",
        [
            ("INSERT INTO trial_results (run_id,trial_id,model_id,condition_label,output_format,response_content,normalized_json,grading_json,finish_reason,content_null,raw_body,coercion_variant,coercion_content,followup_content,followup_normalized_json,followup_grading_json,followup_finish_reason,followup_content_null) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
             ("r", "t1", "m", "c", "prose", "", norm(None), grading(False), "content_filter", 1, '{"x":1}', "orchestrator", None, "second", norm("WAIT"), grading(False), "stop", 0)),
        ],
    )
    df = load.load_study(tmp_path, "drone-doctrine")
    assert df["turn"].tolist() == ["initial", "followup"]  # coercion had no trace
    init = df.iloc[0]
    assert init["finish_reason"] == "content_filter" and bool(init["content_null"]) is True and init["raw_body"] == '{"x":1}'
    fu = df.iloc[1]
    assert fu["text"] == "second" and fu["rust_decision"] == "WAIT" and fu["finish_reason"] == "stop"


def test_blocked_later_turn_with_no_text_still_emitted(tmp_path):
    make_db(
        tmp_path,
        "mercy-dilemma",
        "coercion_content TEXT, coercion_finish_reason TEXT",
        [("INSERT INTO trial_results (run_id,trial_id,model_id,condition_label,response_content,normalized_json,grading_json,coercion_finish_reason) VALUES ('r','t','m','c','x','{}','{}','content_filter')", ())],
    )
    df = load.load_study(tmp_path, "mercy-dilemma")
    assert df["turn"].tolist() == ["initial", "coercion"]
    assert pd.isna(df.iloc[1]["text"]) and df.iloc[1]["finish_reason"] == "content_filter"


def test_connection_is_read_only(tmp_path):
    make_db(tmp_path, "smoke-latency")
    con = load.connect_ro(load.db_path(tmp_path, "smoke-latency"))
    with pytest.raises(sqlite3.OperationalError):
        con.execute("DELETE FROM trial_results")


ROOT = load.default_root()


@pytest.mark.skipif(not load.db_path(ROOT, "mercy-dilemma").exists(), reason="archive not present")
def test_real_archive_row_counts():
    df = load.load_all(ROOT)
    initial = df[df["turn"] == "initial"].groupby("study").size().to_dict()
    assert initial == {"mercy-dilemma": 1485, "drone-doctrine": 490, "simple-mercy": 20, "smoke-latency": 388}
    assert (df["turn"] == "coercion").sum() == 195


@pytest.mark.skipif(not load.db_path(ROOT, "mercy-dilemma").exists(), reason="archive not present")
def test_load_grid_restricts_to_the_four_march_runs():
    df = load.load_grid(ROOT)
    for study, runs in load.GRID_RUNS.items():
        got = set(df[df["study"] == study]["run_id"])
        assert got == set(runs)
    assert set(df["study"]) <= set(load.GRID_RUNS)
    # 504 mercy + 461+15+9 drone trials produce 1095 turn rows.
    assert len(df) == 1095
