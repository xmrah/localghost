use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    system: &'a str,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Debug, Serialize)]
struct GenerateOptions {
    temperature: f32,
    num_ctx: u32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GenerateResult {
    pub command: String,
    pub explanation: Option<String>,
    pub risk_level: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: Option<String>,
    done: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    name: String,
}

/// Yapılandırılmış JSON çıktı şeması
fn make_format_schema(explain: bool) -> Value {
    if explain {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string" },
                "explanation": { "type": "string" },
                "risk_level": { "type": "string", "enum": ["safe", "caution", "dangerous"] }
            },
            "required": ["command", "explanation", "risk_level"]
        })
    } else {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string" },
                "risk_level": { "type": "string", "enum": ["safe", "caution", "dangerous"] }
            },
            "required": ["command", "risk_level"]
        })
    }
}

pub async fn generate(
    base_url: &str,
    model: &str,
    prompt: &str,
    system: &str,
    explain: bool,
) -> Result<GenerateResult> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let _format_schema = make_format_schema(explain);

    let payload = GenerateRequest {
        model,
        prompt,
        system,
        stream: false,
        options: GenerateOptions {
            temperature: 0.0,
            num_ctx: 2048,
        },
    };

    let url = format!("{}/api/generate", base_url);
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| anyhow!("Ollama'ya bağlanılamadı: {}\n{} adresinde çalışıyor mu?", e, base_url))?;

    if !response.status().is_success() {
        return Err(anyhow!("Ollama HTTP {}", response.status()));
    }

    let ollama_resp: OllamaResponse = response.json().await?;
    let raw = ollama_resp.response
        .ok_or_else(|| anyhow!("Ollama boş yanıt döndürdü"))?;

    // Markdown blocklarını temizle ve JSON'u çıkar
    let json_str = if let (Some(start), Some(end)) = (raw.find('{'), raw.rfind('}')) {
        if start <= end { &raw[start..=end] } else { &raw }
    } else {
        &raw
    };

    // JSON parse et
    let parsed: serde_json::Map<String, Value> = serde_json::from_str(json_str)
        .map_err(|e| anyhow!("Model geçerli bir komut üretemedi (JSON parse hatası).\n\nBunun sebebi şunlar olabilir:\n1. Verdiğiniz rol (--role) veya istek modelin kafasını karıştırmış olabilir.\n2. Kullanılan model çok küçük (örn: 1b-3b) veya JSON çıktısı vermede başarısız.\n\nHata Detayı: {}\n\nHam Yanıt: {}", e, raw))?;

    let command = parsed.get("command")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("Model komut döndürmedi"))?;

    let explanation = parsed.get("explanation")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let risk_level = parsed.get("risk_level")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(GenerateResult { command, explanation, risk_level })
}

/// Streaming ile gerçek zamanlı token akışı (interaktif mod için)
#[allow(dead_code)]
pub async fn generate_streaming<F>(
    base_url: &str,
    model: &str,
    prompt: &str,
    system: &str,
    mut on_token: F,
) -> Result<String>
where
    F: FnMut(&str),
{
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let payload = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "system": system,
        "stream": true,
        "options": { "temperature": 0.0, "num_ctx": 2048 }
    });

    let url = format!("{}/api/generate", base_url);
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| anyhow!("Ollama'ya bağlanılamadı: {}", e))?;

    let mut stream = response.bytes_stream();
    let mut full_response = String::new();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        let text = std::str::from_utf8(&chunk)?;
        for line in text.lines() {
            if let Ok(resp) = serde_json::from_str::<OllamaResponse>(line) {
                if let Some(token) = resp.response {
                    on_token(&token);
                    full_response.push_str(&token);
                }
            }
        }
    }

    Ok(full_response)
}

pub async fn list_models(base_url: &str) -> Result<Vec<String>> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let url = format!("{}/api/tags", base_url);
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Ok(vec![]);
    }

    let tags: TagsResponse = response.json().await?;
    Ok(tags.models.into_iter().map(|m| m.name).collect())
}
