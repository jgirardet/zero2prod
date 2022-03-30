use actix_web::{post, web, HttpResponse, Responder};

#[derive(serde::Deserialize)]
struct FormData {
    name: String,
    email: String,
}

#[post("/subscriptions")]
async fn subscribe(form: web::Form<FormData>) -> impl Responder {
    HttpResponse::Created().finish()
}
