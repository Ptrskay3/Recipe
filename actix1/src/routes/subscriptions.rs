use actix_web::{web, HttpResponse};
use chrono::Utc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub(crate) struct FormData {
    pub email: String,
    pub name: String,
}

pub(crate) async fn subscribe(
    form: web::Form<FormData>,
    conn: web::Data<sqlx::PgPool>,
) -> HttpResponse {
    match sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(conn.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => {
            println!("Failed to execute query {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
