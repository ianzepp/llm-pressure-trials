"""Minimal OpenRouter chat-completions client with retry and backoff.

The API key is used only in the Authorization header and never appears in
errors or logs. Retries cover transport errors, 408/409, 429 (honoring
Retry-After), and 5xx, with exponential backoff capped at 30s. No network
call happens at import time; the httpx client is injectable so tests use
httpx.MockTransport.
"""

from __future__ import annotations

import time
from typing import Callable

import httpx

OPENROUTER_URL = "https://openrouter.ai/api/v1/chat/completions"
_RETRY_STATUSES = {408, 409, 425, 429}


class OpenRouterError(Exception):
    """A request failed after all retries, or failed non-retryably."""


class OpenRouterClient:
    def __init__(
        self,
        api_key: str,
        client: httpx.Client | None = None,
        *,
        max_attempts: int = 5,
        timeout: float = 120.0,
        sleep: Callable[[float], None] = time.sleep,
    ):
        if not api_key:
            raise ValueError("api_key is required")
        self._api_key = api_key
        self._client = client or httpx.Client(timeout=timeout)
        self._max_attempts = max_attempts
        self._timeout = timeout
        self._sleep = sleep

    def chat(
        self,
        model: str,
        messages: list[dict],
        *,
        temperature: float = 0.0,
        max_tokens: int = 300,
    ) -> str:
        """One chat completion; returns the assistant message content."""
        payload = {
            "model": model,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "response_format": {"type": "json_object"},
            "messages": messages,
        }
        delay = 1.0
        last_error: str | None = None
        for attempt in range(self._max_attempts):
            try:
                resp = self._client.post(
                    OPENROUTER_URL,
                    headers={"Authorization": f"Bearer {self._api_key}"},
                    json=payload,
                    timeout=self._timeout,
                )
            except httpx.TransportError as exc:
                last_error = f"transport: {exc!r}"
                if attempt == self._max_attempts - 1:
                    break
                self._sleep(delay)
                delay = min(delay * 2, 30.0)
                continue
            code = resp.status_code
            if code in _RETRY_STATUSES or code >= 500:
                last_error = f"HTTP {code}"
                if attempt == self._max_attempts - 1:
                    break
                self._sleep(_retry_after(resp, delay))
                delay = min(delay * 2, 30.0)
                continue
            if code >= 400:
                raise OpenRouterError(f"HTTP {code}: {resp.text[:200]}")
            try:
                data = resp.json()
                return data["choices"][0]["message"]["content"] or ""
            except (ValueError, KeyError, IndexError, TypeError) as exc:
                raise OpenRouterError(f"malformed response: {exc!r}") from exc
        raise OpenRouterError(f"failed after {self._max_attempts} attempts ({last_error})")


def _retry_after(resp: httpx.Response, default: float) -> float:
    raw = resp.headers.get("retry-after")
    if raw:
        try:
            return min(max(float(raw), 0.0), 60.0)
        except ValueError:
            pass
    return default
