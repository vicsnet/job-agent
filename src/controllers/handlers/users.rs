use chrono::{ DateTime, Utc };
use sqlx::{PgPool, Row};
use reqwest::Client;
use crate::controllers::embedding::text_to_vec::get_embeddings;


#[derive(Debug)]
pub struct User{
    pub id: i32,
    pub telegram_id: String,
    pub cv_text: Option<String>,
    pub cv_embedding: Option<Vec<f32>>,
    pub created_at: DateTime<Utc>,
    pub state: String,
    pub subscription_status: String,
    pub subscription_expires: Option<DateTime<Utc>>,
    pub daily_requests: i32,
    pub last_request_date: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct PlanLimits{
    daily_Limit:Option<i32>,
}
pub async fn create_user(pool: &PgPool, telegram_id: &str)->Result<(), sqlx::Error>{

    sqlx::query(
        "INSERT INTO users (telegram_id, state, subscription_status, daily_requests) VALUES ($1, $2, $3, $4) ON CONFLICT (telegram_id) DO NOTHING"
    )
    .bind(telegram_id)
    .bind("idle")
    .bind("free")
    .bind(0)
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
    let row = sqlx::query("SELECT id, telegram_id, cv_text, cv_embedding, created_at, state, subscription_status, subscription_expires, daily_requests, last_request_date FROM users WHERE telegram_id = $1")
        .bind(telegram_id)
        .fetch_optional(pool)
        .await?;
    
    if let Some(row) = row {
        let cv_text: Option<String> = row.try_get("cv_text")?;
    
    let embedding: Option<Vec<f64>> = row.try_get("cv_embedding")?;
        Ok(Some(User{
            id: row.try_get("id")?,
            telegram_id: row.try_get("telegram_id")?,
            cv_text: cv_text,
            cv_embedding: embedding.map(|vec| vec.into_iter().map(|x| x as f32).collect()),
            created_at: row.try_get("created_at")?,
            state: row.try_get("state")?,
            subscription_status: row.try_get("subscription_status")?,
            subscription_expires: row.try_get("subscription_expires")?,
            daily_requests: row.try_get("daily_requests")?,
            last_request_date: row.try_get("last_request_date")?,

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

// sasving jobs id

pub async fn save_user_job(pool: &PgPool, telegram_id: &str, job_id: &str) -> Result<(), sqlx::Error>{
    sqlx::query(
        "INSERT INTO user_sent_jobs (telegram_id, job_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    )
    .bind(telegram_id)
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_already_sent_jobs(pool: &PgPool, telegram_id: &str, job_id:&str) -> Result<bool, sqlx::Error>{

    let result = sqlx::query(
        "SELECT 1 FROM user_sent_jobs WHERE telegram_id = $1 AND job_id = $2"
    )
    .bind(telegram_id)
    .bind(job_id)
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

// update requests count and date

pub async fn update_user_request_count(pool: &PgPool, telegram_id: &str) -> Result<(), sqlx::Error>{
    sqlx::query(
        "UPDATE users SET daily_requests = daily_requests + 1, last_request_date = NOW() WHERE telegram_id = $1"
    )
    .bind(telegram_id)
    .execute(pool)
    .await?;
    Ok(())
}

// reset daily requests daily usage
pub async fn reset_daily_requests(pool: &PgPool, telegram_id: &str) -> Result<(), sqlx::Error>{

    let today = Utc::now().date_naive();    

    sqlx::query(
        "UPDATE users SET daily_requests = 0, last_request_date = $1 WHERE telegram_id = $2"
    )
    .bind(today)
    .bind(telegram_id)
    .execute(pool)
    .await?;
    Ok(())
}

// get users plan Limit
//  None is unlimited
pub async fn get_plan_limits(plan: &str) -> PlanLimits{
    match plan{
        "free"=> PlanLimits{daily_Limit: Some(5)},
        "basic"=> PlanLimits{daily_Limit: Some(15)},
        "premium"=> PlanLimits{daily_Limit: None}, // unlimited
        _ => PlanLimits{daily_Limit: Some(5)}, // default to free plan limits

    }
}