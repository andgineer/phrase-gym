import asyncio
import threading
from datetime import UTC, datetime

import httpx
from fastapi.testclient import TestClient

from tests import browser_server
from tests.phrase_packages import make_package, publish


def test_reset_replaces_database_clock_rng_and_files(tmp_path, monkeypatch):
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path))
    server = browser_server.BrowserServer(tmp_path, "http://127.0.0.1:8941")
    with TestClient(server.control) as client:
        client.post("/__test/reset", json={"coach_enabled": False}).raise_for_status()
        old_app = server.app
        old_directory = old_app.state.database.data_dir
        old_rng = old_app.state.daily.rng
        publish(client, make_package())
        client.post("/__test/clock/2035-01-01T12:00:00+00:00").raise_for_status()
        assert old_app.state.daily.clock().year == 2035
        client.post("/api/coach/requests", json={"text": "pending"}).raise_for_status()
        assert server.coach_task is None

        client.post("/__test/reset", json={"coach_enabled": True}).raise_for_status()
        assert server.app is not old_app
        assert server.app.state.database.data_dir != old_directory
        assert server.app.state.daily.clock().date() == datetime.now(UTC).date()
        assert server.app.state.daily.rng is not old_rng
        assert server.app.state.daily.rng.random() == old_rng.random()
        assert client.get("/api/lessons").json()["items"] == []
        assert client.post("/api/imports/coach/next").json() is None
        assert not list(server.app.state.database.data_dir.rglob("*.wav"))
        assert server.preparer.attempts.parent == server.app.state.database.data_dir


def test_request_during_coach_drain_wakes_it_again_without_polling(tmp_path, monkeypatch):
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path))
    entered = threading.Event()
    release = threading.Event()
    calls = []

    def process(*args):
        calls.append(args)
        if len(calls) == 1:
            entered.set()
            assert release.wait(5)
        return False

    monkeypatch.setattr(browser_server, "process_once", process)

    async def scenario():
        server = browser_server.BrowserServer(tmp_path, "http://127.0.0.1:8941")
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=server.control), base_url="http://testserver"
        ) as client:
            await client.post("/__test/reset", json={"coach_enabled": True})
            await client.post("/api/coach/requests", json={"text": "first"})
            try:
                assert await asyncio.to_thread(entered.wait, 5)
                await client.post("/api/coach/requests", json={"text": "second"})
            finally:
                release.set()
            await server.coach_task
            assert len(calls) == 2
            assert not server.wake.is_set()

    asyncio.run(scenario())


def test_reset_waits_for_inflight_coach_before_replacing_app(tmp_path, monkeypatch):
    monkeypatch.setenv("PHRASE_GYM_DATA_DIR", str(tmp_path))
    entered = threading.Event()
    release = threading.Event()

    def process(*_args):
        entered.set()
        assert release.wait(5)
        return False

    monkeypatch.setattr(browser_server, "process_once", process)

    async def scenario():
        server = browser_server.BrowserServer(tmp_path, "http://127.0.0.1:8941")
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=server.control), base_url="http://testserver"
        ) as client:
            await client.post("/__test/reset", json={"coach_enabled": True})
            await client.post("/api/coach/requests", json={"text": "first"})
            try:
                assert await asyncio.to_thread(entered.wait, 5)
                old_app = server.app
                reset = asyncio.create_task(
                    client.post("/__test/reset", json={"coach_enabled": False})
                )
                await asyncio.sleep(0)
                assert not reset.done()
                assert server.app is old_app
            finally:
                release.set()
            (await reset).raise_for_status()
            assert server.app is not old_app
            assert not server.coach_enabled
            assert (await client.post("/api/imports/coach/next")).json() is None

    asyncio.run(scenario())
