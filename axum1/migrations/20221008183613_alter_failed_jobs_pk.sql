ALTER TABLE failed_jobs DROP COLUMN job_id;
ALTER TABLE failed_jobs ADD COLUMN job_id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc();
