use dotenv::dotenv;
use serde_json::Value;
use std::env;
use std::error::Error;
use std::path::Path;
use std::result::Result;

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
    #[allow(unused)]
    pub voice_id: Option<i16>,
}

/// 必要な環境変数を取得する
/// VOICEID は省略可能。未設定または不正値の場合は None
pub async fn get_env() -> Result<Keys, Box<dyn Error>> {
    dotenv().ok();
    let anthropic_key = env::var("ANTHROPIC_API_KEY")?;
    let openai_token = env::var("CHATGPTTOKEN")?;
    let google_key = env::var("GOOGLE_GEMINI_API_KEY")?;
    let voice_id = env::var("VOICEID").ok().and_then(|v| v.parse().ok());

    Ok(Keys {
        anthropic_key,
        openai_token,
        google_key,
        voice_id,
    })
}

#[allow(unused)]
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

pub fn get_content_for_chatgpt(v: &Value) -> Result<(String, u64), String> {
    let choices = v["choices"].as_array().ok_or("choices not found")?;
    if choices.is_empty() {
        return Err("choices is empty".to_string());
    }
    let text = choices[0]["message"]["content"]
        .as_str()
        .ok_or("content not found or not a string")?
        .to_string();
    let tokens = v["usage"]["total_tokens"].as_u64().unwrap_or(0);
    Ok((text, tokens))
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

pub fn get_content_for_claude(v: &Value) -> Result<(String, u64), String> {
    let content = v["content"].as_array().ok_or("content not found")?;
    if content.is_empty() {
        return Err("content is empty".to_string());
    }
    let text = content[0]["text"]
        .as_str()
        .ok_or("text field not found or not a string")?
        .to_string();
    let input = v["usage"]["input_tokens"].as_u64().unwrap_or(0);
    let output = v["usage"]["output_tokens"].as_u64().unwrap_or(0);
    Ok((text, input + output))
}

pub fn get_content_for_gemini(v: &Value) -> Result<(String, u64), String> {
    let text = v
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .ok_or(format!("part.text not found, error: {:?}", v))?
        .as_str()
        .expect("part.text is not a string")
        .to_string();
    let tokens = v["usageMetadata"]["totalTokenCount"].as_u64().unwrap_or(0);
    Ok((text, tokens))
}

/// AIが出力したマークダウン用テキストをHTML出力する
/// ammonia でXSS対策のサニタイズを行う
pub fn convert_markdown_to_html(text: &str) -> Result<String, String> {
    let mut mathed_op = markdown::ParseOptions::gfm();
    mathed_op.constructs.math_flow = true;
    mathed_op.constructs.math_text = true;
    mathed_op.constructs.gfm_task_list_item = true;
    mathed_op.constructs.gfm_table = true;

    let html = markdown::to_html_with_options(
        text,
        &markdown::Options {
            compile: markdown::CompileOptions::default(),
            parse: mathed_op,
        },
    )
    .map_err(|e| format!("markdown::to_html_with_options error: {}", e))?;

    Ok(ammonia::clean(&html))
}

/// invokeへの返り値を作成する
pub fn create_response(
    markdown_content: &str,
    set_model: &str,
    token_count: u64,
    start: chrono::DateTime<chrono::Local>,
) -> String {
    let end = chrono::Local::now();
    format!(
        "{}\n\nModel: {}, Total token: {}, Elaps: {}s",
        markdown_content,
        set_model,
        token_count,
        end.signed_duration_since(start).num_seconds(),
    )
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
    use super::*;

    #[test]
    fn test_convert_markdown_strips_script_tags() {
        let malicious = "hello\n\n<script>alert('xss')</script>\n\nworld";
        let html = convert_markdown_to_html(malicious).unwrap();
        assert!(!html.contains("<script>"), "script tag should be removed: {}", html);
        assert!(!html.contains("alert("), "script content should be removed: {}", html);
    }

    #[test]
    fn test_convert_markdown_strips_onerror_attribute() {
        let malicious = "![img](x)<img src=x onerror=\"alert(1)\">";
        let html = convert_markdown_to_html(malicious).unwrap();
        assert!(!html.contains("onerror"), "onerror attribute should be removed: {}", html);
    }

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
        // VOICEID は省略可能なので None も有効
        if let Some(id) = keys.voice_id {
            assert!(id > 0, "voice_id is wrong, {}", id);
        }
    }

    #[tokio::test]
    async fn test_get_env_succeeds_without_voiceid() {
        // VOICEID を未設定の状態で get_env() がエラーにならないことを確認
        std::env::remove_var("VOICEID");
        std::env::set_var("ANTHROPIC_API_KEY", "dummy_anthropic");
        std::env::set_var("CHATGPTTOKEN", "dummy_openai");
        std::env::set_var("GOOGLE_GEMINI_API_KEY", "dummy_google");

        let result = get_env().await;
        assert!(result.is_ok(), "VOICEID 未設定時に get_env() が失敗した: {:?}", result.err());

        // 後片付け
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("CHATGPTTOKEN");
        std::env::remove_var("GOOGLE_GEMINI_API_KEY");
    }

    #[test]
    fn test_get_content_for_chatgpt_returns_token_count() {
        let v: Value = serde_json::from_str(r#"{
            "choices": [{"message": {"role": "assistant", "content": "Hello"}}],
            "usage": {"prompt_tokens": 9, "completion_tokens": 12, "total_tokens": 21}
        }"#).unwrap();
        let (_, tokens) = get_content_for_chatgpt(&v).unwrap();
        assert_eq!(tokens, 21);
    }

    #[test]
    fn test_get_content_for_claude_returns_token_count() {
        let v: Value = serde_json::from_str(r#"{
            "content": [{"type": "text", "text": "Hello"}],
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#).unwrap();
        let (_, tokens) = get_content_for_claude(&v).unwrap();
        assert_eq!(tokens, 15);
    }

    #[test]
    fn test_get_content_for_gemini_returns_token_count() {
        let v: Value = serde_json::from_str(r#"{
            "candidates": [{"content": {"parts": [{"text": "Hello"}]}}],
            "usageMetadata": {"totalTokenCount": 30}
        }"#).unwrap();
        let (_, tokens) = get_content_for_gemini(&v).unwrap();
        assert_eq!(tokens, 30);
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

        let (content, tokens) = match get_content_for_chatgpt(&v) {
            Ok(v) => v,
            Err(e) => panic!("Failed to get content: {}", e),
        };

        assert_eq!(content, "\n\nHello there, how may I assist you today?");
        assert_eq!(tokens, 21);
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
