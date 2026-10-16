CREATE TABLE alembic_version (
    version_num VARCHAR(32) NOT NULL,
    CONSTRAINT alembic_version_pkc PRIMARY KEY (version_num)
);

CREATE TABLE app_settings (
    id INTEGER NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE chunk_history (
    target_id VARCHAR NOT NULL,
    chunk_id VARCHAR NOT NULL,
    presentations INTEGER NOT NULL,
    recalls INTEGER NOT NULL,
    failures INTEGER NOT NULL,
    last_presented VARCHAR NOT NULL,
    PRIMARY KEY (target_id, chunk_id),
    FOREIGN KEY(target_id) REFERENCES target (id)
);

CREATE TABLE content_import (
    id VARCHAR NOT NULL,
    payload TEXT NOT NULL,
    report TEXT NOT NULL,
    status VARCHAR NOT NULL,
    created_at VARCHAR NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE cue_report (
    target_id VARCHAR NOT NULL,
    revision INTEGER NOT NULL,
    cue_id VARCHAR NOT NULL,
    reason TEXT NOT NULL,
    PRIMARY KEY (target_id, revision, cue_id),
    FOREIGN KEY(target_id) REFERENCES target (id)
);

CREATE TABLE dialogue (
    id VARCHAR NOT NULL,
    revision INTEGER NOT NULL,
    namespace VARCHAR NOT NULL,
    source_id VARCHAR NOT NULL,
    language VARCHAR NOT NULL,
    payload TEXT NOT NULL,
    target_ids TEXT NOT NULL,
    PRIMARY KEY (id, revision),
    UNIQUE (namespace, source_id, revision)
);

CREATE TABLE exposure (
    id VARCHAR NOT NULL,
    target_id VARCHAR NOT NULL,
    revision INTEGER NOT NULL,
    payload TEXT NOT NULL,
    received_at VARCHAR NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY(target_id) REFERENCES target (id)
);

CREATE TABLE learner_recording (
    id VARCHAR NOT NULL,
    target_id VARCHAR NOT NULL,
    attempt_id VARCHAR,
    exposure_id VARCHAR,
    metadata_json TEXT NOT NULL,
    audio BLOB NOT NULL,
    sha256 VARCHAR NOT NULL,
    received_at VARCHAR NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY(target_id) REFERENCES target (id),
    FOREIGN KEY(attempt_id) REFERENCES practice_attempt (id),
    FOREIGN KEY(exposure_id) REFERENCES exposure (id)
);

CREATE TABLE lesson (
    id VARCHAR NOT NULL,
    namespace VARCHAR NOT NULL,
    source_id VARCHAR NOT NULL,
    language VARCHAR NOT NULL,
    title TEXT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (namespace, source_id)
);

CREATE TABLE lesson_membership (
    lesson_id VARCHAR NOT NULL,
    target_id VARCHAR NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY (lesson_id, target_id),
    FOREIGN KEY(lesson_id) REFERENCES lesson (id),
    FOREIGN KEY(target_id) REFERENCES target (id)
);

CREATE TABLE media (
    id VARCHAR NOT NULL,
    sha256 VARCHAR NOT NULL,
    byte_length INTEGER NOT NULL,
    mime_type VARCHAR NOT NULL,
    metadata_json TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE practice_attempt (
    id VARCHAR NOT NULL,
    session_id VARCHAR NOT NULL,
    target_id VARCHAR NOT NULL,
    step INTEGER NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (session_id, step),
    FOREIGN KEY(session_id) REFERENCES practice_session (id),
    FOREIGN KEY(target_id) REFERENCES target (id)
);

CREATE TABLE practice_event (
    id VARCHAR NOT NULL,
    session_id VARCHAR NOT NULL,
    sequence INTEGER NOT NULL,
    payload TEXT NOT NULL,
    receipt TEXT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (session_id, sequence),
    FOREIGN KEY(session_id) REFERENCES practice_session (id)
);

CREATE TABLE "practice_session" (
    id VARCHAR NOT NULL,
    start_payload TEXT NOT NULL,
    manifest TEXT NOT NULL,
    status VARCHAR NOT NULL,
    open_slot INTEGER,
    version INTEGER NOT NULL,
    created_at VARCHAR NOT NULL,
    updated_at VARCHAR NOT NULL,
    cursor INTEGER DEFAULT '0' NOT NULL,
    last_sequence INTEGER DEFAULT '0' NOT NULL,
    queue TEXT DEFAULT '[]' NOT NULL,
    repairs TEXT DEFAULT '[]' NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT one_open_practice_session UNIQUE (open_slot),
    CONSTRAINT session_slot_matches_status CHECK ((status IN ('active', 'paused') AND open_slot IS 1) OR (status IN ('superseded', 'completed') AND open_slot IS NULL))
);

CREATE TABLE preparation_job (
    id VARCHAR NOT NULL,
    input_hash VARCHAR NOT NULL,
    payload TEXT NOT NULL,
    base_bindings TEXT NOT NULL,
    status VARCHAR NOT NULL,
    attempts INTEGER NOT NULL,
    lease_token VARCHAR,
    lease_until VARCHAR,
    retry_at VARCHAR NOT NULL,
    error VARCHAR,
    result TEXT,
    review TEXT,
    published_import_id VARCHAR,
    PRIMARY KEY (id),
    UNIQUE (input_hash)
);

CREATE TABLE review_receipt (
    group_id VARCHAR NOT NULL,
    target_id VARCHAR NOT NULL,
    attempt_id VARCHAR NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY (group_id),
    FOREIGN KEY(target_id) REFERENCES target (id),
    FOREIGN KEY(attempt_id) REFERENCES practice_attempt (id)
);

CREATE TABLE source_handoff (
    export_id VARCHAR NOT NULL,
    payload_hash VARCHAR NOT NULL,
    payload TEXT NOT NULL,
    preparation_id VARCHAR NOT NULL,
    PRIMARY KEY (export_id)
);

CREATE TABLE source_item (
    namespace VARCHAR NOT NULL,
    source_id VARCHAR NOT NULL,
    target_id VARCHAR NOT NULL,
    source_revision VARCHAR NOT NULL,
    PRIMARY KEY (namespace, source_id),
    FOREIGN KEY(target_id) REFERENCES target (id)
);

CREATE TABLE target (
    id VARCHAR NOT NULL,
    language VARCHAR NOT NULL,
    kind VARCHAR NOT NULL,
    meaning_key VARCHAR NOT NULL,
    recall_expression TEXT NOT NULL,
    current_revision INTEGER NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE target_revision (
    target_id VARCHAR NOT NULL,
    revision INTEGER NOT NULL,
    payload TEXT NOT NULL,
    fingerprint VARCHAR NOT NULL,
    created_at VARCHAR NOT NULL,
    PRIMARY KEY (target_id, revision),
    FOREIGN KEY(target_id) REFERENCES target (id)
);

CREATE TABLE target_schedule (
    target_id VARCHAR NOT NULL,
    version INTEGER NOT NULL,
    due_at VARCHAR NOT NULL, memory_state TEXT, policy_version VARCHAR,
    PRIMARY KEY (target_id),
    FOREIGN KEY(target_id) REFERENCES target (id)
);

INSERT INTO alembic_version (version_num) VALUES ('0005');
