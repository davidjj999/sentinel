use serde_json::json;
use crate::error::SentinelError;
use std::time::Duration;
use tracing::warn;
use std::net::{IpAddr, Ipv4Addr};
use rand::Rng;

pub struct GeminiClient {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_MS: u64 = 2000;

impl GeminiClient {
    pub fn new(api_key: String, model: String) -> Result<Self, SentinelError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(90))
            .local_address(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))
            .http1_only()
            .build()?;
        Ok(Self {
            api_key,
            client,
            model,
        })
    }

    pub async fn generate_response(&self, system_prompt: &str, user_input: &str) -> Result<String, SentinelError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model
        );

        // System instructions are a separate field in the request body for Gemini 1.5
        let body = json!({
            "system_instruction": {
                "parts": [{ "text": system_prompt }]
            },
            "contents": [{
                "parts": [{ "text": user_input }]
            }]
        });

        let mut last_error = None;
        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let base_delay = BASE_RETRY_DELAY_MS * 2u64.pow(attempt - 1);
                let jitter = (base_delay / 4) as i64;
                let mut rng = rand::thread_rng();
                let rand_val = rng.gen_range(-(jitter)..=jitter);
                let actual_delay = (base_delay as i64 + rand_val).max(500) as u64;
                let delay = Duration::from_millis(actual_delay);
                warn!("Gemini API retry {}/{} after {:?}", attempt, MAX_RETRIES, delay);
                tokio::time::sleep(delay).await;
            }

            let resp = match self.client.post(&url)
                .header("x-goog-api-key", &self.api_key)
                .json(&body)
                .send()
                .await {
                    Ok(r) => r,
                    Err(e) => {
                        last_error = Some(SentinelError::Reqwest(e));
                        continue;
                    }
                };

            if resp.status().is_server_error() {
                let status = resp.status();
                let error_text = resp.text().await.unwrap_or_default();
                warn!("Gemini API returned {} (attempt {}): {}", status, attempt + 1, error_text);
                last_error = Some(SentinelError::Llm(format!("Gemini API {}: {}", status, error_text)));
                continue;  // Retry on 5xx
            }

            if !resp.status().is_success() {
                let error_text = resp.text().await?;
                return Err(SentinelError::Llm(format!("Gemini API Error: {}", error_text)));
            }

            let json: serde_json::Value = resp.json().await?;
            
            let text = json["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .ok_or_else(|| SentinelError::Llm(format!("Unexpected Gemini response structure: {}", json)))?
                .to_string();

            return Ok(text);
        }

        Err(last_error.unwrap_or_else(|| SentinelError::Llm("Max retries exceeded".to_string())))
    }
}
