CREATE TABLE practice_session (
    id TEXT PRIMARY KEY,
    start_payload TEXT NOT NULL CHECK (json_valid(start_payload)),
    manifest TEXT NOT NULL CHECK (json_valid(manifest)),
    status TEXT NOT NULL CHECK (status IN ('active', 'paused', 'superseded', 'completed')),
    open_slot INTEGER UNIQUE CHECK (open_slot = 1),
    version INTEGER NOT NULL CHECK (version >= 1),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    queue TEXT NOT NULL CHECK (json_valid(queue)),
    repairs TEXT NOT NULL CHECK (json_valid(repairs)),
    cursor INTEGER NOT NULL CHECK (cursor >= 0),
    last_sequence INTEGER NOT NULL CHECK (last_sequence >= 0),
    CHECK ((open_slot IS NOT NULL) = (status IN ('active', 'paused')))
) STRICT;

CREATE TABLE practice_event (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES practice_session (id),
    sequence INTEGER NOT NULL CHECK (sequence >= 1),
    kind TEXT NOT NULL,
    attempt_id TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    payload TEXT NOT NULL CHECK (json_valid(payload)),
    receipt TEXT NOT NULL CHECK (json_valid(receipt)),
    UNIQUE (session_id, sequence)
) STRICT;

CREATE INDEX practice_event_by_attempt ON practice_event (attempt_id, kind);

CREATE TABLE practice_attempt (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES practice_session (id),
    target_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    step INTEGER NOT NULL CHECK (step >= 0),
    payload TEXT NOT NULL CHECK (json_valid(payload)),
    UNIQUE (session_id, step),
    FOREIGN KEY (target_id, revision) REFERENCES target_revision (target_id, revision)
) STRICT;

CREATE INDEX practice_attempt_by_target ON practice_attempt (target_id);

CREATE TABLE dialogue_exposure (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES practice_session (id),
    target_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    occurred_at TEXT NOT NULL,
    FOREIGN KEY (target_id, revision) REFERENCES target_revision (target_id, revision)
) STRICT;

CREATE INDEX dialogue_exposure_by_target ON dialogue_exposure (target_id);

CREATE TABLE review_receipt (
    group_id TEXT PRIMARY KEY,
    target_id TEXT NOT NULL REFERENCES target (id),
    attempt_id TEXT NOT NULL REFERENCES practice_attempt (id),
    payload TEXT NOT NULL CHECK (json_valid(payload))
) STRICT;

CREATE TABLE chunk_history (
    target_id TEXT NOT NULL REFERENCES target (id),
    chunk_id TEXT NOT NULL,
    presentations INTEGER NOT NULL CHECK (presentations >= 0),
    recalls INTEGER NOT NULL CHECK (recalls >= 0),
    failures INTEGER NOT NULL CHECK (failures >= 0),
    last_presented TEXT NOT NULL,
    PRIMARY KEY (target_id, chunk_id)
) STRICT;

CREATE TABLE cue_report (
    target_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    cue_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    PRIMARY KEY (target_id, revision, cue_id),
    FOREIGN KEY (target_id, revision) REFERENCES target_revision (target_id, revision)
) STRICT;

CREATE TABLE learner_recording (
    id TEXT PRIMARY KEY,
    target_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    attempt_id TEXT REFERENCES practice_attempt (id),
    exposure_id TEXT REFERENCES exposure (id),
    metadata TEXT NOT NULL CHECK (json_valid(metadata)),
    audio BLOB NOT NULL CHECK (length(audio) BETWEEN 1 AND 12582912),
    sha256 TEXT NOT NULL CHECK (length(sha256) = 64),
    received_at TEXT NOT NULL,
    FOREIGN KEY (target_id, revision) REFERENCES target_revision (target_id, revision),
    CHECK ((attempt_id IS NULL) != (exposure_id IS NULL))
) STRICT;

CREATE INDEX learner_recording_by_target ON learner_recording (target_id);

CREATE TABLE device_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    payload TEXT NOT NULL CHECK (json_valid(payload))
) STRICT;
