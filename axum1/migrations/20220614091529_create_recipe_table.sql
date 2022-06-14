CREATE TABLE recipes
(
    id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    name TEXT NOT NULL COLLATE "case_insensitive" UNIQUE,
    description TEXT NOT NULL,
    creator_id UUID NOT NULL REFERENCES "users" (user_id) ON DELETE CASCADE
    -- TODO: course, difficulty, prep time, cook time, category, steps..
);
