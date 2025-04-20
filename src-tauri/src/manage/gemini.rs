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
pub async fn gemini_request(
    b: u8,
    msg: &str,
    src: &str,
    state: State<'_, Arc<Mutex<manage::message::Shelf>>>,
) -> Result<String, String> {
    let start_time = chrono::Local::now();

    let (high, low) = model();
    let (set_model, _max_tokens) = if b == 1 {
        (high.as_str(), 8192)
    } else {
        (low.as_str(), 8192)
    };

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
            "contents": messages.iter().map(|m| {
                json!({
                    // roleがuserの場合はuser、それ以外はmodel as assistant
                    "role": if m.role == "user" { "user" } else { "model" },
                    "parts": to_content(m.clone()),
                })
            }).collect::<Vec<_>>(),
        })
    } else {
        // println!("systemInstruction: {}", system_prompt);
        json!({
            "systemInstruction": {
                "parts": [
                    {
                        "text": system_prompt,
                    }
                ],
            },
            "contents": messages.iter().map(|m| {
                json!({
                    "role": if m.role == "user" { "user" } else { "model" },
                    "parts": to_content(m.clone()),
                })
            }).collect::<Vec<_>>(),
        })
    };
    let res = match inner(set_model, body).await {
        Ok(res) => res,
        Err(e) => return Err(format!("Request error: {}", e)),
    };

    // get response message
    let text = match manage::utils::get_content_for_gemini(&res) {
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
        set_model,
        all_messages_string.as_str(),
        src,
        start_time,
    ))
}

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
pub async fn inner(model: &str, body: Value) -> Result<Value, String> {
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

        let res = inner(use_model, body).await;
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
