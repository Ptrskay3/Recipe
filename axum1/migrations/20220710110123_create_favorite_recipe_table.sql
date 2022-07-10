-- Pretty much the same as the `create_favorite_ingredient_table` migration.
CREATE TABLE favorite_recipe
(
    recipe_id           UUID NOT NULL REFERENCES "recipes" (id) ON DELETE CASCADE,

    user_id                 UUID NOT NULL REFERENCES "users" (user_id)  ON DELETE CASCADE,

    created_at TIMESTAMPTZ       NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ,

    PRIMARY KEY (recipe_id, user_id)
);

SELECT trigger_updated_at('favorite_recipe');

CREATE OR REPLACE FUNCTION toggle_favorite_recipe(
    usid UUID, 
    rid UUID
) 
RETURNS NUMERIC AS $$
DECLARE
    row_exists NUMERIC;
BEGIN

    SELECT 1 
    INTO row_exists 
    FROM favorite_recipe 
    WHERE user_id = usid AND recipe_id = rid;

    IF (row_exists > 0) THEN
        DELETE FROM favorite_recipe WHERE user_id = usid AND recipe_id = rid;
        RETURN 0;
    ELSE
        INSERT INTO favorite_recipe (recipe_id, user_id) VALUES (rid, usid);
        RETURN 1;
    END IF;

END; 
$$
LANGUAGE plpgsql;
