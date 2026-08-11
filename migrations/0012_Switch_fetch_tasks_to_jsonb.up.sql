ALTER TABLE fetch_tasks ADD COLUMN params JSONB NOT NULL DEFAULT '{}'::JSONB;
UPDATE fetch_tasks SET params = jsonb_build_object('url', url);
ALTER TABLE fetch_tasks ALTER COLUMN params DROP DEFAULT;
CREATE UNIQUE INDEX fetch_tasks_project_name_params_idx ON fetch_tasks(project_name, params);
DROP INDEX fetch_tasks_project_name_url_idx;
ALTER TABLE fetch_tasks DROP COLUMN url;
