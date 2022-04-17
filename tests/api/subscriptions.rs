use crate::helpers::spawn_app;
use sqlx::query;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_returns_a_201_for_valid_form_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=le%40mail.fr";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app.post_subscription(body.to_string()).await;
    assert_eq!(201, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persiste_the_new_subscriber() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=le%40mail.fr";
    // Mock::given(path("/email"))
    //     .and(method("POST"))
    //     .respond_with(ResponseTemplate::new(200))
    //     .mount(&app.email_server)
    //     .await;

    app.post_subscription(body.to_string()).await;

    let res = query!("SELECT email, name, status from subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Fail retrive in db");
    assert_eq!(res.email, "le@mail.fr");
    assert_eq!(res.name, "le guin");
    assert_eq!(res.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_a_400_for_data_missing_or_invalid() {
    let app = crate::helpers::spawn_app().await;
    let cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=le%40mail.fr", "missing the name"),
        ("", "missing email and name"),
        ("name=&email=dazd@dz.gt", "name empty"),
        ("name=le%20guin&email=", "mail empty"),
        ("name=ursulat&email=not-an-email", "name empty"),
    ];
    for (body, erreur) in cases {
        let response = app.post_subscription(body.into()).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "Erreur {} avec {}",
            erreur,
            body
        );
        // assert_eq!(response.text().await.unwrap(), erreur);
    }
}

#[tokio::test]
async fn subscribe_sends_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.to_string()).await;
}

#[tokio::test]
async fn subscribe_sends_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.to_string()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let links = app.get_confirmation_links(&email_request).await;
    assert_eq!(links.html, links.plain_text);
}

#[tokio::test]
async fn subscibre_filas_if_there_is_a_fatal_database_error() {
    let app = spawn_app().await;
    let body = "name=fez&email=fez@fe.gt";
    sqlx::query!("ALTER TABLE subscriptions DROP COLUMN email;",)
        .execute(&app.db_pool)
        .await
        .unwrap();
    let response = app.post_subscription(body.into()).await;
    assert_eq!(response.status().as_u16(), 500);
}
