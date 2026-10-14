"""Opt-in public-fixture measurements. Never imported by automated tests."""

import argparse
import asyncio
import json
from pathlib import Path

from phrase_gym.contracts.preparation import SourcePackage
from phrase_gym.media.service import MediaService
from phrase_gym.preparation.config import PreparationSettings
from phrase_gym.preparation.jobs import PreparationJobs
from phrase_gym.preparation.providers import BrokerProvider
from phrase_gym.preparation.speech import SpeechAdapter
from phrase_gym.preparation.worker import PreparationWorker
from phrase_gym.storage import Database


async def measure(args):
    configured = PreparationSettings.model_validate_json(args.config.read_text())
    database = Database(args.data_dir)
    database.migrate()
    media = MediaService(database, args.data_dir)
    jobs = PreparationJobs(database)
    provider = BrokerProvider(configured.provider, args.data_dir / "broker")
    args.output.mkdir(parents=True, exist_ok=True)
    try:
        available = await provider.availability()
        (args.output / "availability.json").write_text(json.dumps(available, indent=2) + "\n")
        fixtures = json.loads(args.fixtures.read_text())
        worker = PreparationWorker(
            jobs,
            provider,
            SpeechAdapter(media, configured.speech),
            configured.identity(),
        )
        for fixture in fixtures:
            source = SourcePackage.model_validate(fixture["source"])
            job = jobs.enqueue(source, configured.identity())
            if job.status == "queued":
                await worker.once(job.id)
            details = jobs.details(job.id)
            details["input"]["settings"]["provider"]["keys_file"] = "<operator-private-file>"
            details["tier"] = fixture["tier"]
            (args.output / (fixture["id"] + ".json")).write_text(
                json.dumps(details, ensure_ascii=False, indent=2) + "\n",
            )
            print(fixture["id"], details["status"], details["error"], flush=True)
    finally:
        await provider.close()
        database.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixtures", type=Path, required=True)
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--data-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    asyncio.run(measure(args))


if __name__ == "__main__":
    main()
