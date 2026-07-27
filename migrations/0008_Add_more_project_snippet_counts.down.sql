ALTER TABLE projects RENAME COLUMN max_snippet_counts TO snippet_counts;
ALTER TABLE projects DROP COLUMN latest_snippet_counts;
