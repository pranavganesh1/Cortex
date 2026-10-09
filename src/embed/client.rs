use serde::{Deserialize, Serialize};
use anyhow::Result;

pub const DEFAULT_OLLAMA_HOST: &str = "http://localhost:11434";
pub const DEFAULT_MODEL: &str = "nomic-embed-text";

/// Returns the base URL for the Ollama instance, respecting `OLLAMA_HOST`.
pub fn ollama_base_url() -> String {
    std::env::var("OLLAMA_HOST").unwrap_or_else(|_| DEFAULT_OLLAMA_HOST.to_string())
}

/// Returns the embedding model name to use, respecting `CORTEX_EMBED_MODEL`.
pub fn embedding_model() -> String {
    std::env::var("CORTEX_EMBED_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string())
}

/// Returns the full embeddings endpoint URL.
pub fn embeddings_endpoint() -> String {
    let base = ollama_base_url();
    let trimmed = base.trim_end_matches('/');
    format!("{}/api/embeddings", trimmed)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EmbedResponse {
    pub embedding: Vec<f32>,
}

/// Request embeddings for the provided text using the local Ollama instance.
pub async fn embed_text(text: &str) -> Result<Vec<f32>> {
    let client = reqwest::Client::new();
    let model = embedding_model();
    let req = EmbedRequest {
        model: model.clone(),
        prompt: text.to_string(),
    };
    
    let res = client.post(embeddings_endpoint())
        .json(&req)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;
    
    if !res.status().is_success() {
        anyhow::bail!("Ollama returned {} — is the model pulled? Run: ollama pull {}", res.status(), model);
    }
    
    let body: EmbedResponse = res.json().await?;
    Ok(body.embedding)
}

/// Check if the local Ollama daemon is reachable.
pub async fn check_ollama() -> bool {
    let base = ollama_base_url();
    reqwest::Client::new()
        .get(base)
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_request_serialization() {
        let req = EmbedRequest {
            model: "nomic-embed-text".to_string(),
            prompt: "fn calculate()".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"nomic-embed-text\""));
        assert!(json.contains("\"prompt\":\"fn calculate()\""));
    }

    #[test]
    fn test_embed_response_deserialization() {
        let json_str = r#"{"embedding": [0.1, -0.2, 0.35]}"#;
        let res: EmbedResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(res.embedding, vec![0.1, -0.2, 0.35]);
    }

    #[test]
    fn test_default_endpoints() {
        assert_eq!(DEFAULT_MODEL, "nomic-embed-text");
        assert!(embeddings_endpoint().ends_with("/api/embeddings"));
    }
}

