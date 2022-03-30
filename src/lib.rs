pub mod configuration;
pub mod routes;
pub mod startup;

use std::net::TcpListener;

use actix_web::{dev::Server, App, HttpServer};
use routes::{health_check, subscribe};

pub fn run(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(
        || App::new().service(health_check).service(subscribe), // .route("/health_check", web::get().to(health_check))
    )
    .listen(listener)?
    .run();
    Ok(server)
}
