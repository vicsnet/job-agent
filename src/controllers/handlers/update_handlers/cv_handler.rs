use actix_web::{ web, Responder, HttpRequest, HttpResponse };
use reqwest::Client;
use sqlx::{ PgPool };
use actix_multipart::Multipart;
use serde_json::json;

use crate::helpers::jwt::{ user_id_from_request };
use crate::controllers::handlers::services::cv::update_cv_pdf;
use futures_util::StreamExt;

const MAX_SIZE: usize = 5 * 1024 * 1024;
pub async fn upload_cv(
    req: HttpRequest,
    client: web::Data<Client>,
    pool: web::Data<PgPool>,
    mut payload: Multipart
) -> impl Responder {
    let user_id = match user_id_from_request(&req) {
        Some(id) => id,
        None => {
         
            return HttpResponse::Unauthorized().json(json!({"error": "Missing or invalid token"}));
        }
    };

    let mut bytes: Vec<u8> = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => {
                   eprintln!("multipart eror: {:?}", e);
                return HttpResponse::BadRequest().json(json!({"error": "Malformed upload"}));
            }
        };

        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(d) => d,
                Err(_) => {
                    return HttpResponse::BadRequest().json(json!({"error": "Faild reading file"}));
                }
            };

            if bytes.len() + data.len() > MAX_SIZE {
                return HttpResponse::PayloadTooLarge().json(
                    json!({"error":"File must be under 5M"})
                );
            }
            bytes.extend_from_slice(&data);
        }
    }
    if bytes.is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "File is empty"}));
    }
    if !bytes.starts_with(b"%PDF") {
        return HttpResponse::BadRequest().json(json!({"error": "File must be a PDF"}));
    }

    match update_cv_pdf(&pool, user_id, &bytes, &client).await {
        Ok(text) =>
            HttpResponse::Ok().json(
                json!({
            "message": "CV uploaded",
            "characters": text.len(),
        })
            ),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}
