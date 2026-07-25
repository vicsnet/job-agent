use serde::{ Serialize, Deserialize };
use chrono::{ DateTime, Utc };
use sqlx::{ PgPool, Row };
use bcrypt::{ hash, verify, DEFAULT_COST };
use jsonwebtoken::{ encode, decode, Header, Validation, EncodingKey };
use validator::Validate;


#[derive(Debug)]
pub struct Login {
    pub id: i32,
    pub user_id: i32,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    pub confirm_password: String,
}
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Not a valid email address"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters long"))]
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
    body.validate().map_err(|e| format!("Validation error: {}", e))?;

    if body.password != body.confirm_password {
        return Err("Passwords do not match".into());
    }

    let email = body.email.trim().to_lowercase();

    let existing_email = sqlx
        ::query_scalar::<_, i64>("SELECT COUNT(*) from login WHERE email = $1")
        .bind(&email)
        .fetch_one(pool).await?;

    if existing_email > 0 {
        return Err("Email already exists".into());
    }
    let password_hash = hash(&body.password, DEFAULT_COST).map_err(|e|
        format!("Error hashing password: {}", e)
    )?;

    let user_id = sqlx
        ::query_scalar::<_, i32>(
            "INSERT INTO users (telegram_id,state, subscription_status, daily_requests) VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(None::<String>)
        .bind("idle")
        .bind("free")
        .bind(0)
        .fetch_one(pool).await?;
    sqlx
        ::query("INSERT INTO login (user_id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&email)
        .bind(&password_hash)
        .execute(pool).await?;

    let token = create_token(user_id, &email).await?;

    Ok(token)
}

pub async fn users_login(
    pool: &PgPool,
    body: LoginRequest
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // normalise the email to lowercase and trim whitespace
    let email = body.email.trim().to_lowercase();

    let result = sqlx
        ::query_as::<_, (i32, String)>("SELECT id, password_hash FROM login WHERE email = $1")
        .bind(&email)
        .fetch_optional(pool).await?;
    
    let (user_id, password_hash) = match result {
        Some(row) => row,
        None => {
            return Err("Invalid email or password".into());
        }
    };

    let valid = verify(&body.password, &password_hash).map_err(|e|
        format!("Error verifying password: {}", e)
    )?;
    if !valid {
        return Err("Invalid email or password".into());
    }

    let token = create_token(user_id, &body.email).await?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use ::dotenvy::dotenv;
    use std::env;

    #[tokio::test]
    async fn test_register() {
        dotenv().ok();
        let pool = PgPool::connect(
            &env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file")
        ).await.unwrap();
        let email = format!("test5_{}@example.com", chrono::Utc::now().timestamp());
        println!("Registering user with email: {}", email);
        let body = RegisterRequest {
            email: email,
            password: "password12".to_string(),
            confirm_password: "password12".to_string(),
        };
        let token = register_user(&pool, body).await.unwrap();

        println!("Token: {}", token);

        assert_eq!(token.split('.').count(), 3, "token should be a JWT");
    }

    #[tokio::test]
    async fn test_login() {
        dotenv().ok();
        let pool = PgPool::connect(
            &env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file")
        ).await.unwrap();

        let email = "test5_1784990249@example.com".to_string();

        let login_token = users_login(&pool, LoginRequest {
            email: email,
            password: "password12".to_string(),
        }).await.unwrap();

        println!("Login Token: {}", login_token);

        assert_eq!(login_token.split('.').count(), 3, "token should be a JWT");
    }
}
