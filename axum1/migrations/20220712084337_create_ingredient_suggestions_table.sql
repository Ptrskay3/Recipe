CREATE TABLE "ingredient_suggestions"
(
    id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    ingredient_id UUID REFERENCES ingredients (id)      ON DELETE CASCADE,
    user_id       UUID REFERENCES users       (user_id) ON DELETE CASCADE,
    is_delete_vote BOOLEAN DEFAULT 'FALSE',
    name TEXT,
    category food_category[],
    calories_per_100g REAL,
    original_name TEXT,
    protein REAL,
    water REAL,
    fat REAL,
    sugar REAL,
    carbohydrate REAL,
    fiber REAL,
    caffeine REAL,
    contains_alcohol BOOLEAN,
    g_per_piece REAL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    UNIQUE (ingredient_id, user_id)
);

SELECT trigger_updated_at('"ingredient_suggestions"');
