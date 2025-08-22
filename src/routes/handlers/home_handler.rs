use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use sqlx::PgPool;
use password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand_core::OsRng;

pub static USAGE_STATS: Lazy<Mutex<HashMap<String, u64>>> = Lazy::new( || Mutex::new(HashMap::new()));

pub fn increment(endpoint: &str) {
    let mut stats: std::sync::MutexGuard<'_, HashMap<String, u64>> = USAGE_STATS.lock().unwrap();
    *stats.entry(endpoint.to_string()).or_insert(0) += 1;
}

pub fn get_stats() -> HashMap<String, u64> {
    let stats: std::sync::MutexGuard<'_, HashMap<String, u64>> = USAGE_STATS.lock().unwrap();
    stats.clone()
}


#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
pub struct CoinParams {
    pub coin: String,
}

#[derive(Deserialize)]
pub struct HistoryParams {
    pub coin: String,
    pub date: u32,
}

#[derive(Deserialize)]
pub struct ExchangeParams {
    pub from: String,
    pub to: String,
}

#[derive(Deserialize)]
pub struct RegisterParams {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("hello {}", name)
}



#[get("/fetch")]
async fn fetch_price(query: web::Query<CoinParams>) -> impl Responder {
    increment("fetch");
    let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", query.coin);
    match reqwest::get(&url).await {
        Ok(resp) => {
            match resp.text().await {
                Ok(text) => if text == "{}" {
                    HttpResponse::NotFound().json(ErrorResponse {
                        error: format!("Coin '{}' not found", query.coin),
                    })
                } else {
                    HttpResponse::Ok().content_type("application/json").body(text)
                }   
                Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to parse response".to_string(),
                }),
            }
        },
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to fetch data".to_string(),
        }),
    }
}

#[get("/coins")]
async fn list() -> impl Responder {
    increment("coins");
    let url = "https://api.coingecko.com/api/v3/coins/list";
    match reqwest::get(url).await {
        Ok(resp) => {
            match resp.text().await {
                Ok(text) => HttpResponse::Ok().content_type("application/json").body(text),
                Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to parse response".to_string(),
                }),
            }
        },
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to fetch data".to_string(),
        }),
    }
}

#[get("/fetchwithdate")]
async fn historical(query: web::Query<HistoryParams>) -> impl Responder {
    increment("historical");
    let url = format!("https://api.coingecko.com/api/v3/coins/{}/history?date={}", query.coin, query.date);
    match reqwest::get(&url).await {
        Ok(resp) => {
            match resp.text().await {
                Ok(text) => if text == "{}" {
                    HttpResponse::NotFound().json(ErrorResponse {
                        error: format!("No historical data found for coin '{}' on date '{}'", query.coin, query.date),
                    })
                } else {
                    HttpResponse::Ok().content_type("application/json").body(text)
                }
                Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to parse response".to_string(),
                }),
            }
        },
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to fetch data".to_string(),
        }),
    }
}

#[get("/trending")]
async fn trending() -> impl Responder {
    increment("trending");
    let url = "https://api.coingecko.com/api/v3/search/trending";
    match reqwest::get(url).await {
        Ok(resp) => {
            match resp.text().await {
                Ok(text) => HttpResponse::Ok().content_type("application/json").body(text),
                Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to parse response".to_string(),
                }),
            }
        },
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to fetch data".to_string(),
        }),
    }
}

#[get("/exchange")]
async fn exchange(query: web::Query<ExchangeParams>) -> impl Responder{
    increment("exchange");
    let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}", query.from, query.to);
    match reqwest::get(&url).await {
        Ok(resp) => {
            match resp.text().await {
                Ok(text) => if text == "{}" {
                    HttpResponse::NotFound().json(ErrorResponse {
                        error: format!("Exchange rate from '{}' to '{}' not found", query.from, query.to),
                    })
                } else {
                    HttpResponse::Ok().content_type("application/json").body(text)
                }
                Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to parse response".to_string(),
                }),
            }
        },
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to fetch data".to_string(),
        }),
    }
}

#[get("/metrics")]
async fn find_stats() -> impl Responder {
    let stats: std::collections::HashMap<String, u64> = get_stats();
    HttpResponse::Ok().json(stats)
}
//test commit
#[post("/register")]
async fn register(
    pool: web::Data<PgPool>, 
    query:web::Query<RegisterParams>
) -> impl Responder {
    let salt: SaltString = SaltString::generate(&mut OsRng);
    let argon2: Argon2<'_> = Argon2::default();
    let password_hash: String = match argon2.hash_password(query.password.as_bytes(), &salt)
    {
        Ok(hash) => hash.to_string(),
        Err(_) => return HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to hash password".to_string(),
        }),
    };

    let user = sqlx::query!(
        r#"
        INSERT INTO users (username, password_hash, salt, email) VALUES ($1, $2, $3, $4)
        "#,
        query.username,
        query.email,
        password_hash.as_str(),
        salt.as_str(),
    ).execute(pool.get_ref()).await;

    match user {
        Ok(_) => HttpResponse::Ok().json(format!("User {}, registered", query.username)),
        Err(e) => {
            eprintln!("Failed to register user: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to register user".to_string(),
            })

        }
    }
}

