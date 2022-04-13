use crate::helpers::spawn_app;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};
#[tokio::test]
async fn confirmations_without_token_are_rekected_with_400() {
    let app = spawn_app().await;
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscriber_returns_a_200_if_called() {
    let app = spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscription("name=bla&email=bla@email.com".into())
        .await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let email_body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links = linkify::LinkFinder::new()
            .links(s)
            .filter(|x| *x.kind() == linkify::LinkKind::Url)
            .collect::<Vec<_>>();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_confirmation_link = get_link(&email_body["HtmlBody"].as_str().unwrap());

    let mut confirmation_link = reqwest::Url::parse(&html_confirmation_link).unwrap();

    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    confirmation_link.set_port(Some(app.port)).unwrap();

    let res2 = reqwest::get(confirmation_link.as_str()).await.unwrap();

    assert_eq!(res2.status().as_u16(), 200);
}
