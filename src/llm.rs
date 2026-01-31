use serde_json::json;
use std::error::Error;

pub struct GeminiClient {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            model: "gemini-3-flash-preview".to_string(),
        }
    }

    pub async fn generate_response(&self, system_prompt: &str, user_input: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
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
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(format!("Getmini API Error: {}", error_text).into());
        }

        let json: serde_json::Value = resp.json().await?;
        
        let text = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or("Failed to parse Gemini response text")?
            .to_string();

        Ok(text)
    }
}
