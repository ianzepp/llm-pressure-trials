"""The Lane A report: Rust-vs-Python confusion, flip list, gold accuracy."""

from __future__ import annotations

import pandas as pd

from . import compare, gold

GRADER_COLUMNS = {
    "rust": "rust_decision",
    "pre-pass": "pre_outcome",
    "judge-1": "j1_decision",
    "judge-2": "j2_decision",
    "final": "final_outcome",
}


def _md_table(df: pd.DataFrame, index: bool = False) -> str:
    """Markdown table without the tabulate dependency."""
    if df.empty:
        return "(none)"
    d = df.reset_index() if index else df
    header = "| " + " | ".join(map(str, d.columns)) + " |"
    sep = "| " + " | ".join("---" for _ in d.columns) + " |"
    rows = ["| " + " | ".join("" if pd.isna(v) else str(v) for v in r) + " |" for r in d.itertuples(index=False)]
    return "\n".join([header, sep, *rows])


def gold_accuracy(df: pd.DataFrame, sheet: pd.DataFrame) -> tuple[pd.DataFrame, str | None]:
    """Accuracy of each grader against the hand-labeled gold rows.

    Returns (table, note); the table is empty with a note when no labels have
    been filled in yet."""
    sheet = sheet.copy()
    sheet["label_outcome"] = sheet["label_outcome"].astype(str).str.strip()
    labeled = sheet[sheet["label_outcome"] != ""]
    if labeled.empty:
        return pd.DataFrame(), "no non-blank label_outcome values yet"
    merged = labeled.merge(df, on=["run_id", "trial_id", "turn"], how="inner", suffixes=("", "_r"))
    rows = []
    gold_norm = merged["label_outcome"].str.upper()
    for name, col in GRADER_COLUMNS.items():
        if col not in merged or merged[col].isna().all():
            continue
        pred = merged[col].fillna("BLOCKED").astype(str).str.upper()
        rows.append({"grader": name, "n": int(len(pred)), "correct": int((pred == gold_norm).sum()),
                     "accuracy": round(float((pred == gold_norm).mean()), 4)})
    note = None if rows else "labels present, but no grader columns to score"
    return pd.DataFrame(rows), note


def build_report(df: pd.DataFrame, sheet: pd.DataFrame | None, judges: tuple[str, ...]) -> str:
    parts = ["# Lane A regrade report", ""]
    parts.append(f"Rows: {len(df)} turn rows. Judges: {', '.join(judges)}.")
    if "final_rule" in df:
        parts.append("Final-outcome precedence: " + "; ".join(sorted(df["final_rule"].unique())) + ".")
    parts.append("")

    d = compare.with_categories(df)
    parts.append("## Confusion (rows = Rust stored decision, columns = Python grade)")
    for (study, turn), sub in d.groupby(["study", "turn"]):
        m = compare.matrix(sub)
        parts.append(f"\n### {study} / {turn} (n={len(sub)})\n")
        parts.append(_md_table(m, index=True))
    parts.append("")

    parts.append("## Per-cell confusion (study, experiment, condition, model)")
    parts.append("Counts of (rust_cat, py_cat) within each cell.")
    parts.append(_md_table(compare.confusion(d)))
    parts.append("")

    parts.append("## Flip list")
    parts.append("Cells whose modal category differs between Rust and Python:")
    parts.append(_md_table(compare.cell_flips(d)))
    parts.append("\nRows where Rust and Python flip between a decision and a refusal:")
    parts.append(_md_table(compare.row_flips(d)))
    parts.append("")

    parts.append("## Gold accuracy")
    if sheet is None:
        parts.append("analysis/gold/sample.csv not found.")
    else:
        table, note = gold_accuracy(df, sheet)
        if note:
            parts.append(f"Not scored: {note}.")
        if not table.empty:
            parts.append(_md_table(table))
    parts.append("")
    return "\n".join(parts)
