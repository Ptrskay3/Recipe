use std::net::TcpListener;

use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};

pub fn run(listener: TcpListener, conn: sqlx::PgPool) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(conn);
    let server = HttpServer::new(move || {
        App::new()
            .route("/subscriptions", web::post().to(subscribe))
            .route("/health_check", web::get().to(health_check))
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
