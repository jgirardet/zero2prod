use std::net::TcpListener;

use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, connection: PgPool) -> std::io::Result<Server> {
    let db_pool = web::Data::new(connection);
    let server = HttpServer::new(
        move || {
            App::new()
                .wrap(TracingLogger::default())
                .service(health_check)
                .service(subscribe)
                .app_data(db_pool.clone())
        }, // .route("/health_check", web::get().to(health_check))
    )
    .listen(listener)?
    .run();
    Ok(server)
}
