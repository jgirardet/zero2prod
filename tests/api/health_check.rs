use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let url = format!("{}/health_check", app.address);
    let client = reqwest::Client::new();
    println!("{}", &url);
    let response = client
        .get(&url)
        .send()
        .await
        .expect("Erreur l'appel client");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}
