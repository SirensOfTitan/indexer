ALTER TABLE file ADD COLUMN last_updated DATETIME;

CREATE TRIGGER update_last_updated_trigger
AFTER UPDATE ON file
FOR EACH ROW
BEGIN
    UPDATE file SET last_updated = CURRENT_TIMESTAMP WHERE rowid = NEW.rowid;
END;
