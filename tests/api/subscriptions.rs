use sqlx::query;

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_201_for_valid_form_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=le%40mail.fr";

    let response = app.post_subscription(body.to_string()).await;
    assert_eq!(201, response.status().as_u16());

    let res = query!("SELECT email, name from subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Fail retrive in db");
    assert_eq!(res.email, "le@mail.fr");
    assert_eq!(res.name, "le guin");
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
