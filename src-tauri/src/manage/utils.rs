use dotenv::dotenv;
use serde_json::Value;
use std::env;
use std::error::Error;
use std::path::Path;
use std::result::Result;

use tiktoken_rs::cl100k_base;

use markdown;

use crate::sub;

pub fn model_high_and_low(key: &str) -> (String, String) {
    match std::env::var(key) {
        Ok(val) => {
            let models: Vec<&str> = val.split(',').collect();
            if models.len() == 2 {
                (models[0].to_string(), models[1].to_string())
            } else {
                ("".to_string(), "".to_string())
            }
        }
        Err(_) => ("".to_string(), "".to_string()),
    }
}

#[derive(Debug)]
pub struct Keys {
    pub anthropic_key: String,
    pub openai_token: String,
    pub google_key: String,
    pub voice_id: i16,
}

/// 必要な環境変数を取得する
pub async fn get_env() -> Result<Keys, Box<dyn Error>> {
    dotenv().ok();
    let anthropic_key = env::var("ANTHROPIC_API_KEY")?;
    let openai_token = env::var("CHATGPTTOKEN")?;
    let google_key = env::var("GOOGLE_GEMINI_API_KEY")?;
    let voice_id = env::var("VOICEID")?.parse()?;

    Ok(Keys {
        anthropic_key,
        openai_token,
        google_key,
        voice_id,
    })
}

pub fn get_file_type_by_extension(file_path: &str) -> Option<&str> {
    let path = Path::new(file_path);
    match path.extension()?.to_str()? {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        "mp4" => Some("video/mp4"),
        "txt" => Some("text/plain"),
        "json" => Some("application/json"),
        "csv" => Some("text/csv"),
        "pdf" => Some("application/pdf"),
        // 他の拡張子の処理...
        _ => None,
    }
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

pub fn get_content_for_chatgpt_dell3(v: &Value) -> Result<(String, String), String> {
    // get choices[0].message.content to string
    let data = v["data"].as_array().ok_or("data not found")?;

    // check if data is empty
    if data.is_empty() {
        return Err("choices is empty".to_string());
    }

    let prompt = data[0]["revised_prompt"]
        .as_str()
        .ok_or("prompt not found or not a string")?
        .to_string();

    let url = data[0]["url"]
        .as_str()
        .ok_or("url not found or not a string")?
        .to_string();

    Ok((prompt, url))
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

pub fn get_content_for_gemini(v: &Value) -> Result<String, String> {
    // part.textを取得
    // 'part.text'を取得
    // println!("v: {:?}", v);
    // 値を一度に確認して、unwrapするときにエラーメッセージを指定します
    let result = v
        .get("candidates")
        .and_then(|candidates| candidates.get(0))
        .and_then(|first_candidate| first_candidate.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(|parts| parts.get(0))
        .and_then(|first_part| first_part.get("text"))
        .ok_or(format!(
            "part.text not found or not a string, error: {:?}",
            v
        ))?;

    let result = result
        .as_str()
        .expect("part.text is not a string")
        .to_string();

    // 値が見つかった場合は、文字列に変換して返します
    Ok(result)
}

/// AIが出力したマークダウン用テキストをHTML出力する
pub fn convert_markdown_to_html(text: &str) -> Result<String, String> {
    // KaTeXの数式をレンダリングする
    // TODO

    let mut mathed_op = markdown::ParseOptions::gfm();
    mathed_op.constructs.math_flow = true;
    mathed_op.constructs.math_text = true;
    mathed_op.constructs.gfm_task_list_item = true;
    mathed_op.constructs.gfm_table = true;

    markdown::to_html_with_options(
        text,
        &markdown::Options {
            compile: markdown::CompileOptions::default(),
            parse: mathed_op,
        },
    )
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

    #[test]
    fn test_get_content_for_chatgpt_dell3() {
        let v: Value = match serde_json::from_str(
            r#"{"created":1727198587,"data":[{"revised_prompt":"Visualize a cute, small domestic cat with bright green eyes. It has a soft, fluffy coat, which is a blend of white and ginger colors. The cat is seated comfortably in a cozy, charming setting, perhaps on a soft carpet or by a small warm fireplace. It looks quite content and appears to be purring softly, and the light from the fireplace casts warm, dancing shadows around the room.","url":"https://oaidalleapiprodscus.blob.core.windows.net/private/org-kHS4P2uzP7F6H7Sv9IkJtK5x/user-uxA7ibEcqpioj7QFJFaRzKF1/img-uHR0LPYIEMf8NVDKQjSMSaoB.png?st=2024-09-24T16%3A23%3A07Z&se=2024-09-24T18%3A23%3A07Z&sp=r&sv=2024-08-04&sr=b&rscd=inline&rsct=image/png&skoid=d505667d-d6c1-4a0a-bac7-5c84a87759f8&sktid=a48cca56-e6da-484e-a814-9c849652bcb3&skt=2024-09-23T23%3A19%3A20Z&ske=2024-09-24T23%3A19%3A20Z&sks=b&skv=2024-08-04&sig=5FUWINIWzfLKDYS99GgcMiysHP96TPIawJVW4HyUPe8%3D"}]}"#,
        ) {
            Ok(v) => v,
            Err(e) => panic!("Failed to parse JSON: {}", e),
        };
        assert!(v.is_object());

        let (prompt, url) = match get_content_for_chatgpt_dell3(&v) {
            Ok((prompt, url)) => (prompt, url),
            Err(e) => panic!("Failed to get content: {}", e),
        };

        println!("prompt: {}", prompt);
        println!("url: {}", url);

        assert_eq!(
            prompt,
            "Visualize a cute, small domestic cat with bright green eyes. It has a soft, fluffy coat, which is a blend of white and ginger colors. The cat is seated comfortably in a cozy, charming setting, perhaps on a soft carpet or by a small warm fireplace. It looks quite content and appears to be purring softly, and the light from the fireplace casts warm, dancing shadows around the room."
        );
        assert_eq!(
            url,
            "https://oaidalleapiprodscus.blob.core.windows.net/private/org-kHS4P2uzP7F6H7Sv9IkJtK5x/user-uxA7ibEcqpioj7QFJFaRzKF1/img-uHR0LPYIEMf8NVDKQjSMSaoB.png?st=2024-09-24T16%3A23%3A07Z&se=2024-09-24T18%3A23%3A07Z&sp=r&sv=2024-08-04&sr=b&rscd=inline&rsct=image/png&skoid=d505667d-d6c1-4a0a-bac7-5c84a87759f8&sktid=a48cca56-e6da-484e-a814-9c849652bcb3&skt=2024-09-23T23%3A19%3A20Z&ske=2024-09-24T23%3A19%3A20Z&sks=b&skv=2024-08-04&sig=5FUWINIWzfLKDYS99GgcMiysHP96TPIawJVW4HyUPe8%3D"
        );
    }
}
