-- TODO: maybe an enum here later?
ALTER TABLE users ADD COLUMN oauth_provider TEXT;
ALTER TABLE users ADD COLUMN oauth_id TEXT;

-- TODO: also, we need to make only email unique, and change the login procedure
-- ALTER TABLE users DROP CONSTRAINT users_name_key;
