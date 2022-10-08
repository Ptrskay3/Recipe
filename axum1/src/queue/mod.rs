use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Transaction};
use tracing::{field::display, Span};

use crate::config::{DatabaseSettings, Settings};

use crate::email::{Email, EmailClient};
use crate::error::ApiError;

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);
    let email_client = configuration.email_client.client();
    worker_loop(connection_pool, email_client).await
}

async fn worker_loop(pool: PgPool, email_client: EmailClient) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(&pool, &email_client).await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Ok(ExecutionOutcome::TaskCompleted) => {}
        }
    }
}

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutcome, anyhow::Error> {
    let task = dequeue_task(pool).await?;
    if task.is_none() {
        return Ok(ExecutionOutcome::EmptyQueue);
    }
    let (mut transaction, confirmation_id, email) = task.unwrap();
    Span::current()
        .record("confirmation_id", &display(confirmation_id.clone()))
        .record("user_email", &display(&email));
    match Email::parse(email.clone()) {
        Ok(email) => {
            if let Err(e) = email_client
                .send_mail(
                    email.clone(),
                    "Recipe App confirm registration",
                    &format!(
                        "Visit http://localhost:3001/confirm?token={} to confirm your registration.",
                        confirmation_id
                    ),
                    &format!(
                        "Visit <a href=http://localhost:3001/confirm?token={}>the website</a> to confirm your registration.",
                        confirmation_id
                    ),
                )
                .await
            {
                insert_failed_task(&mut transaction, confirmation_id.clone(), email.as_ref(), &e).await?;
                tracing::error!(
                    error.cause_chain = ?e,
                    error.message = %e,
                    "Failed to deliver email to a confirm registration. \
                        Skipping.",
                );
            }
        }
        Err(e) => {
            insert_failed_task(&mut transaction, confirmation_id.clone(), &email, &e).await?;
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Skipping a sending confirmation email. \
                    Their stored details are invalid",
            );
        }
    }
    delete_task(transaction, confirmation_id, &email).await?;
    Ok(ExecutionOutcome::TaskCompleted)
}

type PgTransaction = Transaction<'static, Postgres>;

#[tracing::instrument(skip_all)]
async fn dequeue_task(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, String, String)>, anyhow::Error> {
    let mut tx = pool.begin().await?;
    let r = sqlx::query!(
        r#"
        SELECT confirmation_id, user_email
        FROM confirmation_delivery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#,
    )
    .fetch_optional(&mut tx)
    .await?;
    if let Some(r) = r {
        Ok(Some((tx, r.confirmation_id, r.user_email)))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(skip_all)]
async fn delete_task(
    mut tx: PgTransaction,
    confirmation_id: String,
    email: &str,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        DELETE FROM confirmation_delivery_queue
        WHERE 
            confirmation_id = $1 AND
            user_email = $2 
        "#,
        confirmation_id,
        email
    )
    .execute(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn insert_failed_task<E>(
    tx: &mut PgTransaction,
    confirmation_id: String,
    email: &str,
    e: &E,
) -> Result<(), anyhow::Error>
where
    E: Into<ApiError> + std::fmt::Display,
{
    sqlx::query!(
        r#"
        INSERT INTO failed_jobs
        (job_id, job_type, context)
        VALUES
        ($1, 'email_delivery', $2)
        ON CONFLICT DO NOTHING
        "#,
        confirmation_id,
        serde_json::json!({ "email": email, "error": e.to_string() })
    )
    .execute(tx)
    .await?;
    Ok(())
}
