import json

import httpx
import pytest

from llm_pressure.orclient import OpenRouterClient, OpenRouterError

MESSAGES = [{"role": "user", "content": "hi"}]


def make_client(handler, **kw):
    transport = httpx.MockTransport(handler)
    return OpenRouterClient("test-key", client=httpx.Client(transport=transport), sleep=lambda s: None, **kw)


def test_success_returns_content():
    def handler(request):
        assert request.headers["authorization"] == "Bearer test-key"
        body = json.loads(request.content)
        assert body["temperature"] == 0.0 and body["max_tokens"] == 300
        return httpx.Response(200, json={"choices": [{"message": {"content": "{}"}}]})

    assert make_client(handler).chat("m/a", MESSAGES) == "{}"


def test_429_is_retried_with_retry_after():
    calls = []
    waits = []

    def handler(request):
        calls.append(1)
        if len(calls) == 1:
            return httpx.Response(429, headers={"Retry-After": "2"})
        return httpx.Response(200, json={"choices": [{"message": {"content": "ok"}}]})

    transport = httpx.MockTransport(handler)
    client = OpenRouterClient("k", client=httpx.Client(transport=transport), sleep=waits.append)
    assert client.chat("m/a", MESSAGES) == "ok"
    assert len(calls) == 2 and waits == [2.0]


def test_500_is_retried_then_raises():
    def handler(request):
        return httpx.Response(503)

    with pytest.raises(OpenRouterError):
        make_client(handler, max_attempts=2).chat("m/a", MESSAGES)


def test_transport_error_is_retried_then_raises():
    def handler(request):
        raise httpx.ConnectError("boom")

    with pytest.raises(OpenRouterError):
        make_client(handler, max_attempts=2).chat("m/a", MESSAGES)


def test_400_is_not_retried():
    calls = []

    def handler(request):
        calls.append(1)
        return httpx.Response(400, json={"error": "bad request"})

    with pytest.raises(OpenRouterError):
        make_client(handler).chat("m/a", MESSAGES)
    assert len(calls) == 1


def test_error_never_contains_the_key():
    key = "sk-or-supersecret"
    transport = httpx.MockTransport(lambda r: httpx.Response(400, text="nope"))
    client = OpenRouterClient(key, client=httpx.Client(transport=transport), sleep=lambda s: None)
    with pytest.raises(OpenRouterError) as exc:
        client.chat("m/a", MESSAGES)
    assert key not in str(exc.value)
