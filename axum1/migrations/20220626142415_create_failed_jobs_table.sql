CREATE TABLE failed_jobs(
    -- TODO: obviously this structure is very rudimentary
    job_id TEXT NOT NULL PRIMARY KEY,
    job_type TEXT NOT NULL,
    context JSON,
    failed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
