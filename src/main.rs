use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
#[tokio::main]

async fn main() -> std::io::Result<()> {
    // Redirect all `log`'s events to our subscriber
    init_subscriber(get_subscriber(
        "zero2prod".to_string(),
        "info".to_string(),
        std::io::stdout,
    ));

    //configuration + database
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool =
        PgPool::connect(&configuration.database.connection_string().expose_secret())
            .await
            .expect("Connection to database");

    // tcp
    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .expect("Le port n'est pas libre");
    run(listener, connection_pool)?.await
}
