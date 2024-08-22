use dotenv::dotenv;
use serde_json::Value;
use std::env;
use std::error::Error;
use std::result::Result;

use chrono::prelude::*;
use tiktoken_rs::cl100k_base;

use serde_json::json;

use markdown;

use crate::sub;

#[derive(Debug)]
pub struct Keys {
    pub anthropic_key: String,
    pub openai_token: String,
    pub voice_id: i16,
}

/// 必要な環境変数を取得する
pub async fn get_env() -> Result<Keys, Box<dyn Error>> {
    dotenv().ok();
    let anthropic_key = env::var("ANTHROPIC_API_KEY")?;
    let openai_token = env::var("CHATGPTTOKEN")?;
    let voice_id = env::var("VOICEID")?.parse()?;

    Ok(Keys {
        anthropic_key,
        openai_token,
        voice_id,
    })
}

pub fn get_content_for_chatgpt(v: &Value) -> Result<String, String> {
    // get choices[0].message.content to string
    let choices = v["choices"].as_array().ok_or("choices not found")?;

    // check if choices is empty
    if choices.is_empty() {
        return Err("choices is empty".to_string());
    }

    Ok(choices[0]["message"]["content"]
        .as_str()
        .ok_or("content not found or not a string")?
        .to_string())
}

pub fn get_content_for_claude(v: &Value) -> Result<String, String> {
    // Get content[0].text as a string
    // Get the content array
    let content = v["content"].as_array().ok_or("content not found")?;

    // Check if the content array is empty
    if content.is_empty() {
        return Err("content is empty".to_string());
    }

    // Get the text field from the first content item
    let text = content[0]["text"]
        .as_str()
        .ok_or("text field not found or not a string")?;

    Ok(text.to_string())
}

/// AIが出力したマークダウン用テキストをHTML出力する
pub fn convert_markdown_to_html(text: &str) -> Result<String, String> {
    markdown::to_html_with_options(text, &markdown::Options::gfm())
        .map_err(|e| format!("markdown::to_html_with_options error: {}", e))
}

/// invokeへの返り値を作成する
pub fn create_response(
    markdown_content: &str,
    set_model: &str,
    tokenize_resource: &str,
    src: &str,
    start: chrono::DateTime<chrono::Local>,
) -> String {
    let end = chrono::Local::now();
    let bpe = cl100k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens([tokenize_resource, src].concat().as_str());
    let msg = format!(
        "{}\n\nModel: {}, Total token: {}, Elaps: {}s",
        markdown_content,
        set_model,
        tokens.len(),
        end.signed_duration_since(start).num_seconds(),
    );
    msg
}

pub fn create_content_for_chatgpt(msg: &str, src: &str) -> Value {
    if src.is_empty() {
        json!([{"type": "text", "text": msg}])
    } else {
        json!([
            {
                "type": "image_url",
                "image_url": {
                    "url": src,
                },
            },
            {"type": "text", "text": msg}
        ])
    }
}

pub fn create_content_for_claude(msg: &str, src: &str) -> Value {
    if src.is_empty() {
        json!([{"type": "text", "text": msg}])
    } else {
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
            {"type": "text", "text": msg}
        ])
    }
}

pub fn say(msg: String) -> bool {
    // VoiceIDの指定を読み込み
    let mut is_voice: bool;
    let voice_id: i16 = match env::var("VOICEID") {
        Ok(val) => {
            is_voice = true;
            println!("VOICEID: {}", val);
            val.parse().unwrap()
        }
        Err(e) => {
            println!("couldn't interpret VOICEID: {}", e);
            is_voice = false;
            1
        }
    };

    // メッセージを発言
    // 棒読みちゃんが起動していない場合は無視します
    if is_voice {
        match sub::voice::say(voice_id, msg.as_str()) {
            Ok(_) => is_voice = true,
            Err(e) => {
                println!(
                    "棒読みちゃんが起動していないか、エラーが発生しました: {}",
                    e
                );
                is_voice = false;
            }
        };
    }

    is_voice
}

#[cfg(test)]
mod tests {
    use dotenv::from_filename;

    use super::*;

    #[tokio::test]
    async fn test_get_env() {
        let current = env::current_dir().expect("Failed to get current directory");
        let filepath = current.join(".env");
        println!("filepath: {:?}", filepath);
        let _ = dotenv::from_path(filepath.as_path());

        let keys = match get_env().await {
            Ok(keys) => keys,
            Err(e) => panic!("Failed to get env: {}", e),
        };
        assert!(
            !keys.anthropic_key.is_empty(),
            "anthropic_key is empty, {}",
            keys.anthropic_key
        );
        assert!(
            !keys.openai_token.is_empty(),
            "openai_token is empty, {}",
            keys.openai_token
        );
        assert!(keys.voice_id > 0, "voice_id is wrong, {}", keys.voice_id);
    }

    #[test]
    fn test_get_content_for_chatgpt() {
        let v: Value = match serde_json::from_str(
            r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4o-mini",
            "system_fingerprint": "fp_44709d6fcb",
            "choices": [{
              "index": 0,
              "message": {
                "role": "assistant",
                "content": "\n\nHello there, how may I assist you today?"
              },
              "logprobs": null,
              "finish_reason": "stop"
            }],
            "usage": {
              "prompt_tokens": 9,
              "completion_tokens": 12,
              "total_tokens": 21
            }
          }"#,
        ) {
            Ok(v) => v,
            Err(e) => panic!("Failed to parse JSON: {}", e),
        };
        assert!(v.is_object());

        let content = match get_content_for_chatgpt(&v) {
            Ok(content) => content,
            Err(e) => panic!("Failed to get content: {}", e),
        };

        assert_eq!(content, "\n\nHello there, how may I assist you today?");
    }
}
