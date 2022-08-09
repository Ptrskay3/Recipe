use anyhow::Context;
use axum::extract::Query;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{Acquire, PgExecutor, Postgres, Transaction};

use crate::{
    error::ApiError,
    extractors::DatabaseConnection,
    queue::email::{Email, EmailClient},
};

pub async fn _send_confirmation_email(
    email_client: &EmailClient,
    subscriber_email: &str,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), ApiError> {
    let confirmation_link = format!("{}/confirm?token={}", base_url, subscription_token);
    let plain_body = format!(
        "Welcome to Recipes app!\nVisit {} to confirm your registration.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our Recipes app!<br />Click <a href=\"{}\">here</a> to confirm your registration.",
        confirmation_link
    );
    let email = Email::parse(subscriber_email.to_owned())?;
    email_client
        .send_mail(email, "Welcome to Recipes!", &html_body, &plain_body)
        .await
        .map_err(ApiError::Reqwest)
}

pub fn generate_confirmation_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(confirmation_token, tx)
)]
pub async fn store_token(
    tx: &mut Transaction<'_, Postgres>,
    confirmation_token: &str,
    user_id: uuid::Uuid,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
        INSERT INTO confirmation_tokens (confirmation_token, user_id)
        VALUES ($1, $2)
        "#,
        confirmation_token,
        user_id
    )
    .execute(tx)
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn enqueue_delivery_task(
    tx: &mut Transaction<'_, Postgres>,
    confirmation_id: String,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO confirmation_delivery_queue (
            confirmation_id, 
            user_email
        )
        SELECT $1, email
        FROM users
        WHERE confirmed = 'FALSE';
        "#,
        confirmation_id,
    )
    .execute(tx)
    .await?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct Parameters {
    token: String,
}

#[tracing::instrument(name = "Confirm a registration", skip(parameters, conn))]
pub async fn confirm(
    parameters: Query<Parameters>,
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<(), ApiError> {
    let mut tx = conn.begin().await?;
    let user_id = get_user_id_from_token(&mut tx, &parameters.token)
        .await
        .context("Failed to retrieve the user_id associated with the provided token.")?
        .ok_or(ApiError::BadRequest)?;
    confirm_subscriber(&mut tx, user_id)
        .await
        .context("Failed to update the user status to `confirmed`.")?;

    sqlx::query!(
        r#"DELETE FROM confirmation_tokens WHERE user_id = $1"#,
        user_id
    )
    .execute(&mut tx)
    .await
    .context("Failed to delete from confirmation_tokens")?;

    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(user_id, pool))]
pub async fn confirm_subscriber<'c, E>(pool: E, user_id: uuid::Uuid) -> Result<(), ApiError>
where
    E: PgExecutor<'c>,
{
    sqlx::query!(
        r#"UPDATE users SET confirmed = 'TRUE' WHERE user_id = $1"#,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(confirmation_token, pool))]
pub async fn get_user_id_from_token<'c, E>(
    pool: E,
    confirmation_token: &str,
) -> Result<Option<uuid::Uuid>, sqlx::Error>
where
    E: PgExecutor<'c>,
{
    let result = sqlx::query!(
        r#"SELECT user_id FROM confirmation_tokens WHERE confirmation_token = $1"#,
        confirmation_token,
    )
    .fetch_optional(pool)
    .await?;
    Ok(result.map(|r| r.user_id))
}
