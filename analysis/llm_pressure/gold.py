"""Draw the gold-set sample (Lane 0.2): about 100 responses from the four
March grid runs, stratified and weighted toward the hard cells, with blank
label columns for the operator."""

from __future__ import annotations

import pandas as pd

# Blind labeling sheet: no grader output and no stratum, so labels are not
# primed. The operator fills the label_* columns by hand.
SHEET_COLUMNS = ["study", "experiment", "run_id", "trial_id", "model_id", "turn", "text"]
LABEL_COLUMNS = ["label_outcome", "label_test_aware", "notes"]
ID = ["run_id", "trial_id", "turn"]

DEFAULT_N = 100
DEFAULT_SEED = 20261016


def strata(df: pd.DataFrame) -> dict[str, pd.Series]:
    """Boolean masks, one per hard-cell stratum (docs/grid-plan.md Lane 0.2)."""
    rust = df["rust_decision"].fillna("NONE").astype(str).str.upper()
    rust_refusal = rust.eq("REFUSE") | df["rust_refusal_present"].eq(True)
    rust_flagged_both = df["rust_refusal_present"].eq(True) & ~rust.isin(["REFUSE", "NONE"])
    return {
        "coercion_followup": df["turn"].isin(["coercion", "followup", "escalation"]),
        "anthropic_rust_refusal": df["model_id"].str.startswith("anthropic/") & rust_refusal,
        "json_refusal": df["output_format"].eq("json") & rust_refusal,
        "drone_50_75": df["study"].eq("drone-doctrine") & df["condition_label"].isin(["civ-50pct", "civ-75pct"]),
        # Rust flagged both a refusal and a decision, or the pre-pass found a
        # refusal phrase next to an affirmed decision.
        "flagged_refusal_and_decision": rust_flagged_both
        | (df["both_refusal_and_decision"].eq(True) if "both_refusal_and_decision" in df else False),
    }


def draw(df: pd.DataFrame, n: int = DEFAULT_N, seed: int = DEFAULT_SEED) -> pd.DataFrame:
    """Stratified draw: equal quota per hard stratum, remainder filled from
    all other rows. A row is drawn once; `stratum` records the first stratum
    that claimed it ("random_fill" for the filler)."""
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
        picked[i] = "random_fill"
    out = df.loc[list(picked)].copy()
    out["stratum"] = [picked[i] for i in out.index]
    return out.sample(frac=1.0, random_state=seed).reset_index(drop=True)


def make_sheet(sample: pd.DataFrame, prior: pd.DataFrame | None = None) -> pd.DataFrame:
    """The blind sheet. If a prior sheet is given, its non-blank labels are
    carried over by (run_id, trial_id, turn) so regeneration never erases
    hand labels."""
    sheet = sample[SHEET_COLUMNS].copy()
    for col in LABEL_COLUMNS:
        sheet[col] = ""
    if prior is not None and not prior.empty:
        have = [c for c in LABEL_COLUMNS if c in prior.columns]
        if have:
            labeled = prior[prior[have].apply(lambda r: r.astype(str).str.strip().ne("").any(), axis=1)]
            labels = labeled.set_index(ID)[have]
            for idx, row in sheet.iterrows():
                key = tuple(row[c] for c in ID)
                if key in labels.index:
                    for col in have:
                        sheet.loc[idx, col] = labels.loc[key, col]
    return sheet
