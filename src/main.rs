mod controllers;
mod helpers;
mod bot;

use dotenvy::dotenv;
use sqlx::PgPool;
use reqwest::Client;
use std::env;
// use controllers::handlers::api_calls::job_fetch_scheduler;


#[tokio::main]
async fn main() {
    // println!("Hello, world!");
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    let pool = PgPool::connect(
            &database_url
        ).await.unwrap();

    let client = Client::new();
    // let pool2 = pool.clone();
    // let client2 = client.clone();

    // tokio::spawn(async move {
    //     job_fetch_scheduler(&pool2, &client2).await;
    // });

    bot::my_bot::run_bot(pool, client).await;

}
