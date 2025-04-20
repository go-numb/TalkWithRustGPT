use crate::manage::{self, message::Message, utils};

use super::utils::get_env;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Mutex;
use std::{result::Result, sync::Arc};
use tauri::State;

#[tauri::command]
pub async fn chatgpt_request(
    b: u8,
    msg: &str,
    src: &str,
    state: State<'_, Arc<Mutex<manage::message::Shelf>>>,
) -> Result<String, String> {
    let start_time = chrono::Local::now();

    let (high, low) = model();
    let (set_model, _max_tokens) = if b == 1 { (high, 4096) } else { (low, 16384) };

    // add new request message, and get message history
    let messages = {
        let set_src = if src.is_empty() {
            None
        } else {
            Some(src.to_string())
        };
        let mut mut_shelf = state.lock().unwrap();
        mut_shelf.add_to_messages("user".to_string(), msg.to_string(), set_src);

        let guard_shelf = mut_shelf.clone();
        let mut messages = guard_shelf.get_messages();

        let system_prompt = mut_shelf.system_messages.get();
        if let Some(prompt) = system_prompt.last() {
            messages.push(prompt.clone());
        }

        messages
    };

    // request
    let body = json!({
        "model": set_model,
        // "max_tokens": max_tokens, 4o-previewではサポートされていない
        "messages": messages.iter().map(|m| {
            json!({
                "role": m.role,
                "content": to_content(m.clone())
            })
        }).collect::<Vec<_>>(),
    });
    let res = match inner(body).await {
        Ok(res) => res,
        Err(e) => return Err(format!("Request error: {}", e)),
    };

    // get response message
    let text = match manage::utils::get_content_for_chatgpt(&res) {
        Ok(text) => text,
        Err(e) => format!("Error: {}", e),
    };

    // メッセージを履歴に追加
    let history_messages = {
        let mut mut_shelf = state.lock().unwrap();
        mut_shelf.add_to_messages("assistant".to_string(), text.clone(), None);
        let guard_shelf = mut_shelf.clone();
        guard_shelf.get_messages()
    };

    // for (index, message) in history_messages.iter().enumerate() {
    //     println!(
    //         "{} - role: {}, content: {}",
    //         index, message.role, message.content
    //     );
    // }

    // Token数を算出するため
    // 履歴は消費する
    let all_messages_string = history_messages
        .into_iter()
        .map(|message| message.content.to_string())
        .collect::<String>();

    // VoiceIDの指定を読み込み
    manage::utils::say(text.to_string());

    // マークダウン整形
    let markdown_content = manage::utils::convert_markdown_to_html(text.as_str())?;

    // トークン数・実行時間を算出し、整形する
    Ok(manage::utils::create_response(
        markdown_content.as_str(),
        set_model.as_str(),
        all_messages_string.as_str(),
        src,
        start_time,
    ))
}

#[tauri::command]
pub async fn chatgpt_request_to_dell3(size: u8, msg: &str) -> Result<String, String> {
    // request
    let res = match manage::chatgpt::request_to_dell3(size, msg).await {
        Ok(res) => res,
        Err(e) => return Err(format!("Request error: {}", e)),
    };

    // get response message
    let (prompt, url) = match manage::utils::get_content_for_chatgpt_dell3(&res) {
        Ok(text) => text,
        Err(e) => return Err(format!("Error: {}", e).to_string()),
    };

    let text = json!({
        "prompt": prompt,
        "url": url,
    });

    Ok(text.to_string())
}

pub fn model() -> (String, String) {
    let (mut high_model, mut low_model) = utils::model_high_and_low("CHATGPT_MODELS");
    high_model = if high_model.is_empty() {
        String::from("chatgpt-4o")
    } else {
        high_model
    };
    low_model = if low_model.is_empty() {
        String::from("gpt-4o-mini")
    } else {
        low_model
    };

    (high_model, low_model)
}

pub fn to_content(message: Message) -> Value {
    if message.src.is_none() {
        json!([{"type": "text", "text": message.content }])
    } else {
        json!([
            {
                "type": "image_url",
                "image_url": {
                    "url": message.src.unwrap(),
                },
            },
            {"type": "text", "text": message.content},
        ])
    }
}

pub async fn inner(body: Value) -> Result<Value, String> {
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

pub async fn request_to_dell3(size_type: u8, prompt: &str) -> Result<Value, String> {
    let keys = get_env().await.unwrap();

    let size = match size_type {
        1 => "1024x1024",
        2 => "1792x1024",
        3 => "1024x1792",
        _ => "1024x1024",
    };

    let body = json!({
      "model": "dall-e-3",
      "prompt": prompt,
      "n": 1,
      "size": size,
    });

    // リクエストを送信
    let client = Client::new();
    let res = match client
        .post("https://api.openai.com/v1/images/generations")
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

        let res = inner(body).await;
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

    #[tokio::test]
    async fn test_request_to_dell3() {
        let res = request_to_dell3(1, "a cute cat").await;
        match res {
            Ok(value) => {
                println!("response: {}", value);
            }
            Err(e) => panic!("Failed to request: {}", e),
        }
    }
}
