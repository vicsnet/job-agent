mod controllers;
mod helpers;
mod bot;

use dotenvy::dotenv;
use sqlx::PgPool;
use reqwest::Client;
use std::env;
use actix_web::{ web, App, HttpServer };
// use controllers::handlers::api_calls::job_fetch_scheduler;
use controllers::handlers::auth_handlers::auth::{ login, register };

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    let pool = PgPool::connect(&database_url).await.unwrap();

    let client = Client::new();
    // let pool2 = pool.clone();
    // let client2 = client.clone();

    // tokio::spawn(async move {
    //     job_fetch_scheduler(&pool2, &client2).await;
    // });
    let bot_pool = pool.clone();
    let bot_client = client.clone();
    tokio::spawn(async move {
        bot::my_bot::run_bot(bot_pool, bot_client).await;
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web
                    ::scope("/auth")
                    .route("/register", web::post().to(register))
                    .route("/login", web::post().to(login))
            )
    })
        .bind(("127.0.0.1", 8080))?
        .run().await
}
