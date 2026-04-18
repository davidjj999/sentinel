use serde_json::json;
use crate::error::SentinelError;
use std::time::Duration;

pub struct GeminiClient {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl GeminiClient {
    pub fn new(api_key: String, model: String) -> Result<Self, SentinelError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
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

        let resp = self.client.post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(SentinelError::Llm(format!("Gemini API Error: {}", error_text)));
        }

        let json: serde_json::Value = resp.json().await?;
        
        let text = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| SentinelError::Llm(format!("Unexpected Gemini response structure: {}", json)))?
            .to_string();

        Ok(text)
    }
}
