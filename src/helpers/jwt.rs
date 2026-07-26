use actix_web::HttpRequest;
use crate::controllers::login::register::Claims;
use jsonwebtoken::{ decode, DecodingKey, Validation };

pub fn user_id_from_request(req: &HttpRequest) -> Option<i32> {

    let header = req.headers().get("Authorization")?;
    let token = header.to_str().ok()?.strip_prefix("Bearer ")?;
    let claims = verify_token(token).ok()?;

    Some(claims.sub)
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}



