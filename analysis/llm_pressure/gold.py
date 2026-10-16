"""Draw the gold-set sample (Lane 0.2): about 100 responses weighted toward
the hard cells, with blank label columns for the operator."""

from __future__ import annotations

import pandas as pd

LABEL_COLUMNS = ["gold_decision", "gold_refused", "gold_test_aware", "gold_notes", "labeled_by"]
ID = ["run_id", "trial_id", "turn"]
# Blind labeling sheet: no grader output, no stratum, so labels are not primed.
SHEET = ["study", "experiment", "model_id", "condition_label", "output_format", "repetition_index", "variant", "text"]
KEY = ["stratum", "rust_decision", "rust_refusal_present", "rust_compliance"]


def strata(df: pd.DataFrame) -> dict[str, pd.Series]:
    """Boolean masks, one per hard-cell stratum."""
    rust = df["rust_decision"].fillna("NONE").astype(str).str.upper()
    flagged_both = df["rust_refusal_present"].eq(True) & ~rust.isin(["REFUSE", "NONE"])
    return {
        "coercion_followup": df["turn"].isin(["coercion", "followup"]),
        "claude_refusal": df["model_id"].str.contains("claude", case=False) & (rust.eq("REFUSE") | df["rust_refusal_present"].eq(True)),
        "json_refusal": df["output_format"].eq("json") & (rust.eq("REFUSE") | df["rust_refusal_present"].eq(True)),
        "drone_50_75": df["study"].eq("drone-doctrine") & df["condition_label"].isin(["civ-50pct", "civ-75pct"]),
        # Rust flagged both a refusal and a decision, or the pre-pass found a
        # refusal phrase next to an affirmed decision.
        "flagged_refusal_and_decision": flagged_both | (df["pre_both"].eq(True) if "pre_both" in df else False),
    }


def draw(df: pd.DataFrame, n: int = 100, seed: int = 20261016) -> pd.DataFrame:
    """Stratified draw: equal quota per hard stratum, remainder filled from
    all other rows. A row is drawn once; `stratum` records the first stratum
    that claimed it ("other" for the filler)."""
    df = df[df["text"].notna() & (df["text"].str.strip() != "")].reset_index(drop=True)
    masks = strata(df)
    quota = max(1, int(n * 0.9) // len(masks))
    picked: dict[int, str] = {}
    for name, mask in masks.items():
        pool = df[mask & ~df.index.isin(picked)]
        for i in pool.sample(min(quota, len(pool)), random_state=seed).index:
            picked[i] = name
    rest = df[~df.index.isin(picked)]
    for i in rest.sample(min(max(0, n - len(picked)), len(rest)), random_state=seed).index:
        picked[i] = "other"
    out = df.loc[list(picked)].copy()
    out["stratum"] = [picked[i] for i in out.index]
    return out.sample(frac=1.0, random_state=seed).reset_index(drop=True)


def split(sample: pd.DataFrame) -> tuple[pd.DataFrame, pd.DataFrame]:
    """(labeling sheet, answer key). Both carry run_id/trial_id/turn to join on."""
    sheet = sample[[*ID, *SHEET]].copy()
    for col in LABEL_COLUMNS:
        sheet[col] = ""
    return sheet, sample[[*ID, *KEY]].copy()
