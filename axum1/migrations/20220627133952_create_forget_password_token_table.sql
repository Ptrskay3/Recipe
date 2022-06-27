CREATE TABLE forget_password_tokens(
   token UUID NOT NULL,
   user_id UUID NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
   created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
   PRIMARY KEY (token)
);
