ALTER TABLE users ADD CONSTRAINT unique_oauth UNIQUE (oauth_id, oauth_provider);
