CREATE TRIGGER clear_virtual_table_on_hash_change
AFTER UPDATE OF hash ON file
FOR EACH ROW
BEGIN
 DELETE FROM file_embeddings WHERE file_path = NEW.path;
END;
