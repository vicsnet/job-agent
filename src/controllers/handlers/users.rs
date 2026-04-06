use chrono::{ DateTime, Utc };
use sqlx::{PgPool, Row};
use reqwest::Client;
use crate::controllers::embedding::text_to_vec::get_embeddings;


#[derive(Debug)]
pub struct User{
    pub id: i32,
    pub telegram_id: String,
    pub cv_text: String,
    pub cv_embeddings: Vec<f32>,
    pub created_at: DateTime<Utc>,
    pub state: String,
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
            "UPDATE users SET cv_text = $1, cv_embedding = $2 WHERE telegram_id = $3"
        )
        .bind(cv_text)
        .bind(&cv_embedding)
        .bind(telegram_id)
        .execute(pool)
        .await?;
    // Placeholder for the actual implementation
    Ok(())
}


pub async fn get_user_by_telegram_id(pool: &PgPool, telegram_id: &str) -> Result<Option<User>, sqlx::Error>{
    let row = sqlx::query("SELECT id, telegram_id, cv_text, cv_embeddings, created_at FROM users WHERE telegram_id = $1")
        .bind(telegram_id)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        Ok(Some(User{
            id: row.try_get("id")?,
            telegram_id: row.try_get("telegram_id")?,
            cv_text: row.try_get("cv_text")?,
            cv_embeddings: row.try_get("cv_embeddings")?,
            created_at: row.try_get("created_at")?,
            state: row.try_get("state")?,
        }))
    } else {
        Ok(None)
    }
}

pub async fn update_user_state(pool: &PgPool, telegram_id: &str, state: &str) -> Result<(), sqlx::Error>{
    sqlx::query(
        "UPDATE users SET state = $1 WHERE telegram_id = $2"
    )
    .bind(state)
    .bind(telegram_id)
    .execute(pool)
    .await?;
    // Placeholder for the actual implementation
    Ok(())
}