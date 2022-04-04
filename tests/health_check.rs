use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{query, Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::sink);
        init_subscriber(subscriber);
    };
});

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener =
        TcpListener::bind("127.0.0.1:0").expect("impossible de trouver un port au client de test");
    let port = listener.local_addr().unwrap().port();
    let address = listener.local_addr().unwrap().ip();
    let mut db_settings = get_configuration().expect("get config").database;
    let db_name = uuid::Uuid::new_v4();
    db_settings.database_name = db_name.to_string();
    let pg_pool = configure_database(&db_settings).await;
    let server = zero2prod::startup::run(listener, pg_pool.clone()).expect("run le sspwn faked");
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://{}:{}", address, port),
        db_pool: pg_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    PgConnection::connect_with(&config.without_db())
        .await
        .expect("Echec de connection à Postgre")
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Echec de création de la bdd de test");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Echec de création du pool");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Echec des migrations.");

    connection_pool
}

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

#[tokio::test]
async fn subscribe_returns_a_201_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=le%40mail.fr";
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("La requête a échoué");
    assert_eq!(201, response.status().as_u16());

    let res = query!("SELECT email, name from subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Fail retrive in db");
    assert_eq!(res.email, "le@mail.fr");
    assert_eq!(res.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_for_data_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=le%40mail.fr", "missing the name"),
        ("", "missing email and name"),
    ];
    for (body, erreur) in cases {
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect(&format!("La requête a échoué  : {}", erreur));
        assert_eq!(400, response.status().as_u16());
        // assert_eq!(response.text().await.unwrap(), erreur);
    }
}
