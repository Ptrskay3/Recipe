CREATE TABLE favorite_ingredient
(
    -- By default, "references" (foreign key) relationships will throw errors if the row in the referenced table
    -- is deleted before this one. The `on delete cascade` clause causes this to be automatically deleted if the
    -- corresponding user row is deleted.
    --
    -- Before applying `on delete cascade` to a foreign key clause, though, you should consider the actual semantics
    -- of that table. Does it, for example, contain purchase records that are linked to a payment processor? You may not
    -- want to delete those records for auditing purposes, even if you want to delete the user record itself.
    --
    -- In cases like that, I usually just forego the foreign-key clause and treat the user ID as a plain data column
    -- so the row sticks around even if the user is deleted. There's also `on delete set null` but then that
    -- requires the column to be nullable which makes it unwieldy in queries when it should not be null 99% of the time.
    ingredient_id           UUID NOT NULL REFERENCES "ingredients" (id) ON DELETE CASCADE,

    user_id                 UUID NOT NULL REFERENCES "users" (user_id)  ON DELETE CASCADE,

    created_at TIMESTAMPTZ       NOT NULL DEFAULT NOW(),

    -- We don't really need an `updated_at` column because there isn't anything to update here.
    -- However, columns with nulls take up very little extra space on-disk in Postgres so it's worth adding
    -- for posterity anyway. In one project that had a "follow this user" feature, there was extra mutable data
    -- on the row in the "follow" table, so there are practical reasons to have this column.
    --
    -- It can also serve as a canary for queries that are modifying this table in weird ways (as normally you'd
    -- expect this to always be null, so seeing this set to a value may be a red flag).
    updated_at TIMESTAMPTZ,

    PRIMARY KEY (ingredient_id, user_id)
);

SELECT trigger_updated_at('favorite_ingredient');
