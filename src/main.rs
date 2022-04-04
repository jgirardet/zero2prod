use secrecy::ExposeSecret;
use sqlx::{postgres::PgPoolOptions};
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
    let configuration = get_configuration().expect("Failed to read configuration, désolé");

    dbg!(&configuration);
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(configuration.database.connection_string().expose_secret())
        .expect("FAiled to Connect to database");

    // tcp
    let listener = TcpListener::bind(configuration.application.connection_string())
        .expect("Le port n'est pas libre");
    run(listener, connection_pool)?.await
}
