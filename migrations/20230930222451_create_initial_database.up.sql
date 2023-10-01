-- Add up migration script here
CREATE USER app;
CREATE TABLE IF NOT EXISTS profiles
(
    id     UUID    DEFAULT gen_random_uuid(),
    user_name   CHAR[32],
    email       VARCHAR,
    create_date TIMESTAMP,
    update_date TIMESTAMP,
    hash        bytea,
    update_hash BOOLEAN DEFAULT FALSE,
    PRIMARY KEY(id)
);
CREATE INDEX BTREE_user_name ON profiles (user_name);
CREATE INDEX BTREE_email ON profiles (email);
CREATE INDEX BTREE_create_date ON profiles (create_date);
CREATE INDEX BTREE_update_date ON profiles (update_date);
