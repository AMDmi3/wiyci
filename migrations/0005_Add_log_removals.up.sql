CREATE TABLE logs_to_remove (
    id         INTEGER NOT NULL PRIMARY KEY,
    size       INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE FUNCTION on_delete_log()
RETURNS trigger AS $$
BEGIN
    INSERT INTO logs_to_remove(id, size)
         VALUES (OLD.id, OLD.size);
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_log_delete
         AFTER DELETE
            ON logs
           FOR EACH ROW
       EXECUTE FUNCTION on_delete_log();
