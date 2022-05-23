use std::ops::Range;

use crate::helpers::{spawn_app, ConfirmationLinks, TestApp};
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    // Mock::given(any())
    //     .respond_with(ResponseTemplate::new(200))
    //     .expect(0)
    //     .mount(&app.email_server)
    //     .await;
    // app.mock_mail_server("p", 200, 0);
    // mockmail![app "p",- 200];
    let newsletter_request_body = serde_json::json!({
    "title": "Newsletter title",
    "content": {
    "text": "Newsletter body as plain text",
    "html": "<p>Newsletter body as HTML</p>",
    }
    });
    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .json(&newsletter_request_body)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // Mock verifies on Drop that we haven't sent the newsletter email
}

#[tokio::test]
async fn newsletter_are_delivered_ton_confirmed_subscibers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
    "title": "Newsletter title",
    "content": {
    "text": "Newsletter body as plain text",
    "html": "<p>Newsletter body as HTML</p>",
    }
    });

    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .json(&newsletter_request_body)
        .send()
        .await
        .expect("Failed to execut request");

    assert_eq!(response.status().as_u16(), 200);
}

async fn newsletters_return_400_for_invalid_data() {
    let app = spawn_app().await;
    let bodys = vec![
        (serde_json::json!({}), "empty request"),
        (serde_json::json![{"title":"bla"}], "no bdy present"),
        (
            serde_json::json![{"title":"bla", "content":{"html":"bka"}}],
            "no plain text",
        ),
        (
            serde_json::json![{"title":"bla", "content":{"text":"bka"}}],
            "no plain html",
        ),
    ];
    for (j, m) in bodys {
        let resp = app.post_newsletter(j).await;
        assert_eq!(
            resp.status().as_u16(),
            400,
            "Should have returned 400 with {}",
            m
        )
    }
}

#[tokio::test]
async fn newletters_are_not_delevered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
}

/// Use the public API of the application under test to create
/// an unconfirmed subscriber.
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscription(body.into())
        .await
        .error_for_status()
        .unwrap();
    app.get_confirmation_links(
        &app.email_server
            .received_requests()
            .await
            .unwrap()
            .pop()
            .unwrap(),
    )
    .await
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let link = create_unconfirmed_subscriber(app).await.html;
    reqwest::get(link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}
