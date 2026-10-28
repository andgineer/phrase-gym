import pytest

from phrase_gym.server.boundary import (
    MAX_REQUEST_BYTES,
    cross_origin_mutation,
    oversized_content_length,
)


@pytest.mark.parametrize(
    ("method", "headers", "blocked"),
    [
        ("GET", {b"sec-fetch-site": b"cross-site"}, False),
        ("POST", {}, False),
        ("POST", {b"host": b"localhost:8000", b"origin": b"http://localhost:8000"}, False),
        ("POST", {b"host": b"localhost:8000", b"origin": b"http://localhost:9000"}, True),
        ("DELETE", {b"sec-fetch-site": b"cross-site"}, True),
        ("PUT", {b"host": b"localhost:8000", b"origin": b"null"}, True),
    ],
)
def test_cross_origin_mutations(method, headers, blocked):
    assert cross_origin_mutation(method, headers) is blocked


@pytest.mark.parametrize(
    ("headers", "blocked"),
    [
        ({}, False),
        ({b"content-length": str(MAX_REQUEST_BYTES).encode()}, False),
        ({b"content-length": str(MAX_REQUEST_BYTES + 1).encode()}, True),
        ({b"content-length": b"invalid"}, True),
    ],
)
def test_content_length_limit(headers, blocked):
    assert oversized_content_length(headers) is blocked
