use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::{error::DatabaseError, PgPool};

/// Attempts to register the provided user with the given password. Returns
/// `Ok(true)` if the user was registered, `Ok(false)` if a user with the given
/// username already existed, and `Err` if a miscellaneous SQL error occurred.
pub async fn register(pg: &PgPool, username: &str, password: &str) -> Result<bool, sqlx::Error> {
    let (password, salt) = hash(password);

    let result = sqlx::query!(
            "INSERT INTO users (username, password, salt) VALUES ($1, $2, $3)",
            username, &password, &salt
        )
        .execute(pg)
        .await;

    // User registered, good!
    let Err(err) = result else {
        return Ok(true);
    };
    
    // Filter for "duplicate key value violates unique constraint"
    // (here, it means the user already exists)
    let user_already_exists = err.as_database_error()
        .and_then(DatabaseError::code)
        .filter(|code| code == "23505")
        .is_some();

    if user_already_exists {
        Ok(false)
    } else {
        Err(err)
    }
    
}

/// Checks whether or not the provided password is valid for the given username.
/// If the user exists, returns whether or not the provided password is valid.
/// If there is an SQL error (including the user not existing), `Err` is
/// returned. 
pub async fn login(pg: &PgPool, username: &str, password: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
            "SELECT password, salt FROM users WHERE username = $1",
            username
        )
        .fetch_one(pg)
        .await?;

    let salt = result.salt.as_slice();
    let correct_hash = result.password.as_slice();

    Ok(hash_salt(password, salt) == correct_hash) 
}

/// Hashes the provided password, generating a salt, hashing the password with
/// it, and then returning both.
/// 
/// See [hash_salt] for a note on the hashing function used.
pub fn hash(password: &str) -> ([u8; 32], [u8; 8]) {
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
pub fn hash_salt(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    hasher.update(password);
    hasher.update(salt);

    hasher.finalize().into()
}
