use actix_web::web;
use super::handlers;


pub fn config(config: &mut web::ServiceConfig) {
    config.service(handlers::home_handler::greet);
    config.service(handlers::home_handler::fetch_price);
    config.service(handlers::home_handler::list);
    config.service(handlers::home_handler::historical);
    config.service(handlers::home_handler::exchange);
    config.service(handlers::home_handler::find_stats);
    config.service(handlers::home_handler::register);
}