CREATE INDEX logs_expiration_idx ON logs(project_name, variant, source_pkgname, binary_pkgname) WHERE fetch_task_id IS NOT NULL
