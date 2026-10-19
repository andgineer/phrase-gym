CREATE TABLE profile_revision (
    revision INTEGER PRIMARY KEY CHECK (revision > 0),
    mutation_id TEXT UNIQUE,
    request TEXT NOT NULL,
    payload TEXT NOT NULL
);
