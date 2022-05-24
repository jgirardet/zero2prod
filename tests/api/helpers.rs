use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use wiremock::MockServer;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
}

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("FAiled to execute request in test APP")
    }

    pub async fn get_confirmation_links(
        &self,
        email_request: &wiremock::Request,
    ) -> ConfirmationLinks {
        let mut links = ConfirmationLinks::from_request(email_request);
        links.set_port(self.port);
        links
    }

    pub async fn post_newsletter(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", self.address))
            .json(&body)
            .send()
            .await
            .expect("request failed")
    }
}

#[derive(Debug)]
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl ConfirmationLinks {
    pub fn from_request(request: &wiremock::Request) -> Self {
        let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
        let get_links = |s: &str| {
            let links = linkify::LinkFinder::new()
                .links(s)
                .filter(|x| *x.kind() == linkify::LinkKind::Url)
                .collect::<Vec<_>>();
            assert_eq!(links.len(), 1);
            links[0].as_str().to_owned()
        };
        let html = reqwest::Url::parse(&get_links(&body["HtmlBody"].as_str().unwrap())).unwrap();
        let plain_text =
            reqwest::Url::parse(&get_links(&body["TextBody"].as_str().unwrap())).unwrap();

        Self { html, plain_text }
    }
    pub fn set_port(&mut self, port: u16) {
        self.html.set_port(Some(port)).expect("Failed to set port");
        self.plain_text
            .set_port(Some(port))
            .expect("Failed to set port");
    }
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

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;
    let configuration = {
        let mut c = get_configuration().expect("test init faild in get config");
        c.database.database_name = uuid::Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.timeout_ms = 50;
        c.email_client.base_url = email_server.uri();
        c
    };

    //create and midgrate database
    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("DAuild to build Application");
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address: format!("http://localhost:{}", application_port),
        db_pool: get_connection_pool(&configuration.database),
        email_server,
        port: application_port,
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
