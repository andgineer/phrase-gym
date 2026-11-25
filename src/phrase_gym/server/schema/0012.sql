ALTER TABLE phrases ADD COLUMN instruction TEXT;
ALTER TABLE imports ADD COLUMN dismissed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE imports ADD COLUMN correction_json TEXT;
DELETE FROM import_notices WHERE package_id IN (
  SELECT package_id FROM imports WHERE kind='coach' AND status='error' AND retry_helps=0);
