mod controllers;
mod helpers;
mod bot;

use dotenvy::dotenv;
use sqlx::PgPool;
use reqwest::Client;
use std::env;


#[tokio::main]
async fn main() {
    // println!("Hello, world!");
    dotenv().ok();

    let pool = PgPool::connect(
            "postgres://job_user:strongpassword@localhost/job_agent"
        ).await.unwrap();

    let client = Client::new();

    bot::my_bot::run_bot(pool, client).await;

}
