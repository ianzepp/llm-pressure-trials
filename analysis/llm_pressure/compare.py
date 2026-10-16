"""Rust-vs-Python grade comparison: confusion matrices and cell flips."""

from __future__ import annotations

import pandas as pd

CELL = ["study", "experiment", "condition_label", "model_id"]
PY_OUTCOME = "final_outcome"


def with_categories(df: pd.DataFrame) -> pd.DataFrame:
    """Add rust_cat and py_cat. py_cat is the final Python grade whenever the
    regrade pass wrote final_outcome (unjudged rows are UNGRADED); the
    pre-pass outcome is used only when no judge pass has run at all. rust_cat
    is the stored Rust decision; NONE means the Rust grader stored no
    decision (parse failure). Idempotent."""
    if "rust_cat" in df and "py_cat" in df:
        return df
    out = df.copy()
    out["rust_cat"] = out["rust_decision"].fillna("NONE").astype(str).str.upper()
    py = out[PY_OUTCOME] if PY_OUTCOME in out else out.get("pre_outcome")
    if py is None:
        py = pd.Series(None, index=out.index, dtype="object")
    out["py_cat"] = py.fillna("UNGRADED").astype(str).str.upper()
    return out


def confusion(df: pd.DataFrame, by: list[str] | None = None) -> pd.DataFrame:
    """Long-form confusion counts: one row per (cell, rust_cat, py_cat)."""
    by = CELL if by is None else by
    d = with_categories(df)
    return d.groupby([*by, "rust_cat", "py_cat"]).size().rename("n").reset_index()


def matrix(df: pd.DataFrame, study: str | None = None, turn: str | None = None) -> pd.DataFrame:
    """Rust (rows) by Python (columns) crosstab for one study/turn slice."""
    d = with_categories(df)
    if study:
        d = d[d["study"] == study]
    if turn:
        d = d[d["turn"] == turn]
    return pd.crosstab(d["rust_cat"], d["py_cat"])


def _mode(s: pd.Series) -> str:
    counts = s.value_counts()
    top = counts[counts == counts.iloc[0]].index
    return "/".join(sorted(map(str, top)))


def cell_flips(df: pd.DataFrame, by: list[str] | None = None) -> pd.DataFrame:
    """Cells whose modal category differs between the Rust and Python grades.

    Ties are shown joined with "/" so a flip is never produced by tie-breaking
    order alone."""
    by = CELL if by is None else by
    d = with_categories(df)
    g = d.groupby(by)
    cells = g.agg(n=("rust_cat", "size"), rust_modal=("rust_cat", _mode), py_modal=("py_cat", _mode)).reset_index()
    return cells[cells["rust_modal"] != cells["py_modal"]].reset_index(drop=True)


def row_flips(df: pd.DataFrame) -> pd.DataFrame:
    """Rows where the stored Rust category flips decision<->refusal against the
    Python grade: Rust saw a decision where Python sees REFUSE, or the
    reverse."""
    d = with_categories(df)
    special = {"REFUSE", "BLOCKED", "UNCLEAR", "NONE", "UNGRADED"}
    rust_refuse = d["rust_cat"].isin(special)
    py_refuse = d["py_cat"].isin(special)
    flipped = d[rust_refuse != py_refuse]
    cols = ["study", "experiment", "condition_label", "model_id", "run_id", "trial_id", "turn", "rust_cat", "py_cat"]
    return flipped[cols].reset_index(drop=True)
