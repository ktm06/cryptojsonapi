use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;
use actix_session::{SessionMiddleware, storage::RedisSessionStore};


pub mod utils;
pub mod routes;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    // initialize logger
    env_logger::init();
    let address: String = utils::constants::ADDRESS.clone();
    let port: u16 = utils::constants::PORT.clone();
    // load .env variables
    dotenv::dotenv().ok();
    println!("Server running at http://{}:{}", address, port);
    // setup redis session store for login/registration management
    let redis_store = RedisSessionStore::new("redis://127.0.0.1:6379").await.unwrap();
    
    // start http server
    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                actix_web::cookie::Key::generate(),
            ))
            .wrap(Logger::default())
            .configure(routes::home_routes::config)
    })
    .bind((address, port))?
    .run()
    .await
}
