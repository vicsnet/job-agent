use dotenvy::dotenv;

use job_agent::controllers::handlers::api_calls::job_fetch_scheduler;

use reqwest::Client;
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    let pool = PgPool::connect(&database_url).await.unwrap();
    let client = Client::new();

    job_fetch_scheduler(&pool, &client).await;
}
