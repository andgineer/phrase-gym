CREATE TABLE import_notices (
  package_id TEXT PRIMARY KEY REFERENCES imports(package_id), created_at TEXT NOT NULL
);
