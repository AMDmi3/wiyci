ALTER TABLE fetch_tasks ADD COLUMN url TEXT NOT NULL DEFAULT '';
UPDATE fetch_tasks SET url = params->>'url';
ALTER TABLE fetch_tasks ALTER COLUMN url DROP DEFAULT;
CREATE UNIQUE INDEX fetch_tasks_project_name_url_idx ON fetch_tasks(project_name, url);
DROP INDEX fetch_tasks_project_name_params_idx;
ALTER TABLE fetch_tasks DROP COLUMN params;
