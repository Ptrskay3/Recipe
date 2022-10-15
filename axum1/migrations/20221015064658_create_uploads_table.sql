CREATE TABLE "uploads"
(
    uploader_id  UUID NOT NULL REFERENCES "users" (user_id) ON DELETE CASCADE,

    file_name    TEXT NOT NULL,
  
    bytes        REAL NOT NULL,

    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at   TIMESTAMPTZ,

    PRIMARY KEY(uploader_id, file_name)
);

SELECT trigger_updated_at('"uploads"');
