use actix_web::{web::Query, HttpResponse, Responder};

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    _subscription_token: String,
}

#[tracing::instrument]
#[actix_web::get("/subscriptions/confirm")]
async fn confirm(query: Query<Parameters>) -> impl Responder {
    HttpResponse::Ok().finish()
}
