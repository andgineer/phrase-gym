ALTER TABLE phrases ADD COLUMN language TEXT NOT NULL DEFAULT 'en';
CREATE INDEX phrases_by_language ON phrases(language, lesson_id);
ALTER TABLE practice_log ADD COLUMN language TEXT NOT NULL DEFAULT 'en';
CREATE INDEX practice_log_by_language ON practice_log(language, learning_date, mode, kind);
CREATE TABLE language_extra_targets (
  learning_date TEXT NOT NULL, language TEXT NOT NULL, due_target INTEGER NOT NULL,
  new_target INTEGER NOT NULL, PRIMARY KEY (learning_date, language)
);
INSERT INTO language_extra_targets
  SELECT learning_date, 'en', due_target, new_target FROM extra_targets;
DROP TABLE extra_targets;
ALTER TABLE language_extra_targets RENAME TO extra_targets;
CREATE TABLE language_views (
  language TEXT PRIMARY KEY, traversal_id TEXT NOT NULL,
  learning_date TEXT NOT NULL, origin TEXT NOT NULL, lesson_id TEXT,
  skipped_json TEXT NOT NULL, phrase_id TEXT, view_token TEXT, kind TEXT,
  practice INTEGER NOT NULL, revealed INTEGER NOT NULL, cue_index INTEGER,
  hint_index INTEGER, operations_json TEXT NOT NULL
);
INSERT INTO language_views SELECT 'en', traversal_id, learning_date, origin, lesson_id,
  skipped_json, phrase_id, view_token, kind, practice, revealed, cue_index, hint_index,
  operations_json FROM current_view;
DROP TABLE current_view;
ALTER TABLE language_views RENAME TO current_view;
CREATE INDEX current_view_by_token ON current_view(view_token);
