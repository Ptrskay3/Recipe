CREATE TABLE ingredients_to_recipes
(
    recipe_id    UUID NOT NULL REFERENCES "recipes"     (id) ON DELETE CASCADE,
    ingredient_id UUID NOT NULL REFERENCES "ingredients" (id) ON DELETE CASCADE,
    quantity TEXT NOT NULL,
    quantity_unit TEXT NOT NULL,
    PRIMARY KEY (reciple_id, ingredient_id)
);
