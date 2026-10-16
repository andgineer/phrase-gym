CREATE TABLE alembic_version (
    version_num VARCHAR(32) NOT NULL,
    CONSTRAINT alembic_version_pkc PRIMARY KEY (version_num)
);

CREATE TABLE app_settings (
    id INTEGER NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE content_import (
    id VARCHAR NOT NULL,
    payload TEXT NOT NULL,
    report TEXT NOT NULL,
    status VARCHAR NOT NULL,
    created_at VARCHAR NOT NULL,
    PRIMARY KEY (id)
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
