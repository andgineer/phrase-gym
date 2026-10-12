import asyncio

import pytest

from phrase_gym.api.boundary import LocalBoundary, MAX_REQUEST_BYTES


def run_boundary(events, headers=()):
    received = []
    sent = []
    stream = iter(events)

    async def receive():
        return next(stream)

    async def send(message):
        sent.append(message)

    async def app(_scope, replay, _send):
        received.append(await replay())
        received.append(await replay())

    scope = {"type": "http", "method": "PUT", "scheme": "http", "headers": list(headers)}
    asyncio.run(LocalBoundary(app, ())(scope, receive, send))
    return received, sent


@pytest.mark.parametrize("headers", [(), ((b"content-length", b"2"),)])
def test_actual_stream_size_is_bounded_even_without_honest_content_length(headers):
    received, sent = run_boundary(
        [
            {"type": "http.request", "body": b"x" * (MAX_REQUEST_BYTES // 2), "more_body": True},
            {
                "type": "http.request",
                "body": b"x" * (MAX_REQUEST_BYTES // 2 + 1),
                "more_body": False,
            },
        ],
        headers,
    )
    assert received == []
    assert sent[0]["status"] == 413


def test_split_body_replayed_exactly_once_and_disconnect_propagates():
    received, sent = run_boundary(
        [
            {"type": "http.request", "body": b'{"native_', "more_body": True},
            {"type": "http.request", "body": b'language":"ru"}', "more_body": False},
            {"type": "http.disconnect"},
        ]
    )
    assert received == [
        {"type": "http.request", "body": b'{"native_language":"ru"}', "more_body": False},
        {"type": "http.disconnect"},
    ]
    assert sent == []


def test_mid_upload_disconnect_never_calls_handler():
    received, sent = run_boundary(
        [
            {"type": "http.request", "body": b"partial", "more_body": True},
            {"type": "http.disconnect"},
        ]
    )
    assert received == sent == []


def test_invalid_content_length_rejected_before_receiving_body():
    received, sent = run_boundary([], ((b"content-length", b"invalid"),))
    assert received == []
    assert sent[0]["status"] == 413
