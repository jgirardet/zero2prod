use actix_web::{post, web, HttpResponse, Responder};
use sqlx::{query, PgPool};
use tracing::Instrument;
use uuid::Uuid;
#[derive(serde::Deserialize)]
struct FormData {
    name: String,
    email: String,
}

#[post("/subscriptions")]
async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> impl Responder {
    let request_id = uuid::Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding new subseriber",
        %request_id,
        %form.name,
        %form.email
    );
    let _request_span_gard = request_span.enter();

    let query_span = tracing::info_span!("ENregistrment en ddb en cours");
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
    .execute(pool.as_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("nouveau subscriber enregistré");
            HttpResponse::Created().finish()
        }
        Err(e) => {
            tracing::error!("fail to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
