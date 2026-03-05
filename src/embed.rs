/// Embedding provider abstraction.
///
/// The vector space is model-specific: you must use the same provider and
/// model for both indexing and querying. Switching models requires a full
/// rebuild. The provider is stored in the index metadata so engram can
/// detect mismatches and prompt you to rebuild.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

pub const DIMS_OPENAI_SMALL: usize = 1536;
pub const DIMS_NOMIC: usize = 768;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Provider {
    /// OpenAI text-embedding-3-small via API (requires OPENAI_API_KEY or OPENROUTER_API_KEY)
    OpenAiSmall,
    /// nomic-embed-text via local Ollama (no API key, privacy-preserving)
    OllamaNomic { base_url: String },
}

impl Provider {
    pub fn dims(&self) -> usize {
        match self {
            Provider::OpenAiSmall => DIMS_OPENAI_SMALL,
            Provider::OllamaNomic { .. } => DIMS_NOMIC,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Provider::OpenAiSmall => "openai/text-embedding-3-small",
            Provider::OllamaNomic { .. } => "ollama/nomic-embed-text",
        }
    }
}

pub fn embed(text: &str, provider: &Provider) -> Result<Vec<f32>> {
    match provider {
        Provider::OpenAiSmall => embed_openai(text),
        Provider::OllamaNomic { base_url } => embed_ollama(text, base_url),
    }
}

fn embed_openai(text: &str) -> Result<Vec<f32>> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("OPENROUTER_API_KEY"))
        .context("Set OPENAI_API_KEY or OPENROUTER_API_KEY to use OpenAI embeddings")?;

    let base_url = if std::env::var("OPENROUTER_API_KEY").is_ok() {
        "https://openrouter.ai/api/v1"
    } else {
        "https://api.openai.com/v1"
    };

    let resp: serde_json::Value = ureq::post(&format!("{base_url}/embeddings"))
        .set("Authorization", &format!("Bearer {api_key}"))
        .send_json(serde_json::json!({
            "model": "text-embedding-3-small",
            "input": text,
        }))?
        .into_json()?;

    parse_embedding(&resp)
}

fn embed_ollama(text: &str, base_url: &str) -> Result<Vec<f32>> {
    let resp: serde_json::Value = ureq::post(&format!("{base_url}/api/embed"))
        .send_json(serde_json::json!({
            "model": "nomic-embed-text",
            "input": text,
        }))?
        .into_json()?;

    // Ollama embed API returns { "embeddings": [[...]] }
    resp["embeddings"][0]
        .as_array()
        .context("No embeddings in Ollama response")?
        .iter()
        .map(|v| v.as_f64().map(|f| f as f32).context("Non-numeric embedding value"))
        .collect()
}

fn parse_embedding(resp: &serde_json::Value) -> Result<Vec<f32>> {
    resp["data"][0]["embedding"]
        .as_array()
        .context("No embedding in API response")?
        .iter()
        .map(|v| v.as_f64().map(|f| f as f32).context("Non-numeric embedding value"))
        .collect()
}

/// Detect provider from environment. Prefers Ollama (local/private) when available.
pub fn detect_provider() -> Provider {
    // Check if local Ollama is up
    let base_url = std::env::var("OLLAMA_HOST")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());

    if ureq::get(&format!("{base_url}/api/tags"))
        .call()
        .map(|r| r.status() == 200)
        .unwrap_or(false)
    {
        return Provider::OllamaNomic { base_url };
    }

    // Fall back to OpenAI-compatible
    Provider::OpenAiSmall
}
