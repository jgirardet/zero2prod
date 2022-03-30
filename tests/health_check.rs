use sqlx::{query, Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;

fn spawn_app() -> String {
    let listener =
        TcpListener::bind("127.0.0.1:0").expect("impossible de trouver un port au client de test");
    let port = listener.local_addr().unwrap().port();
    let address = listener.local_addr().unwrap().ip();
    let server = zero2prod::run(listener).expect("run le sspwn faked");
    let _ = tokio::spawn(server);
    format!("http://{}:{}", address, port)
}

#[test]
fn test_config() {
    let conf = get_configuration()
        .expect("config fail")
        .database
        .connection_string();
    assert_eq!(
        conf,
        "postgres://postgres:password@127.0.0.1:5432/newsletter".to_string()
    );
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let url = format!("{}/health_check", address);
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

#[tokio::test]
async fn subscribe_returns_a_201_for_valid_form_data() {
    let app_address = spawn_app();
    let configuration =
        zero2prod::configuration::get_configuration().expect("Erreur pour obtenir la config");
    let connect_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connect_string)
        .await
        .expect("failed to connect to posgres");
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=le%40mail.fr";
    let response = client
        .post(format!("{}/subscriptions", app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("La requête a échoué");
    assert_eq!(201, response.status().as_u16());

    let res = query!("SELECT email, name from subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Fail retrive in db");
    assert_eq!(res.email, "ma@fz.fr")
}

#[tokio::test]
async fn subscribe_returns_a_400_for_data_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=le%40mail.fr", "missing the name"),
        ("", "missing email and name"),
    ];
    for (body, erreur) in cases {
        let response = client
            .post(format!("{}/subscriptions", app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("La requête a échoué");
        assert_eq!(400, response.status().as_u16());
        // assert_eq!(response.text().await.unwrap(), erreur);
    }
}
