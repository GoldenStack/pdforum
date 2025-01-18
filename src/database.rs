use std::env;

use anyhow::Result;
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::{error::DatabaseError, postgres::PgPoolOptions, types::time::OffsetDateTime, PgPool};

use crate::routes::Post;

/// Opens a connection to the Postgres database.
/// This uses the Postgres connection string defined in the `DATABASE_URL`
/// environment variable.
pub async fn open_connection() -> Result<PgPool> {
    let database_url = env::var("DATABASE_URL")?;

    let db = PgPoolOptions::new()
        .max_connections(16)
        .connect(&database_url)
        .await?;

    Ok(db)
}

/// Attempts to register the provided user with the given password. Returns
/// `Ok(Some(id))` if the user was registered, `Ok(None)` if a user with the given
/// username already existed, and `Err` if a miscellaneous SQL error occurred,
/// where the returned ID is the unique integer ID referring to the user.
pub async fn register(
    pg: &PgPool,
    username: &str,
    password: &str,
) -> Result<Option<i32>, sqlx::Error> {
    let (password, salt) = hash(password);

    let result = sqlx::query!(
        "INSERT INTO users (username, password, salt) VALUES ($1, $2, $3) RETURNING id",
        username,
        &password,
        &salt
    )
    .fetch_one(pg)
    .await;

    // User registered, good!
    let err = match result {
        Ok(record) => return Ok(Some(record.id)),
        Err(err) => err,
    };

    // Filter for "duplicate key value violates unique constraint"
    // (here, it means the user already exists)
    let user_already_exists = err
        .as_database_error()
        .and_then(DatabaseError::code)
        .filter(|code| code == "23505")
        .is_some();

    if user_already_exists {
        Ok(None)
    } else {
        Err(err)
    }
}

/// Checks whether or not the provided password is valid for the given username.
/// If the user exists, returns the ID of the user (`Ok(Some(id))`). If the user
/// doesn't exist, returns nothing.
/// If there is an SQL error (including the user not existing), `Err` is
/// returned.
pub async fn login(
    pg: &PgPool,
    username: &str,
    password: &str,
) -> Result<Option<i32>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT id, password, salt FROM users WHERE username = $1",
        username
    )
    .fetch_one(pg)
    .await?;

    let salt = result.salt.as_slice();
    let correct_hash = result.password.as_slice();

    if hash_salt(password, salt) == correct_hash {
        Ok(Some(result.id))
    } else {
        Ok(None)
    }
}

/// Publishes a message as the given author, returning the ID of the posted
/// message (or an SQL error if one occurred).
pub async fn publish(pg: &PgPool, author: i32, content: &str) -> Result<i32, sqlx::Error> {
    let result = sqlx::query!(
        "INSERT INTO posts (author, content) VALUES ($1, $2) RETURNING id",
        author,
        content
    )
    .fetch_one(pg)
    .await?;

    Ok(result.id)
}

/// Returns a page of browsing results for an arbitrary user.
pub async fn browse(pg: &PgPool) -> Result<Vec<Post>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT posts.id, posts.content, posts.created_at, users.username
        FROM posts
        INNER JOIN users
        ON posts.author=users.id
        ORDER BY posts.created_at DESC
        FETCH NEXT 10 ROWS ONLY",
    )
    .fetch_all(pg)
    .await?;

    Ok(result
        .into_iter()
        .map(|record| Post {
            id: record.id,
            author: record.username,
            created_at: record.created_at,
            content: record.content,
        })
        .collect::<Vec<_>>())
}

/// Retrieves a post based on its ID, returning None if there isn't such a post.
pub async fn retrieve_post(pg: &PgPool, post_id: i32) -> Result<Option<Post>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT posts.id, posts.content, posts.created_at, users.username
        FROM posts
        INNER JOIN users
        ON posts.author=users.id
        WHERE posts.id=$1
        ",
        post_id
    )
    .fetch_optional(pg)
    .await?;

    Ok(result.map(|record| Post {
        id: record.id,
        author: record.username,
        created_at: record.created_at,
        content: record.content,
    }))
}

/// Hashes the provided password, generating a salt, hashing the password with
/// it, and then returning both.
///
/// See [hash_salt] for a note on the hashing function used.
fn hash(password: &str) -> ([u8; 32], [u8; 8]) {
    let mut salt = [0; 8];

    OsRng.fill_bytes(&mut salt);

    let hash = hash_salt(password, &salt);

    (hash, salt)
}

/// Hashes the provided password and salt with Sha256.
///
/// [Argon2id] would be used here, but unfortunately it takes up too much time for
/// my poor $3.5/mo Hetzner VPS to handle, so don't rely on this being too
/// secure. Sorry!
///
/// [Argon2id]: https://en.wikipedia.org/wiki/Argon2
fn hash_salt(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    hasher.update(password);
    hasher.update(salt);

    hasher.finalize().into()
}
