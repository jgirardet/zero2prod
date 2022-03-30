use std::net::TcpListener;

use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, middleware::Logger, web, App, HttpServer};
use sqlx::PgPool;

pub fn run(listener: TcpListener, connection: PgPool) -> std::io::Result<Server> {
    let db_pool = web::Data::new(connection);
    let server = HttpServer::new(
        move || {
            App::new()
                .wrap(Logger::default())
                .service(health_check)
                .service(subscribe)
                .app_data(db_pool.clone())
        }, // .route("/health_check", web::get().to(health_check))
    )
    .listen(listener)?
    .run();
    Ok(server)
}
