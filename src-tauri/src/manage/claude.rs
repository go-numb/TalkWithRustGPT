use crate::manage::{self, message::Message, utils};

use super::utils::get_env;
use reqwest::Client;
use serde_json::{json, Value};
use std::{
    result::Result,
    sync::{Arc, Mutex},
};
use tauri::State;

#[tauri::command]
pub async fn claude_request(
    b: u8,
    msg: &str,
    src: &str,
    state: State<'_, Arc<Mutex<manage::message::Shelf>>>,
) -> Result<String, String> {
    let start_time = chrono::Local::now();

    let (high, low) = model();
    let (set_model, max_tokens) = if b == 1 { (high, 8192) } else { (low, 4096) };

    // add new request message, and get message history
    let (messages, system_prompt) = {
        let set_src = if src.is_empty() {
            None
        } else {
            Some(src.to_string())
        };
        let mut mut_shelf = state.lock().unwrap();
        mut_shelf.add_to_messages("user".to_string(), msg.to_string(), set_src);

        let guard_shelf = mut_shelf.clone();

        let system_prompts = mut_shelf.system_messages.get();
        let system_prompt = if !system_prompts.is_empty() {
            // 最期の配列をStringで出力
            let prompt = system_prompts.last().unwrap();
            prompt.content.to_string()
        } else {
            "".to_string()
        };

        (guard_shelf.get_messages(), system_prompt)
    };

    // request
    let body = if system_prompt.is_empty() {
        json!({
            "model": set_model,
            "max_tokens": max_tokens,
            "messages":
                messages.iter().map(|m| {
                    json!({
                        "role": m.role,
                        "content": to_content(m.clone())
                    })
                }).collect::<Vec<_>>(),
        })
    } else {
        json!({
            "model": set_model,
            "max_tokens": max_tokens,
            "system": system_prompt,
            "messages":
            messages.iter().map(|m| {
                json!({
                    "role": m.role,
                    "content": to_content(m.clone())
                })
            }).collect::<Vec<_>>(),
        })
    };
    let res = match inner(body).await {
        Ok(res) => res,
        Err(e) => return Err(format!("Request error: {}", e)),
    };

    // get response message
    let text = match manage::utils::get_content_for_claude(&res) {
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

pub fn model() -> (String, String) {
    let (mut high_model, mut low_model) = utils::model_high_and_low("CLAUDE_MODELS");
    high_model = if high_model.is_empty() {
        String::from("claude-3-7-sonnet-latest")
    } else {
        high_model
    };
    low_model = if low_model.is_empty() {
        String::from("claude-3-5-haiku")
    } else {
        low_model
    };

    (high_model, low_model)
}

pub fn to_content(message: Message) -> Value {
    if message.src.is_none() {
        json!([{"type": "text", "text": message.content}])
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

        json!([
            {
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": media_type,
                    "data": src
                }
            },
            {"type": "text", "text": message.content}
        ])
    }
}

pub async fn inner(body: Value) -> Result<Value, String> {
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

    #[test]
    fn test_default_model_names_have_no_trailing_comma() {
        let (high, low) = super::model();
        assert!(!high.ends_with(','), "high model has trailing comma: {:?}", high);
        assert!(!low.ends_with(','), "low model has trailing comma: {:?}", low);
    }

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

        let res = inner(body).await;
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
