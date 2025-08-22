use once_cell::sync::Lazy;
use std::env;

pub static ADDRESS: Lazy<String> = Lazy::new(|| set_address());
pub static PORT: Lazy<u16> = Lazy::new(|| set_port());


fn set_address() -> String {
    dotenv::dotenv().ok();
    env::var("ADDRESS")
        .expect("ADDRESS must be set in the environment")
        .parse()
        .expect("ADDRESS must be a valid String")
}

fn set_port() -> u16 {
    dotenv::dotenv().ok();
    env::var("PORT")
        .expect("PORT must be set in the environment")
        .parse()
        .expect("PORT must be a valid u16")
}