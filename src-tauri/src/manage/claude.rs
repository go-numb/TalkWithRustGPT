use super::utils::get_env;
use reqwest::Client;
use serde_json::Value;
use std::result::Result;

pub async fn request(body: Value) -> Result<Value, String> {
    let keys = get_env().await.unwrap();

    // リクエストを送信
    let client = Client::new();
    let res = match client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", keys.anthropic_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(err) => {
            return Err(format!("Request error: {}", err));
        }
    };

    match res.json().await {
        Ok(json) => Ok(json),
        Err(err) => Err(format!("JSON parse error: {}", err)),
    }
}

#[cfg(test)]
mod tests {
    use crate::manage::utils::get_content_for_claude;

    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_request() {
        let body = json!({
            "model": "claude-3-5-sonnet-20240620",
            "max_tokens": 1024,
            "messages": [
            {
                "role": "user",
                "content": "What is the meaning of life?"
            }
        ]});

        let res = request(body).await;
        match res {
            Ok(value) => {
                let text = match get_content_for_claude(&value) {
                    Ok(text) => text,
                    Err(e) => format!("Failed to get content: {}", e),
                };
                println!("response: {}", text);
            }
            Err(e) => panic!("Failed to request: {}", e),
        };
    }
}
