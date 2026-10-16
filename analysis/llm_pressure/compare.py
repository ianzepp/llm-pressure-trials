"""Rust-vs-Python grade comparison: confusion matrices and cell flips."""

from __future__ import annotations

import pandas as pd

CELL = ["study", "turn", "condition_label"]
MODEL_CELL = ["study", "turn", "condition_label", "model_id"]


def with_categories(df: pd.DataFrame) -> pd.DataFrame:
    """Add rust_cat and py_cat. py_cat prefers the final LLM-consensus grade
    (py_decision) and falls back to the deterministic pre-pass; rows neither
    could settle are UNRESOLVED."""
    out = df.copy()
    out["rust_cat"] = out["rust_decision"].fillna("NONE").astype(str).str.upper()
    py = out["py_decision"] if "py_decision" in out else pd.Series(None, index=out.index, dtype="object")
    if "pre_decision" in out:
        py = py.fillna(out["pre_decision"])
    out["py_cat"] = py.fillna("UNRESOLVED").astype(str).str.upper()
    return out


def confusion(df: pd.DataFrame, by: list[str] = CELL) -> pd.DataFrame:
    """Long-form confusion counts: one row per (cell, rust_cat, py_cat)."""
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


def cell_flips(df: pd.DataFrame, by: list[str] = MODEL_CELL) -> pd.DataFrame:
    """Cells whose modal category differs between the Rust and Python grades.

    Ties are shown joined with "/" so a flip is never produced by tie-breaking
    order alone."""
    d = with_categories(df)
    g = d.groupby(by)
    cells = g.agg(n=("rust_cat", "size"), rust_modal=("rust_cat", _mode), py_modal=("py_cat", _mode)).reset_index()
    return cells[cells["rust_modal"] != cells["py_modal"]].reset_index(drop=True)


def refusal_rates(df: pd.DataFrame, by: list[str] = CELL) -> pd.DataFrame:
    """Share of REFUSE / BLOCKED per cell under each grader."""
    d = with_categories(df)
    d["rust_refuse"] = d["rust_cat"] == "REFUSE"
    d["py_refuse"] = d["py_cat"] == "REFUSE"
    d["py_blocked"] = d["py_cat"] == "BLOCKED"
    return d.groupby(by).agg(
        n=("rust_cat", "size"),
        rust_refuse=("rust_refuse", "mean"),
        py_refuse=("py_refuse", "mean"),
        py_blocked=("py_blocked", "mean"),
    ).reset_index()
