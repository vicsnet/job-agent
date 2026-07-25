use crate::controllers::login::register::{
    register_user,
    users_login,
    RegisterRequest,
    LoginRequest,
};
use actix_web::{ web, HttpResponse, post, Responder };
use sqlx::PgPool;
use serde_json::json;

// #[post("/register")]
pub async fn register(pool: web::Data<PgPool>, body: web::Json<RegisterRequest>) -> impl Responder {
    match register_user(pool.get_ref(), body.into_inner()).await {
        Ok(token) => HttpResponse::Created().json(json!({ "token": token })),
        Err(e) => HttpResponse::BadRequest().json(json!({ "error": e.to_string() })),
    }
}

// #[post("/login")]
pub async fn login(pool: web::Data<PgPool>, body: web::Json<LoginRequest>) -> impl Responder {
    match users_login(pool.get_ref(), body.into_inner()).await {
        Ok(token) => HttpResponse::Ok().json(json!({ "token": token })),
        Err(e) => HttpResponse::Unauthorized().json(json!({ "error": e.to_string() })),
    }
}
