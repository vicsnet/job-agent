use reqwest::Client;
use serde_json::json;
use dotenvy::dotenv;


pub async fn get_embeddings(text: &str, openaiclient: &Client) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    dotenv().ok();

    let res = openaiclient
        .post("https://api.openai.com/v1/embeddings")
        .bearer_auth(std::env::var("OPEN_API_KEY")?)
        .json(&json!({
            "model": "text-embedding-3-small",
            "input": text
        }))
        .send()
        .await?;


    let json: serde_json::Value = res.json().await?;
    let embedding = json["data"][0]["embedding"]
        .as_array()
        .ok_or("Invalid response format")?
        .iter()
        .map(|v| v.as_f64().unwrap_or(0.0) as f32)
        .collect();
    // Placeholder for the actual implementation
    Ok(embedding)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_embeddings(){
        let client = Client::new();

        let text = "Hello, world!";
        let result = get_embeddings(text, &client).await;
        // assert!(result.is_ok());
        dbg!("Embedding vector: {:?}", result.unwrap());

    }



}