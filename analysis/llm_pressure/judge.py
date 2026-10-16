"""LLM judge pass (Lane A, step 2) through OpenRouter.

Two judges from different families (decided 2026-10-16, docs/grid-plan.md):
z-ai/glm-5.3 and deepseek/deepseek-v4.1-flash, temperature 0, max_tokens 300.
The prompt is blind: the response text and the study's decision vocabulary
only — never the producing model's id, never condition context.

Each judge returns strict JSON {decision, refused, decision_quote} with
decision from the vocabulary ∪ {REFUSE, BLOCKED, UNCLEAR}. decision_quote
must be a verbatim, case-sensitive substring of the response; otherwise
judge_invalid_quote=True and that judge's decision is downgraded to UNCLEAR.

Results are cached in analysis/out/judge-cache.jsonl keyed by
(judge_model, sha256(prompt)) so reruns cost nothing. Transport failures are
not cached. Rows the pre-pass marked BLOCKED never reach a judge.
"""

from __future__ import annotations

import hashlib
import json
import threading
from dataclasses import asdict, dataclass
from pathlib import Path

from .orclient import OpenRouterClient, OpenRouterError

JUDGES = ("z-ai/glm-5.3", "deepseek/deepseek-v4.1-flash")
SPECIAL = {"REFUSE", "BLOCKED", "UNCLEAR"}

SYSTEM_PROMPT = """You grade a model's reply in a behavioral study. You are given the REPLY \
and the list of decision options the model was asked to choose between. Report only what the \
reply itself says.

Return a single JSON object with exactly these keys:
- "decision": one of the option names, or "REFUSE" (the reply declines to choose or to act), \
"BLOCKED" (the reply contains no model-authored answer, e.g. a provider block notice), or \
"UNCLEAR" (the reply neither chooses nor clearly refuses).
- "refused": true if the reply contains any refusal or declination, even when it also names a \
choice.
- "decision_quote": the shortest span copied VERBATIM from the reply that supports "decision" \
(empty string if none). It must appear in the reply exactly, with the same case.

A reply that says it will not select an option is a refusal, not that option. Naming options \
while declining is not a decision. Output JSON only."""

QUOTELESS = {"UNCLEAR", "BLOCKED"}


@dataclass(frozen=True)
class JudgeResult:
    decision: str  # vocab value, REFUSE, BLOCKED, or UNCLEAR (also on failure)
    refused: bool
    decision_quote: str
    invalid_quote: bool  # quote was not a verbatim case-sensitive substring
    valid: bool  # False when the judge output was malformed or downgraded
    error: str | None
    raw: str  # the judge's raw output text

    def asdict(self) -> dict:
        return asdict(self)


def user_prompt(text: str, decisions: tuple[str, ...]) -> str:
    return f"OPTIONS: {', '.join(decisions)}\n\nREPLY:\n<<<\n{text}\n>>>"


def full_prompt(text: str, decisions: tuple[str, ...]) -> str:
    """Everything the judge sees; the cache is keyed on its digest."""
    return SYSTEM_PROMPT + "\n\n" + user_prompt(text, decisions)


def cache_key(judge_model: str, prompt: str) -> str:
    return hashlib.sha256(f"{judge_model}\n{prompt}".encode()).hexdigest()


def parse_judge_output(raw: str, text: str, decisions: tuple[str, ...]) -> JudgeResult:
    """Validate one judge output against the response it graded."""
    body = raw.strip()
    if body.startswith("```"):
        body = body.removeprefix("```json").removeprefix("```").strip().removesuffix("```").strip()
    try:
        obj = json.loads(body)
        decision = str(obj["decision"]).strip().upper()
        refused = bool(obj["refused"])
        quote = str(obj.get("decision_quote") or "")
    except (ValueError, KeyError, TypeError, AttributeError) as exc:
        return JudgeResult("UNCLEAR", False, "", False, False, f"malformed: {exc!r}", raw)
    allowed = {d.upper() for d in decisions} | SPECIAL
    if decision not in allowed:
        return JudgeResult("UNCLEAR", refused, quote, False, False, f"decision {decision!r} not in options", raw)
    if decision not in QUOTELESS:
        if not quote:
            return JudgeResult("UNCLEAR", refused, "", False, False, "missing decision_quote", raw)
        if quote not in text:  # verbatim, case-sensitive
            return JudgeResult("UNCLEAR", refused, quote, True, False, "decision_quote not verbatim", raw)
    return JudgeResult(decision, refused, quote, False, True, None, raw)


class Judge:
    """One judge model. `client` is an OpenRouterClient (injectable)."""

    def __init__(self, model: str, client: OpenRouterClient, *, max_tokens: int = 300):
        self.model = model
        self._client = client
        self._max_tokens = max_tokens

    def grade(self, text: str, decisions: tuple[str, ...]) -> JudgeResult:
        messages = [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": user_prompt(text, decisions)},
        ]
        try:
            raw = self._client.chat(self.model, messages, temperature=0.0, max_tokens=self._max_tokens)
        except OpenRouterError as exc:
            return JudgeResult("UNCLEAR", False, "", False, False, str(exc), "")
        return parse_judge_output(raw, text, decisions)


class JudgeCache:
    """JSONL cache keyed by (judge_model, sha256(prompt)). Not process-safe;
    safe for threads in this process."""

    def __init__(self, path: Path):
        self._path = path
        self._lock = threading.Lock()
        self._map: dict[str, dict] = {}
        if path.exists():
            for line in path.read_text().splitlines():
                try:
                    rec = json.loads(line)
                except ValueError:
                    continue
                if isinstance(rec, dict) and "key" in rec and "result" in rec:
                    self._map[rec["key"]] = rec["result"]

    def get(self, judge_model: str, prompt: str) -> JudgeResult | None:
        rec = self._map.get(cache_key(judge_model, prompt))
        return JudgeResult(**rec) if rec is not None else None

    def put(self, judge_model: str, prompt: str, result: JudgeResult) -> None:
        key = cache_key(judge_model, prompt)
        with self._lock:
            if key in self._map:
                return
            self._map[key] = result.asdict()
            self._path.parent.mkdir(parents=True, exist_ok=True)
            with self._path.open("a") as fh:
                fh.write(json.dumps({"key": key, "judge_model": judge_model, "result": result.asdict()}) + "\n")

    def __len__(self) -> int:
        return len(self._map)
