use serde::{ Serialize, Deserialize };
use chrono::{ DateTime, Utc };
use sqlx::{ PgPool, Row };
use bcrypt::{ hash, verify, DEFAULT_COST };
use jsonwebtoken::{ encode, decode, Header, Validation, EncodingKey };

#[derive(Debug)]
pub struct Login {
    pub id: i32,
    pub user_id: i32,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub email: String,
    pub exp: usize,
}

async fn create_token(
    user_id: i32,
    email: &str
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))?;
    Ok(token)
}
pub async fn register_user(
    pool: &PgPool,
    body: RegisterRequest
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if body.password != body.confirm_password {
        return Err("Passwords do not match".into());
    }

    let existing_email = sqlx
        ::query_scalar::<_, i64>("SELECT COUNT(*) from login WHERE email = $1")
        .bind(&body.email)
        .fetch_one(pool).await?;

    if existing_email > 0 {
        return Err("Email already exists".into());
    }
    let password_hash = hash(&body.password, DEFAULT_COST).map_err(|e|
        format!("Error hashing password: {}", e)
    )?;

    let user_id = sqlx
        ::query_scalar::<_, i32>(
            "INSERT INTO users (telegram_id,state, subscription_status, daily_requests) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(None::<String>)
        .bind("idle")
        .bind("free")
        .bind(0)
        .fetch_one(pool).await?;
    sqlx
        ::query("INSERT INTO login (user_id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&body.email)
        .bind(&password_hash)
        .execute(pool).await?;

    let token = create_token(user_id, &body.email).await?;

    Ok(token)
}
