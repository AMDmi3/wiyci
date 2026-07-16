ALTER TABLE projects ADD COLUMN snippet_counts JSONB;

WITH
    new_counts_by_project_key AS (
        SELECT project_name
             , key
             , MAX(value::BIGINT) AS value
          FROM logs, jsonb_each(parsed_snippet_counts) AS counts(key, value)
         GROUP BY project_name, key
    )
  , new_counts_by_project AS (
        SELECT project_name AS name
             , jsonb_object_agg(key, value) AS snippet_counts
          FROM new_counts_by_project_key
         GROUP BY project_name
    )
UPDATE projects
   SET snippet_counts = new_counts_by_project.snippet_counts
  FROM new_counts_by_project
 WHERE projects.name = new_counts_by_project.name
