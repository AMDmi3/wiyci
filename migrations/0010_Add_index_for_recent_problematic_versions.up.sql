CREATE INDEX versions_recent_problematic_idx
          ON versions (created_at DESC)
       WHERE (versions.max_snippet_counts->>'CompilerWarning')::integer > 0
          OR (versions.max_snippet_counts->>'FailedTest')::integer > 0
