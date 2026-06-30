-- SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
-- SPDX-License-Identifier: GPL-3.0-or-later

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE projects (
    name       TEXT NOT NULL PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX projects_name_trgm ON projects USING gin (name gin_trgm_ops);
