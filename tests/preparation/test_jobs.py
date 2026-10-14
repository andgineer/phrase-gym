from datetime import timedelta

import pytest

from phrase_gym.errors import DomainError
from phrase_gym.preparation.jobs import LEASE_SECONDS, MAX_ATTEMPTS, PreparationJobs


def test_dedup_config_and_restart_lease_fencing(prepared_runtime, source):
    jobs, _, _, clock = prepared_runtime
    first = jobs.enqueue(source, {"voice": "one"})
    assert jobs.enqueue(source, {"voice": "one"}).id == first.id
    assert jobs.enqueue(source, {"voice": "two"}).id != first.id
    job_id, token, _, _ = jobs.claim()
    jobs.checkpoint(job_id, token, {"generation": "retained"})
    restarted = PreparationJobs(jobs.db, lambda: clock[0])
    clock[0] += timedelta(seconds=LEASE_SECONDS + 1)
    claimed = restarted.claim()
    assert claimed[0] == job_id and claimed[3] == {"generation": "retained"}
    with pytest.raises(DomainError, match="expired or was replaced"):
        jobs.checkpoint(job_id, token, {"late": "must not overwrite"}, complete=True)
    assert jobs.get(job_id).result == {"generation": "retained"}


def test_retry_limit_and_manual_retry(prepared_runtime, source):
    jobs, _, _, clock = prepared_runtime
    job = jobs.enqueue(source, {})
    for _ in range(MAX_ATTEMPTS):
        job_id, token, _, _ = jobs.claim()
        jobs.fail(job_id, token, "provider_unavailable")
        clock[0] += timedelta(minutes=2)
    assert jobs.claim() is None
    assert jobs.get(job.id).error.startswith("retry_limit")
    assert jobs.retry(job.id).status == "queued"
    assert jobs.claim()[0] == job.id
