use chrono::{ DateTime, Utc };
use sqlx::{PgPool, Row};
use reqwest::Client;
use crate::controllers::embedding::text_to_vec::get_embeddings;



struct User{
    pub id: i32,
    pub telegram_id: String,
    pub cv_text: String,
    pub cv_embeddings: Vec<f32>,
    pub created_at: DateTime<Utc>,
}

pub async fn create_user(pool: &PgPool, telegram_id: &str)->Result<(), sqlx::Error>{

    sqlx::query(
        "INSERT INTO users (telegram_id) VALUES ($1) ON CONFLICT (telegram_id) DO NOTHING"
    )
    .bind(telegram_id)
    .execute(pool)
    .await?;

    Ok(())
}


pub async fn update_user_cv(pool: &PgPool, telegram_id: &str, cv_text: &str, client: &Client) -> Result<(), sqlx::Error>{

        let cv_embedding = match get_embeddings(cv_text, client).await {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error getting CV embedding: {}", e);
                return Err(sqlx::Error::RowNotFound); // Return an error if embedding fails
            }
        };

        sqlx::query(
            "UPDATE users SET cv_text = $1, cv_embeddings = $2 WHERE telegram_id = $3"
        )
        .bind(cv_text)
        .bind(&cv_embedding)
        .bind(telegram_id)
        .execute(pool)
        .await?;
    // Placeholder for the actual implementation
    Ok(())
}