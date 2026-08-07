DROP INDEX fetch_tasks_queue_idx;
ALTER TABLE fetch_tasks DROP COLUMN kind;
CREATE UNIQUE INDEX fetch_tasks_queue_idx ON fetch_tasks(next_fetch_attempt_at, id) WHERE next_fetch_attempt_at IS NOT NULL;
