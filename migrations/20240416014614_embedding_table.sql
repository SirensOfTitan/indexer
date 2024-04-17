CREATE TABLE file_embeddings (
    file_path BLOB,
    embedding BLOB
);

CREATE VIRTUAL TABLE vss_file_embeddings using vss0 (
    embedding(384)
);

CREATE TRIGGER update_vss_file_embeddings
AFTER INSERT ON file_embeddings
FOR EACH ROW
BEGIN
    DELETE FROM vss_file_embeddings
    WHERE rowid = NEW.rowid;

    INSERT INTO vss_file_embeddings (rowid, embedding)
    SELECT NEW.rowid, NEW.embedding
    WHERE NOT EXISTS (
        SELECT 1 FROM vss_file_embeddings WHERE rowid = NEW.rowid
    );
END;

-- Delete Trigger
CREATE TRIGGER delete_vss_file_embeddings
AFTER DELETE ON file_embeddings
FOR EACH ROW
BEGIN
    DELETE FROM vss_file_embeddings
    WHERE rowid = OLD.rowid;
END;
