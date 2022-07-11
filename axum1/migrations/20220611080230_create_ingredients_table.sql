CREATE TYPE food_category as ENUM (
    'vegetable',
    'fruit',
    'meat',
    'dairy',
    'grains',
    'legumes',
    'baked',
    'eggs',
    'seafood',
    'nuts_and_seeds',
    'herbs_and_spices',
    'garnishes',
    'deserts_and_sweets',
    'supplements',
    'beverages',
    'uncategorized'
);

CREATE TABLE "ingredients"
(
    id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    name TEXT COLLATE "case_insensitive" UNIQUE NOT NULL,
    category food_category[] NOT NULL DEFAULT '{uncategorized}',
    calories_per_100g REAL NOT NULL,
    original_name TEXT NOT NULL,
    protein REAL NOT NULL,
    water REAL NOT NULL,
    fat REAL NOT NULL,
    sugar REAL NOT NULL,
    carbohydrate REAL NOT NULL,
    fiber REAL NOT NULL,
    caffeine REAL NOT NULL,
    contains_alcohol BOOLEAN NOT NULL,
    g_per_piece REAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    creator_id UUID REFERENCES users (user_id) DEFAULT NULL
);

SELECT trigger_updated_at('"ingredients"');
