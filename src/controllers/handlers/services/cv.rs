use sqlx::{ PgPool };
use crate::controllers::embedding::text_to_vec::get_embeddings;
use reqwest::Client;
use pdf_extract::extract_text_from_mem;

pub async fn update_cv_pdf(
    pool: &PgPool,
    user_id: i32,
    pdf_bytes: &[u8],
    client: &Client
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let cv_text = extract_text_from_mem(pdf_bytes).map_err(|e|
        format!("Could not read PDF: {}", e)
    )?;

    let cv_text = cv_text.trim().to_string();

    if cv_text.len() < 100 {
        return Err("Could not extract text, is this a scanned image".into());
    }

    // EMBEDDINGS

    let cv_embedding = match get_embeddings(&cv_text, client).await {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error getting CV embedding: {}", e);
            return Err("Error getting CV embedding".into());
        }
    };
    sqlx
        ::query("UPDATE users SET cv_text = $1, cv_embedding = $2 WHERE id = $3")
        .bind(&cv_text)
        .bind(&cv_embedding)
        .bind(user_id)
        .execute(pool).await?;

    // Placeholder for the actual implementation
    Ok(cv_text)
}
