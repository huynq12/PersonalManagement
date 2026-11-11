-- Initial migration: enable shared extensions.
-- citext: case-insensitive text, used for the email column.
CREATE EXTENSION IF NOT EXISTS citext;
