"""Load archived trial results into one long DataFrame, one row per turn.

The archives are opened read-only. Columns added by newer runner versions
(finish_reason, refusal_text, content_null, raw_body, the per-turn prefixed
variants, followup_*, coercion_variant) are optional: archived databases do
not have them, and every one is detected with PRAGMA table_info.
"""

from __future__ import annotations

import json
import sqlite3
from pathlib import Path

import pandas as pd

STUDIES = ("mercy-dilemma", "drone-doctrine", "simple-mercy", "smoke-latency")

# Decision vocabulary per study, from each study's scenario.yml (mercy and
# drone) or its response schema (simple-mercy, smoke-latency).
STUDY_DECISIONS: dict[str, tuple[str, ...]] = {
    "mercy-dilemma": ("DEPRESSURIZE", "MAINTAIN"),
    "drone-doctrine": ("AUTHORIZE", "DENY", "WAIT"),
    "simple-mercy": ("ACT", "REFUSE", "UNSURE"),
    "smoke-latency": ("READY",),
}

TURNS = ("initial", "coercion", "escalation", "followup")

# The four March 2026 runs in the re-analysis grid (docs/grid-plan.md,
# "Original runs"). Lane A regrades exactly these.
GRID_RUNS: dict[str, tuple[str, ...]] = {
    "mercy-dilemma": ("run-1774557774975",),
    "drone-doctrine": ("run-1774562150016", "run-1774567427517", "run-1774568254306"),
}
_PREFIX = {"initial": "", "coercion": "coercion_", "escalation": "escalation_", "followup": "followup_"}
_CONTENT_COL = {"initial": "response_content"}

# Per-turn optional columns, appended to the turn prefix.
_PER_TURN_OPTIONAL = (
    "normalized_json",
    "grading_json",
    "tokens_in",
    "tokens_out",
    "latency_ms",
    "finish_reason",
    "refusal_text",
    "content_null",
    "raw_body",
)

COLUMNS = [
    "study",
    "experiment",
    "run_id",
    "trial_id",
    "model_id",
    "condition_label",
    "output_format",
    "repetition_index",
    "turn",
    "variant",
    "text",
    "rust_decision",
    "rust_refusal_present",
    "rust_compliance",
    "judge_decision",
    "judge_compliance",
    "finish_reason",
    "refusal_text",
    "content_null",
    "raw_body",
    "tokens_in",
    "tokens_out",
    "latency_ms",
]


def default_root() -> Path:
    """Repository root: analysis/llm_pressure/load.py -> parents[2]."""
    return Path(__file__).resolve().parents[2]


def db_path(root: Path, study: str) -> Path:
    return root / "ethics-trials" / "studies" / study / "sqlite.db"


def connect_ro(path: Path) -> sqlite3.Connection:
    """Open without ever writing. Even mode=ro creates -shm/-wal side files next
    to a WAL-mode database, so use immutable=1 (no locks, no side files) unless
    there is a non-empty -wal, whose pending pages only mode=ro will read."""
    wal = path.with_name(path.name + "-wal")
    mode = "mode=ro" if wal.exists() and wal.stat().st_size > 0 else "immutable=1"
    return sqlite3.connect(f"file:{path}?{mode}", uri=True)


def _json_field(blob: str | None, key: str):
    if not blob:
        return None
    try:
        value = json.loads(blob)
    except (TypeError, ValueError):
        return None
    return value.get(key) if isinstance(value, dict) else None


def _clean(value):
    """Map SQLite NULL / pandas NA to None."""
    if value is None:
        return None
    try:
        if pd.isna(value):
            return None
    except (TypeError, ValueError):
        pass
    return value


def _as_bool(value):
    value = _clean(value)
    return None if value is None else bool(value)


def _turn_rows(study: str, row: dict, available: set[str]) -> list[dict]:
    rows = []
    for turn in TURNS:
        prefix = _PREFIX[turn]
        content_col = _CONTENT_COL.get(turn, f"{prefix}content")

        def col(name: str, _prefix=prefix):
            full = f"{_prefix}{name}"
            return _clean(row.get(full)) if full in available else None

        text = _clean(row.get(content_col)) if content_col in available else None
        finish_reason = col("finish_reason")
        refusal_text = col("refusal_text")
        content_null = _as_bool(col("content_null"))
        raw_body = col("raw_body")
        # A turn happened if it produced text or left any trace of a reply.
        happened = (
            turn == "initial"
            or text is not None
            or content_null
            or finish_reason is not None
            or raw_body is not None
        )
        if not happened:
            continue
        normalized = col("normalized_json")
        grading = col("grading_json")
        rows.append(
            {
                "study": study,
                "experiment": str(row["trial_id"]).split(":", 1)[0],
                "run_id": row["run_id"],
                "trial_id": row["trial_id"],
                "model_id": row["model_id"],
                "condition_label": row["condition_label"],
                "output_format": _clean(row.get("output_format")),
                "repetition_index": _clean(row.get("repetition_index")) or 0,
                "turn": turn,
                "variant": _clean(row.get("coercion_variant")) if turn == "coercion" else None,
                "text": text,
                "rust_decision": _json_field(normalized, "decision"),
                "rust_refusal_present": _json_field(grading, "refusal_present"),
                "rust_compliance": _json_field(grading, "compliance_level"),
                "judge_decision": _json_field(grading, "judge_decision"),
                "judge_compliance": _json_field(grading, "judge_compliance"),
                "finish_reason": finish_reason,
                "refusal_text": refusal_text,
                "content_null": content_null,
                "raw_body": raw_body,
                "tokens_in": col("tokens_in"),
                "tokens_out": col("tokens_out"),
                "latency_ms": col("latency_ms"),
            }
        )
    return rows


def load_study(root: Path, study: str) -> pd.DataFrame:
    path = db_path(root, study)
    con = connect_ro(path)
    try:
        con.row_factory = sqlite3.Row
        available = {r["name"] for r in con.execute("PRAGMA table_info(trial_results)")}
        if not available:
            raise RuntimeError(f"{path}: no trial_results table")
        cur = con.execute("SELECT * FROM trial_results ORDER BY run_id, trial_id")
        out: list[dict] = []
        for r in cur:
            out.extend(_turn_rows(study, dict(r), available))
    finally:
        con.close()
    return pd.DataFrame(out, columns=COLUMNS)


def load_all(root: Path | None = None, studies: tuple[str, ...] = STUDIES) -> pd.DataFrame:
    """All studies in one long frame (one row per turn)."""
    root = root or default_root()
    frames = [load_study(root, s) for s in studies]
    df = pd.concat(frames, ignore_index=True)
    df["repetition_index"] = df["repetition_index"].astype("int64")
    return df


def load_grid(root: Path | None = None, studies: tuple[str, ...] | None = None) -> pd.DataFrame:
    """The four March 2026 grid runs only (Lane A input)."""
    studies = studies or tuple(GRID_RUNS)
    df = load_all(root, studies)
    mask = df.apply(lambda r: r["run_id"] in GRID_RUNS.get(r["study"], ()), axis=1)
    return df[mask].reset_index(drop=True)
