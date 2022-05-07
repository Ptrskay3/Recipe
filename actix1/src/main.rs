use std::net::TcpListener;

use actix1::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").expect("failed to bind port");
    run(listener)?.await
}
