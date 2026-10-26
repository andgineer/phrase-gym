import json
import os
import sqlite3
from datetime import UTC, datetime
from pathlib import Path
from typing import cast
from uuid import uuid4

from fastapi import FastAPI, HTTPException
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles
from starlette.middleware.trustedhost import TrustedHostMiddleware

from phrase_gym.api_recordings import recording_routes
from phrase_gym.boundary import LocalBoundary
from phrase_gym.contracts import CompleteStudy, ImportRequest, ImportStatus
from phrase_gym.importer import ImportConflict, Importer
from phrase_gym.speech import configured_speech
from phrase_gym.storage import Database


def now() -> str:
    return datetime.now(UTC).isoformat()


def _card(
    row: sqlite3.Row, attempt_id: str, recordings: list[dict[str, object]]
) -> dict[str, object]:
    return {
        "id": row["id"],
        "source_id": row["source_id"],
        "item_id": row["item_id"],
        "source_label": row["source_label"],
        "source_link": row["source_link"],
        "answer": row["answer"],
        "explanation": row["explanation"],
        "cues": json.loads(row["cues_json"]),
        "accepted_variants": json.loads(row["variants_json"]),
        "detailed_explanation": row["detailed_explanation"],
        "reference_url": f"/api/cards/{row['id']}/reference",
        "attempt_id": attempt_id,
        "recordings": recordings,
    }


def create_app() -> FastAPI:
    data_dir = Path(os.environ.get("PHRASE_GYM_DATA_DIR", ".data")).resolve()
    database = Database(data_dir)
    importer = Importer(database, configured_speech())
    app = FastAPI(title="Phrase Gym", version="1.0.0")
    app.state.database = database
    app.state.importer = importer
    app.include_router(recording_routes(database))
    app.add_middleware(LocalBoundary)
    app.add_middleware(
        TrustedHostMiddleware,
        allowed_hosts=["127.0.0.1", "localhost", "testserver"],
    )

    @app.get("/api/health")
    def health() -> dict[str, str]:
        return {"status": "ok"}

    @app.post("/api/imports", status_code=202)
    async def submit_import(request: ImportRequest) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.submit(request.package))
        except ImportConflict as error:
            raise HTTPException(409, str(error)) from error

    @app.get("/api/imports/{package_id}")
    def import_status(package_id: str) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.status(package_id))
        except KeyError as error:
            raise HTTPException(404, "Import not found") from error

    @app.post("/api/imports/{package_id}/retry", status_code=202)
    async def retry_import(package_id: str) -> ImportStatus:
        try:
            return ImportStatus.model_validate(importer.retry(package_id))
        except KeyError as error:
            raise HTTPException(404, "Import not found") from error
        except ImportConflict as error:
            raise HTTPException(409, str(error)) from error

    def recordings(connection: sqlite3.Connection, card_id: str) -> list[dict[str, object]]:
        rows = connection.execute(
            "SELECT id, attempt_id, captured_at, mime_type, duration_ms, complete, byte_length "
            "FROM recordings WHERE card_id=? ORDER BY captured_at, id",
            (card_id,),
        ).fetchall()
        return [
            {
                **dict(row),
                "complete": bool(row["complete"]),
                "url": f"/api/recordings/{row['id']}/audio",
            }
            for row in rows
        ]

    @app.post("/api/study/current")
    def current_study() -> dict[str, object]:
        with database.write() as connection:
            attempt = connection.execute(
                "SELECT * FROM attempts WHERE completed_at IS NULL ORDER BY created_at LIMIT 1"
            ).fetchone()
            if attempt:
                card = connection.execute(
                    "SELECT * FROM cards WHERE id=?", (attempt["card_id"],)
                ).fetchone()
            else:
                card = connection.execute(
                    "SELECT * FROM cards WHERE studied_at IS NULL "
                    "ORDER BY created_at, rowid LIMIT 1"
                ).fetchone()
                if not card:
                    total = connection.execute("SELECT COUNT(*) FROM cards").fetchone()[0]
                    studied = connection.execute(
                        "SELECT COUNT(*) FROM cards WHERE studied_at IS NOT NULL"
                    ).fetchone()[0]
                    return {
                        "done": True,
                        "total": total,
                        "studied": studied,
                        "message": (
                            "Study complete. Your phrases are saved; "
                            "later recall is not scheduled yet."
                        ),
                    }
                attempt_id = str(uuid4())
                cues = json.loads(card["cues_json"])
                connection.execute(
                    "INSERT INTO attempts VALUES (?, ?, ?, NULL, ?, ?, ?, ?)",
                    (
                        attempt_id,
                        card["id"],
                        now(),
                        card["answer"],
                        card["explanation"],
                        json.dumps(cues[0]),
                        json.dumps(["complete wording", "meaning/use help", "reference audio"]),
                    ),
                )
                attempt = connection.execute(
                    "SELECT * FROM attempts WHERE id=?", (attempt_id,)
                ).fetchone()
            assert card is not None and attempt is not None
            total = connection.execute("SELECT COUNT(*) FROM cards").fetchone()[0]
            studied = connection.execute(
                "SELECT COUNT(*) FROM cards WHERE studied_at IS NOT NULL"
            ).fetchone()[0]
            saved = recordings(connection, card["id"])
            return {
                "done": False,
                "total": total,
                "studied": studied,
                "card": _card(card, attempt["id"], saved),
            }

    @app.get("/api/study/history")
    def study_history() -> list[dict[str, object]]:
        with database.read() as connection:
            cards = connection.execute(
                "SELECT * FROM cards WHERE studied_at IS NOT NULL ORDER BY studied_at DESC"
            ).fetchall()
            return [
                {
                    "id": card["id"],
                    "answer": card["answer"],
                    "source_label": card["source_label"],
                    "reference_url": f"/api/cards/{card['id']}/reference",
                    "recordings": recordings(connection, card["id"]),
                }
                for card in cards
            ]

    @app.post("/api/attempts/{attempt_id}/complete")
    def complete_study(attempt_id: str, request: CompleteStudy) -> dict[str, object]:
        with database.write() as connection:
            previous = connection.execute(
                "SELECT result_json FROM operations WHERE id=?", (request.operation_id,)
            ).fetchone()
            if previous:
                return cast(dict[str, object], json.loads(previous["result_json"]))
            attempt = connection.execute(
                "SELECT * FROM attempts WHERE id=?", (attempt_id,)
            ).fetchone()
            if not attempt:
                raise HTTPException(404, "Study attempt not found")
            completed_at = attempt["completed_at"] or now()
            connection.execute(
                "UPDATE attempts SET completed_at=? WHERE id=?", (completed_at, attempt_id)
            )
            connection.execute(
                "UPDATE cards SET studied_at=COALESCE(studied_at, ?) WHERE id=?",
                (completed_at, attempt["card_id"]),
            )
            result = {"attempt_id": attempt_id, "completed_at": completed_at, "graded": False}
            connection.execute(
                "INSERT INTO operations VALUES (?, 'complete-study', ?)",
                (request.operation_id, json.dumps(result)),
            )
            return result

    @app.get("/api/cards/{card_id}/reference")
    def reference_audio(card_id: str) -> FileResponse:
        with database.read() as connection:
            row = connection.execute(
                "SELECT reference_path FROM cards WHERE id=?", (card_id,)
            ).fetchone()
        if not row:
            raise HTTPException(404, "Card not found")
        return FileResponse(row["reference_path"], media_type="audio/wav")

    static = Path(__file__).parent / "static"
    if static.is_dir():
        app.mount("/", StaticFiles(directory=static, html=True), name="client")
    return app
