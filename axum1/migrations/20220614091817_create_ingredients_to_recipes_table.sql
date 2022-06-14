CREATE TABLE ingredients_to_recipes
(
    recipe_id     UUID NOT NULL REFERENCES "recipes"     (id) ON DELETE CASCADE,
    ingredient_id UUID NOT NULL REFERENCES "ingredients" (id) ON DELETE CASCADE,
    -- TODO: make this a number of some sort
    quantity      TEXT NOT NULL COLLATE "case_insensitive",
    -- TODO: make it an enum
    quantity_unit TEXT NOT NULL COLLATE "case_insensitive",
    PRIMARY KEY (reciple_id, ingredient_id)
);
