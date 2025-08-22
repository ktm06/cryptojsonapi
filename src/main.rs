use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;

pub mod utils;
pub mod routes;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let address: String = utils::constants::ADDRESS.clone();
    let port: u16 = utils::constants::PORT.clone();
    dotenv::dotenv().ok();
    println!("Server running at http://{}:{}", address, port);
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .configure(routes::home_routes::config)
    })
    .bind((address, port))?
    .run()
    .await
}
