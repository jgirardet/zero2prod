use actix_web::{post, web, HttpResponse, Responder};
use sqlx::{query, PgPool};
use uuid::Uuid;
#[derive(serde::Deserialize, Debug)]
struct FormData {
    name: String,
    email: String,
}
#[tracing::instrument(name = "Adding new subscriber")]
#[post("/subscriptions")]
async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> impl Responder {
    match insert_subscriber(&form, &pool).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// #[tracing::instrument(
//     name="Insertion en ddb",
//     skip(form, pool),
// )]
#[tracing::instrument]
async fn insert_subscriber(form: &FormData, pool: &PgPool) -> Result<(), sqlx::Error> {
    query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        chrono::Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
