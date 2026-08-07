-- At this point, fetch task kind is only used to split tasks between copies of the same
-- worker to increase concurrency between fetches from different hosts, so it does not
-- matter which kind we actully use here as a filler.
DROP INDEX fetch_tasks_queue_idx;
ALTER TABLE fetch_tasks ADD COLUMN kind TEXT NOT NULL DEFAULT 'Alpine';
ALTER TABLE fetch_tasks ALTER COLUMN kind DROP DEFAULT;
CREATE UNIQUE INDEX fetch_tasks_queue_idx ON fetch_tasks(kind, next_fetch_attempt_at, id) WHERE next_fetch_attempt_at IS NOT NULL;
