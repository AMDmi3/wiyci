CREATE TABLE bulk_update_status (
    last_project_name TEXT,
    next_update_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO bulk_update_status DEFAULT VALUES;
