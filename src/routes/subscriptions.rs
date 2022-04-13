use actix_web::{post, web, HttpResponse, Responder};
use sqlx::{query, PgPool};
use uuid::Uuid;

use crate::domain::NewSubscriber;
#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    pub name: String,
    pub email: String,
}
#[tracing::instrument(name = "Adding new subscriber")]
#[post("/subscriptions")]
async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> impl Responder {
    let new_subscriber: NewSubscriber = match form.0.try_into() {
        Ok(ns) => ns,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match insert_subscriber(&new_subscriber, &pool).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument]
async fn insert_subscriber(new_sub: &NewSubscriber, pool: &PgPool) -> Result<(), sqlx::Error> {
    query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_sub.email.as_ref(),
        new_sub.name.as_ref(),
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
