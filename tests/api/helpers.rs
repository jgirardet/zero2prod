use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestApp {
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

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("test init faild in get config");
        c.database.database_name = uuid::Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.timeout_ms = 50;
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
