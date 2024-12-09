-- Set up the database for PDForum

-- Create a user and a database
CREATE ROLE pdforum LOGIN PASSWORD 'secret';
CREATE DATABASE pdforum ENCODING 'UTF8' OWNER pdforum;

-- Create the table for users
CREATE TABLE "users" (
    id         SERIAL PRIMARY KEY,

    username   TEXT UNIQUE NOT NULL,
    
    bio        TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    password   BYTEA NOT NULL,
    salt       BYTEA NOT NULL
);