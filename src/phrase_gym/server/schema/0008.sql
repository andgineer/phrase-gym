CREATE TABLE settings_document (id INTEGER PRIMARY KEY CHECK(id=1), document TEXT NOT NULL);
INSERT INTO settings_document SELECT id, json_object(
  'native_language', native_language, 'learning_timezone', learning_timezone,
  'card_language', card_language, 'help_language', help_language,
  'daily_due_limit', daily_due_limit, 'daily_new_limit', daily_new_limit) FROM settings;
DROP TABLE settings;
ALTER TABLE settings_document RENAME TO settings;
