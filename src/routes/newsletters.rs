use actix_web::{post, HttpResponse, web::Json};

#[derive(Debug, serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Debug, serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[post("/newsletters")]
pub async fn publish_newsletter(_body: Json<BodyData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
