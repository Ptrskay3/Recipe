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
    'beverages'
);

CREATE TABLE "ingredients"
(
    id  UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    name TEXT COLLATE "case_insensitive" UNIQUE NOT NULL,
    category food_category[] NOT NULL,
    calories_per_100g DECIMAL NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    creator_id UUID REFERENCES users (user_id) DEFAULT NULL
);

INSERT INTO ingredients (name, category, calories_per_100g) VALUES ('hazelnut', '{nuts_and_seeds}', 628.3);
INSERT INTO ingredients (name, category, calories_per_100g) VALUES ('apple', '{fruit}', 52.1);

SELECT trigger_updated_at('"ingredients"');
