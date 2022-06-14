CREATE TABLE "users"
(
    user_id  UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),

    name          TEXT COLLATE "case_insensitive" UNIQUE NOT NULL,
  
    email         TEXT COLLATE "case_insensitive" UNIQUE NOT NULL,

    password_hash TEXT                                   NOT NULL,
  
    created_at    TIMESTAMPTZ                            NOT NULL DEFAULT NOW(),

    updated_at    TIMESTAMPTZ
);

SELECT trigger_updated_at('"users"');
