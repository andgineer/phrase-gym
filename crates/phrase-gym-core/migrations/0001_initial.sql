CREATE TABLE target (
    id TEXT PRIMARY KEY,
    language TEXT NOT NULL CHECK (language IN ('en', 'ru', 'sr', 'de')),
    kind TEXT NOT NULL CHECK (kind IN ('fixed_phrase', 'word_sense', 'frame')),
    meaning_key TEXT NOT NULL,
    recall_expression TEXT NOT NULL,
    current_revision INTEGER NOT NULL CHECK (current_revision >= 1)
) STRICT;

CREATE INDEX target_by_language ON target (language, id);

CREATE TABLE target_revision (
    target_id TEXT NOT NULL REFERENCES target (id),
    revision INTEGER NOT NULL CHECK (revision >= 1),
    payload TEXT NOT NULL CHECK (json_valid(payload)),
    fingerprint TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (target_id, revision)
) STRICT;

CREATE TABLE source_item (
    namespace TEXT NOT NULL,
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL REFERENCES target (id),
    source_revision TEXT NOT NULL,
    PRIMARY KEY (namespace, source_id)
) STRICT;

CREATE INDEX source_item_by_target ON source_item (target_id);

CREATE TABLE lesson (
    id TEXT PRIMARY KEY,
    namespace TEXT NOT NULL,
    source_id TEXT NOT NULL,
    language TEXT NOT NULL CHECK (language IN ('en', 'ru', 'sr', 'de')),
    title TEXT NOT NULL,
    UNIQUE (namespace, source_id)
) STRICT;

CREATE INDEX lesson_by_language ON lesson (language, title, id);

CREATE TABLE lesson_membership (
    lesson_id TEXT NOT NULL REFERENCES lesson (id),
    target_id TEXT NOT NULL REFERENCES target (id),
    position INTEGER NOT NULL CHECK (position >= 0),
    PRIMARY KEY (lesson_id, target_id),
    UNIQUE (lesson_id, position)
) STRICT;

CREATE INDEX lesson_membership_by_target ON lesson_membership (target_id);

CREATE TABLE dialogue (
    id TEXT PRIMARY KEY,
    namespace TEXT NOT NULL,
    source_id TEXT NOT NULL,
    language TEXT NOT NULL CHECK (language IN ('en', 'ru', 'sr', 'de')),
    current_revision INTEGER NOT NULL CHECK (current_revision >= 1),
    UNIQUE (namespace, source_id)
) STRICT;

CREATE INDEX dialogue_by_language ON dialogue (language, source_id);

CREATE TABLE dialogue_revision (
    dialogue_id TEXT NOT NULL REFERENCES dialogue (id),
    revision INTEGER NOT NULL CHECK (revision >= 1),
    payload TEXT NOT NULL CHECK (json_valid(payload)),
    bindings TEXT NOT NULL CHECK (json_valid(bindings)),
    PRIMARY KEY (dialogue_id, revision)
) STRICT;

CREATE TABLE staged_package (
    id TEXT PRIMARY KEY,
    package TEXT NOT NULL CHECK (json_valid(package)),
    report TEXT NOT NULL CHECK (json_valid(report)),
    status TEXT NOT NULL CHECK (status IN ('staged', 'published')),
    staged_at TEXT NOT NULL
) STRICT;

CREATE TABLE reference_media (
    id TEXT PRIMARY KEY,
    sha256 TEXT NOT NULL CHECK (length(sha256) = 64),
    byte_length INTEGER NOT NULL CHECK (byte_length > 0),
    mime_type TEXT NOT NULL,
    metadata TEXT NOT NULL CHECK (json_valid(metadata))
) STRICT;

CREATE INDEX reference_media_by_sha256 ON reference_media (sha256);

CREATE TABLE exposure (
    id TEXT PRIMARY KEY,
    target_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    payload TEXT NOT NULL CHECK (json_valid(payload)),
    received_at TEXT NOT NULL,
    FOREIGN KEY (target_id, revision) REFERENCES target_revision (target_id, revision)
) STRICT;

CREATE INDEX exposure_by_target ON exposure (target_id, revision);

CREATE TABLE target_schedule (
    target_id TEXT PRIMARY KEY REFERENCES target (id),
    version INTEGER NOT NULL CHECK (version >= 1),
    due_at TEXT NOT NULL,
    memory_state TEXT CHECK (memory_state IS NULL OR json_valid(memory_state)),
    policy_version TEXT
) STRICT;
