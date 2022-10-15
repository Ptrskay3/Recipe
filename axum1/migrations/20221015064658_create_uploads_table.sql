CREATE TABLE "uploads"
(
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),

    uploader_id  UUID NOT NULL REFERENCES "users" (user_id) ON DELETE CASCADE,

    file_name    TEXT NOT NULL,
  
    bytes        REAL NOT NULL,

    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at   TIMESTAMPTZ
);

SELECT trigger_updated_at('"uploads"');
