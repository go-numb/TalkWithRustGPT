use crate::manage::{message::Message, utils};

use super::utils::get_env;
use reqwest::Client;
use serde_json::{json, Value};
use std::result::Result;

pub fn model() -> (String, String) {
    let (mut high_model, mut low_model) = utils::model_high_and_low("GEMINI_MODELS");
    high_model = if high_model.is_empty() {
        String::from("gemini-2.0-flash")
    } else {
        high_model
    };
    low_model = if low_model.is_empty() {
        String::from("gemini-2.0-flash")
    } else {
        low_model
    };

    (high_model, low_model)
}

pub fn to_content(message: Message) -> Value {
    if message.src.is_none() {
        json!([{ "text": message.content }])
    } else {
        let src = message.src.unwrap();
        let media_type = if src.contains("data:image/png") {
            "image/png"
        } else {
            "image/jpeg"
        };
        // remove "data:image/png;base64,"
        let src = src.replace("data:image/png;base64,", "");
        let src = src.replace("data:image/jpeg;base64,", "");

        json!([{
            "text": message.content,
        },
        {
            "inline_data": {
                "mime_type":media_type,
                "data": src,
            }
        }])
    }
}
pub async fn request(model: &str, body: Value) -> Result<Value, String> {
    let keys = get_env().await.unwrap();
    // println!("keys: {:?}", keys);

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, keys.google_key
    );

    // リクエストを送信
    let client = Client::new();
    let res = match client
        .post(url)
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
    use std::path::Path;

    use crate::manage::utils::get_content_for_gemini;

    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_request() {
        dotenv::dotenv().ok();

        let use_model = "gemini-1.5-flash";

        let filepath = Path::new(r"D:\Download\evangelion_tv4_title.png");
        let data_type =
            utils::get_file_type_by_extension(filepath.file_name().unwrap().to_str().unwrap())
                .unwrap();
        let file_data = match std::fs::read(filepath) {
            Ok(data) => data,
            Err(e) => panic!("Failed to read file: {}", e),
        };

        let data = base64::encode(file_data);
        // 最初の20文字だけ表示
        println!("data: {}, {:?}", data_type, &data[..20]);

        let body = json!({
            "contents": [{

                "parts": [
                    {
                        "text": "今日の東京の天気を教えて下さい。台風などの特殊状況もあれば合わせて教えて下さい。",
                    },
                    {
                        "inline_data": {
                            "mime_type":data_type,
                            "data": data,
                        }
                    }
                ],

            }]
        });

        let res = request(use_model, body).await;
        match res {
            Ok(value) => {
                let text = match get_content_for_gemini(&value) {
                    Ok(text) => text,
                    Err(e) => format!("Failed to get content: {}", e),
                };
                println!("response: {:?}", text);
                // to temp file
                let path = Path::new(r"D:\Download\response.txt");
                match std::fs::write(path, value.to_string()) {
                    Ok(_) => println!("Wrote to file: {:?}", path),
                    Err(e) => panic!("Failed to write file: {}", e),
                }
            }
            Err(e) => panic!("Failed to request: {}", e),
        };
    }
}
