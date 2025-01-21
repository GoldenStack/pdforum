-- PDForum PSQL database setup
-- Ideally you just run this once to set it up, but you never know

-- Create a user and a database
CREATE ROLE pdforum LOGIN PASSWORD 'secret';
CREATE DATABASE pdforum ENCODING 'UTF8' OWNER pdforum;

-- Now actually enter the context of the database (I don't know how else to
-- describe it) before running the rest of the commands.

-- Create the table for users
CREATE TABLE "users" (
    id         SERIAL PRIMARY KEY,

    username   TEXT UNIQUE NOT NULL,
    
    bio        TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    password   BYTEA NOT NULL,
    salt       BYTEA NOT NULL
);

-- Create the table for posts
CREATE TABLE "posts" (
    id         SERIAL PRIMARY KEY,

    author_id  INTEGER NOT NULL REFERENCES users(id),

    content    TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    likes      INTEGER NOT NULL DEFAULT 0
);

-- create the table for likes
create table "likes" (
    id        serial primary key,

    author_id integer not null references users(id),
    post_id   integer not null references posts(id),

    unique (author_id, post_id)
);

-- Create a function that adds a certain amount to the likes for a post
CREATE FUNCTION modify_likes(post_id INTEGER, change INTEGER) RETURNS INTEGER
    AS $$
        UPDATE posts
            SET likes = likes + change
        WHERE id = post_id
        RETURNING likes
    $$ LANGUAGE SQL;

-- Trigger function for incrementing the like count
CREATE FUNCTION increment_likes() RETURNS TRIGGER
    AS $$ BEGIN
        PERFORM modify_likes(NEW.post_id, 1);
        RETURN NEW;
    END $$
    LANGUAGE plpgsql;    

-- Trigger to increment likes for the post after adding a like
CREATE TRIGGER increment_likes_on_insert
    AFTER INSERT ON likes
    FOR EACH ROW
    EXECUTE FUNCTION increment_likes();