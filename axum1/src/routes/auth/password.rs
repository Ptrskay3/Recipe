use anyhow::Context;
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use secrecy::{ExposeSecret, Secret};

use crate::{error::ApiError, extractors::DatabaseConnection};

use super::Credentials;

pub async fn validate_credentials(
    credentials: Credentials,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<uuid::Uuid, ApiError> {
    let row: Option<_> = sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE email = $1
        "#,
        credentials.email,
    )
    .fetch_optional(&mut conn)
    .await
    .context("Failed to perform a query to retrieve stored credentials.")?;

    let (expected_password_hash, user_id) = match row {
        Some(row) => (row.password_hash, row.user_id),
        None => {
            return Err(ApiError::unprocessable_entity([(
                "email",
                "this email does not exist",
            )]))
        }
    };

    crate::utils::spawn_blocking_with_tracing(move || {
        let expected_password_hash = PasswordHash::new(&expected_password_hash)?;
        Argon2::default().verify_password(
            credentials.password.expose_secret().as_bytes(),
            &expected_password_hash,
        )
    })
    .await
    .context("unexpected error happened during password hashing")?
    .map_err(|_| ApiError::unprocessable_entity([("password", "password is wrong")]))?;
    Ok(user_id)
}

pub fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();
    Ok(Secret::new(password_hash))
}
