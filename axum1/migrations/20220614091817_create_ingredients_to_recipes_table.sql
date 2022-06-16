-- The relationship between recipes and ingredients is what is known as a many-to-many relationship - a recipe
-- might contain any number of ingredients, while an ingredient might be used by any number of recipes.
--
-- In a relational database, the way to create a many-to-many relationship is by introducing a bridge table.
-- 
-- In the case of recipes and ingredients, you will have three tables.
-- One table for ingredients, specifying the name of the ingredient and possibly other ingredient related data.
-- Another table for recipes, specifying the name of the recipe, a description, the text explanation etc..
-- Then you have this bridge table, `ingredients_to_recipes`, that will contain a one-to-many foreign key to the recipe table,
-- a one-to-many foreign key to the ingredient table, and the quantity needed for that specific ingredient in that specific recipe.

CREATE TABLE ingredients_to_recipes
(
    recipe_id     UUID NOT NULL REFERENCES "recipes"     (id) ON DELETE CASCADE,
    ingredient_id UUID NOT NULL REFERENCES "ingredients" (id) ON DELETE CASCADE,
    -- TODO: make this a number of some sort
    quantity      TEXT NOT NULL COLLATE "case_insensitive",
    -- TODO: make it an enum maybe?
    quantity_unit TEXT NOT NULL COLLATE "case_insensitive",
    PRIMARY KEY (recipe_id, ingredient_id)
);
