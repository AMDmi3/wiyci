ALTER TABLE projects RENAME COLUMN snippet_counts TO max_snippet_counts;
ALTER TABLE projects ADD COLUMN latest_snippet_counts JSONB;
-- omitting SQL to prefill latest_snippet_counts, these will be filled on the next project update
