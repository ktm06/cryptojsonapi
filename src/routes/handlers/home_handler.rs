use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use sqlx::PgPool;

// global usage statistics
pub static USAGE_STATS: Lazy<Mutex<HashMap<String, u64>>> = Lazy::new( || Mutex::new(HashMap::new()));

// function to increment usage statistics
pub fn increment(endpoint: &str) {
    let mut stats: std::sync::MutexGuard<'_, HashMap<String, u64>> = USAGE_STATS.lock().unwrap();
    *stats.entry(endpoint.to_string()).or_insert(0) += 1;
}

// function to get current usage statistics
pub fn get_stats() -> HashMap<String, u64> {
    let stats: std::sync::MutexGuard<'_, HashMap<String, u64>> = USAGE_STATS.lock().unwrap();
    stats.clone()
}

// error response struct

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// parameter structs for endpoints
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

#[derive(Deserialize)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
}

// hello handler
#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("hello {}", name)
}


// fetch current price handler
// format: /fetch?coin={coin}
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


// list all coins handler
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

// fetch data from certain date
// format: /fetchwithdate?coin={coin}&date={dd-mm-yyyy}

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

// trending coins

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

// exchange rate handler
// format: /exchange?from={from}&to={to}

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

// usage statistics handler

#[get("/metrics")]
async fn find_stats() -> impl Responder {
    let stats: std::collections::HashMap<String, u64> = get_stats();
    HttpResponse::Ok().json(stats)
}

// register/login handlers
// connected to local postgres database
// missing hash function -> next steps?
// but still works for demo purposes

#[post("/register")]
async fn register(
    pool: web::Data<PgPool>, 
    query: web::Query<RegisterParams>
) -> impl Responder {
    let user = sqlx::query!(
        r#"
        INSERT INTO users (username, email, passphrase) VALUES ($1, $2, $3)
        "#,
        query.username,
        query.email,
        query.password,
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

#[post("/login")]
async fn login(pool: web::Data<PgPool>, query: web::Query<LoginParams>) -> impl Responder {
    let result = sqlx::query!(
        r#"
        SELECT id, username, passphrase FROM users WHERE username = $1
        "#,
        query.username
    ).fetch_optional(pool.get_ref()).await;

    match result {
        Ok(Some(user)) => {
            if query.password == user.passphrase {
                // actix_session::cookie.insert(query.username.clone(), user.id).unwrap();
                HttpResponse::Ok().json(format!("User {} logged in", query.username))

            } else {
                HttpResponse::Unauthorized().json(ErrorResponse {
                    error: "Invalid username or password".to_string(),
                })
            }
        },
        Ok(None) => HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid username or password".to_string(),
        }),
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to fetch user".to_string(),
        }),
    }
}