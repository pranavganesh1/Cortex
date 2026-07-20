use serde::{Deserialize, Serialize};
use anyhow::Result;

const OLLAMA_URL: &str = "http://localhost:11434/api/embeddings";
const MODEL: &str = "nomic-embed-text";

#[derive(Serialize)]
struct EmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

pub async fn embed_text(text: &str) -> Result<Vec<f32>> {
    let client = reqwest::Client::new();
    let req = EmbedRequest {
        model: MODEL.to_string(),
        prompt: text.to_string(),
    };
    
    let res = client.post(OLLAMA_URL)
        .json(&req)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;
    
    if !res.status().is_success() {
        anyhow::bail!("Ollama returned {} — is the model pulled? Run: ollama pull nomic-embed-text", res.status());
    }
    
    let body: EmbedResponse = res.json().await?;
    Ok(body.embedding)
}

pub async fn check_ollama() -> bool {
    reqwest::Client::new()
        .get("http://localhost:11434")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
