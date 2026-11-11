ALTER TABLE imports ADD COLUMN kind TEXT NOT NULL DEFAULT 'package';
INSERT OR REPLACE INTO import_notices (package_id, created_at)
  SELECT package_id, strftime('%Y-%m-%dT%H:%M:%f+00:00', 'now') FROM imports
  WHERE status IN ('pending', 'preparing');
UPDATE imports SET status='error', error='This package was saved without reference audio, '
  || 'which Phrase Gym no longer prepares for packages. Its content is kept. Ask your AI agent '
  || 'to submit it again with complete audio under a new package_id.'
  WHERE status IN ('pending', 'preparing');
