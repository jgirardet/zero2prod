use actix_web::{post, web, HttpResponse, Responder};
use sqlx::{query, PgPool};
use uuid::Uuid;
#[derive(serde::Deserialize)]
struct FormData {
    name: String,
    email: String,
}

#[post("/subscriptions")]
async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> impl Responder {
    match query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        chrono::Utc::now()
    )
    .execute(connection.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => {
            eprintln!("Faile to execute query : {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
