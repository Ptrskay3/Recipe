CREATE TABLE recipes
(
    id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    name TEXT NOT NULL COLLATE "case_insensitive" UNIQUE,
    description TEXT NOT NULL,
    creator_id UUID NOT NULL REFERENCES "users" (user_id) ON DELETE CASCADE,
    -- TODO: other fields such as course, difficulty, prep time, cook time, category, steps..
    created_at    TIMESTAMPTZ                            NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ
);

SELECT trigger_updated_at('"recipes"');
