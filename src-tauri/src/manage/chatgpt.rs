use super::utils::get_env;
use reqwest::Client;
use serde_json::Value;
use std::result::Result;

pub async fn request(body: Value) -> Result<Value, String> {
    let keys = get_env().await.unwrap();

    // リクエストを送信
    let client = Client::new();
    let res = match client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", keys.openai_token))
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
    use crate::manage::utils::get_content_for_chatgpt;

    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_request() {
        let body = json!({
            "model": "gpt-4o",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant."
                },
                {
                    "role": "user",
                    "content": "What is the meaning of life?"
                }
            ]
        });

        let res = request(body).await;
        match res {
            Ok(value) => {
                let content = match get_content_for_chatgpt(&value) {
                    Ok(content) => content,
                    Err(e) => format!("Failed to get content: {}", e),
                };
                println!("response: {}", content);
            }
            Err(e) => panic!("Failed to request: {}", e),
        }
    }
}
