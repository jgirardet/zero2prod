use std::net::TcpListener;
use sqlx::{PgPool};
use zero2prod::{configuration::get_configuration, startup::run};


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Connection to database");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .expect("Le port n'est pas libre");
    run(listener, connection_pool)?.await
}
