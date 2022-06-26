CREATE TABLE confirmation_tokens(
   confirmation_token TEXT NOT NULL,
   user_id UUID NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
   PRIMARY KEY (confirmation_token)
);
