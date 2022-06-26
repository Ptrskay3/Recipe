CREATE TABLE confirmation_delivery_queue (
   confirmation_id TEXT NOT NULL REFERENCES confirmation_tokens (confirmation_token),
   user_email TEXT NOT NULL,
   PRIMARY KEY(confirmation_id, user_email)
);
