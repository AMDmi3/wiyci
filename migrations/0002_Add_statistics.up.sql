CREATE TABLE statistics (
	stored_logs_size      BIGINT NOT NULL DEFAULT 0
);

INSERT INTO statistics DEFAULT VALUES;
UPDATE statistics SET stored_logs_size = (SELECT COALESCE(SUM(size), 0) FROM logs);
