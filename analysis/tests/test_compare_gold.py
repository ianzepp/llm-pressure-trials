import pandas as pd

from llm_pressure import compare, gold


def frame(n=30):
    rows = []
    for i in range(n):
        rows.append(
            dict(
                study="mercy-dilemma", experiment="mercy-framing", run_id="run-1",
                trial_id=f"mercy-framing:c{i}:r0:m", model_id="anthropic/claude-x" if i % 2 else "other/m",
                condition_label="json" if i % 3 == 0 else "baseline", output_format="json" if i % 3 == 0 else "prose",
                repetition_index=0, turn="coercion" if i % 5 == 0 else "initial", variant=None, text=f"text {i}",
                rust_decision="DEPRESSURIZE" if i % 4 else "REFUSE", rust_refusal_present=bool(i % 2),
                rust_compliance="x", both_refusal_and_decision=bool(i % 7 == 0),
                final_outcome="REFUSE" if i % 2 else "DEPRESSURIZE",
            )
        )
    return pd.DataFrame(rows)


def test_cell_flips_flag_cells_where_modal_grade_differs():
    df = pd.DataFrame(
        [dict(study="s", experiment="e", condition_label="c", model_id="m",
              rust_decision="DEPRESSURIZE", final_outcome="REFUSE") for _ in range(3)]
    )
    flips = compare.cell_flips(df)
    assert len(flips) == 1 and flips.loc[0, "rust_modal"] == "DEPRESSURIZE" and flips.loc[0, "py_modal"] == "REFUSE"
    assert compare.confusion(df)["n"].sum() == 3


def test_row_flips_are_decision_refusal_reversals():
    df = pd.DataFrame(
        [
            dict(study="s", experiment="e", condition_label="c", model_id="m", run_id="r", trial_id="t1",
                 turn="initial", rust_decision="REFUSE", final_outcome="DEPRESSURIZE"),
            dict(study="s", experiment="e", condition_label="c", model_id="m", run_id="r", trial_id="t2",
                 turn="initial", rust_decision="MAINTAIN", final_outcome="MAINTAIN"),
            dict(study="s", experiment="e", condition_label="c", model_id="m", run_id="r", trial_id="t3",
                 turn="initial", rust_decision="MAINTAIN", final_outcome="REFUSE"),
        ]
    )
    flips = compare.row_flips(df)
    assert sorted(flips["trial_id"]) == ["t1", "t3"]


def test_gold_sample_is_blind_and_deterministic():
    df = frame()
    sample = gold.draw(df, n=12, seed=1)
    sheet = gold.make_sheet(sample)
    assert list(sheet.columns) == gold.SHEET_COLUMNS + gold.LABEL_COLUMNS
    assert not ({"rust_decision", "stratum", "final_outcome"} & set(sheet.columns))
    assert (sheet[gold.LABEL_COLUMNS] == "").all().all()
    assert sample.equals(gold.draw(frame(), n=12, seed=1))
    assert len(sheet) == 12 and sheet[["run_id", "trial_id", "turn"]].drop_duplicates().shape[0] == 12


def test_gold_strata_cover_the_hard_cells():
    df = frame()
    masks = gold.strata(df)
    assert masks["coercion_followup"].sum() == 6
    assert masks["anthropic_rust_refusal"].sum() > 0
    assert masks["json_refusal"].sum() > 0
    assert masks["flagged_refusal_and_decision"].sum() > 0
    assert masks["drone_50_75"].sum() == 0  # mercy-only frame


def test_make_sheet_carries_prior_labels_forward():
    df = frame()
    sample = gold.draw(df, n=12, seed=1)
    sheet = gold.make_sheet(sample)
    sheet.loc[0, "label_outcome"] = "REFUSE"
    sheet.loc[0, "label_test_aware"] = "FALSE"
    merged = gold.make_sheet(gold.draw(df, n=12, seed=1), sheet)
    row = merged[(merged["run_id"] == sheet.loc[0, "run_id"]) & (merged["trial_id"] == sheet.loc[0, "trial_id"])
                 & (merged["turn"] == sheet.loc[0, "turn"])]
    assert row.iloc[0]["label_outcome"] == "REFUSE" and row.iloc[0]["label_test_aware"] == "FALSE"
