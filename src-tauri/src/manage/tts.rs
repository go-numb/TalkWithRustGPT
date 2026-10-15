use std::time::Duration;

use base64::Engine;
use reqwest::Client;
use serde_json::{json, Value};

const INTERACTIONS_URL: &str = "https://generativelanguage.googleapis.com/v1beta/interactions";
// 2026-10 時点の最新 TTS モデル。GEMINI_TTS_MODEL で上書きできる。
const DEFAULT_MODEL: &str = "gemini-3.8-flash-tts";
// 入力言語を自動判別する多言語のプリセット音声 (Voice Library の gender で性別を確認済み)。
// ja-jp-* などのロケール固有の音声は、英文を日本語に翻訳して読むことがあるため既定にしない。
// Erinome: "Clear, articulate, and precise" / Charon: "Informative and steady"
const DEFAULT_FEMALE_VOICE: &str = "Erinome";
const DEFAULT_MALE_VOICE: &str = "Charon";
// 回答は原文のまま読む。ロケール固有の音声を環境変数で指定した場合の翻訳も防ぐ。
const STYLE: &str = "Read the transcript exactly as written, in its original language. \
Never translate. Clear, calm, and natural narration at a steady pace.";
// フロントエンドは文単位で分割して送る。1 回のリクエストが長くなり過ぎないよう上限を設ける。
const MAX_CHARS: usize = 2000;

/// テキストを Gemini TTS で読み上げ音声 (WAV, 24kHz, mono, 16bit) に変換する
/// voice: "female" または "male"
#[tauri::command]
pub async fn gemini_tts(text: String, voice: String) -> Result<tauri::ipc::Response, String> {
    // 音声はバイナリのまま渡す (JSON 配列にしない)。フロントエンドでは ArrayBuffer になる。
    Ok(tauri::ipc::Response::new(synthesize(&text, &voice).await?))
}

async fn synthesize(text: &str, voice: &str) -> Result<Vec<u8>, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("読み上げるテキストがありません".to_string());
    }
    if text.chars().count() > MAX_CHARS {
        return Err(format!(
            "読み上げテキストが長すぎます ({}文字以内)",
            MAX_CHARS
        ));
    }
    let voice_name = voice_name(voice)?;
    let key = std::env::var("GOOGLE_GEMINI_API_KEY")
        .map_err(|_| "GOOGLE_GEMINI_API_KEY が設定されていません".to_string())?;

    let body = request_body(&model(), &voice_name, text);
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| format!("TTS client error: {}", e))?;
    let res = client
        .post(INTERACTIONS_URL)
        .header("x-goog-api-key", key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("TTS request error: {}", e))?;

    let status = res.status();
    let value: Value = res
        .json()
        .await
        .map_err(|e| format!("TTS JSON parse error ({}): {}", status, e))?;
    if !status.is_success() {
        return Err(api_error(status.as_u16(), &value));
    }

    extract_audio(&value)
}

fn model() -> String {
    env_or("GEMINI_TTS_MODEL", DEFAULT_MODEL)
}

fn voice_name(voice: &str) -> Result<String, String> {
    match voice {
        "female" => Ok(env_or("GEMINI_TTS_VOICE_FEMALE", DEFAULT_FEMALE_VOICE)),
        "male" => Ok(env_or("GEMINI_TTS_VOICE_MALE", DEFAULT_MALE_VOICE)),
        _ => Err(format!("unknown voice: {}", voice)),
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn request_body(model: &str, voice: &str, text: &str) -> Value {
    // Gemini 3.8 TTS は text を逐語の台本として読むため、話し方は speech_metadata に分ける。
    json!({
        "model": model,
        "input": [{
            "type": "user_input",
            "content": [{
                "type": "text",
                "text": text,
                "annotations": [{
                    "type": "speech_metadata",
                    "style": STYLE,
                }],
            }],
        }],
        "response_format": { "type": "audio" },
        "generation_config": {
            "speech_config": [{ "voice": voice }],
        },
    })
}

/// 最後の model_output に含まれる音声を取り出す (unary 応答は RIFF ヘッダ付き WAV)
fn extract_audio(v: &Value) -> Result<Vec<u8>, String> {
    let data = v["steps"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|step| step["type"] == "model_output")
        .flat_map(|step| step["content"].as_array().into_iter().flatten())
        .filter(|content| content["type"] == "audio")
        .filter_map(|content| content["data"].as_str())
        .last()
        .ok_or_else(|| format!("TTS audio not found (status: {})", v["status"]))?;

    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("TTS audio decode error: {}", e))
}

fn api_error(status: u16, v: &Value) -> String {
    let message = v["error"]["message"].as_str().unwrap_or("unknown error");
    format!("TTS API error ({}): {}", status, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_name_maps_gender_and_rejects_unknown() {
        std::env::remove_var("GEMINI_TTS_VOICE_FEMALE");
        std::env::remove_var("GEMINI_TTS_VOICE_MALE");
        assert_eq!(voice_name("female").unwrap(), DEFAULT_FEMALE_VOICE);
        assert_eq!(voice_name("male").unwrap(), DEFAULT_MALE_VOICE);
        assert!(voice_name("Kore").is_err());
    }

    #[test]
    fn test_request_body_keeps_style_out_of_transcript() {
        let body = request_body("gemini-3.8-flash-tts", "Charon", "こんにちは");
        let content = &body["input"][0]["content"][0];
        assert_eq!(body["model"], "gemini-3.8-flash-tts");
        assert_eq!(content["text"], "こんにちは");
        assert_eq!(content["annotations"][0]["type"], "speech_metadata");
        assert!(content["annotations"][0]["style"]
            .as_str()
            .unwrap()
            .contains("Never translate"));
        assert_eq!(body["response_format"]["type"], "audio");
        assert_eq!(
            body["generation_config"]["speech_config"][0]["voice"],
            "Charon"
        );
    }

    #[test]
    fn test_extract_audio_decodes_last_model_output() {
        let v = json!({
            "status": "completed",
            "steps": [
                { "type": "model_output", "content": [{ "type": "audio", "data": "AAAA" }] },
                { "type": "model_output", "content": [
                    { "type": "text", "text": "ignored" },
                    { "type": "audio", "mime_type": "audio/wav", "data": "UklGRg==" }
                ] }
            ]
        });
        assert_eq!(extract_audio(&v).unwrap(), b"RIFF");
    }

    #[test]
    fn test_extract_audio_reports_missing_audio() {
        let v = json!({ "status": "failed", "steps": [] });
        let err = extract_audio(&v).unwrap_err();
        assert!(err.contains("failed"), "{}", err);
    }

    #[test]
    fn test_api_error_uses_message() {
        let v = json!({ "error": { "code": 400, "message": "Invalid voice" } });
        assert_eq!(api_error(400, &v), "TTS API error (400): Invalid voice");
    }

    /// 実 API を呼ぶ。GOOGLE_GEMINI_API_KEY を設定して `cargo test -- --ignored live` で実行する。
    #[tokio::test]
    #[ignore]
    async fn live_gemini_tts_returns_wav() {
        dotenv::dotenv().ok();
        for voice in ["female", "male"] {
            let wav = synthesize("読み上げのテストです。This is a test.", voice)
                .await
                .unwrap_or_else(|e| panic!("{}: {}", voice, e));
            assert_eq!(&wav[..4], b"RIFF", "{}", voice);
            assert!(wav.len() > 44, "{}", voice);
        }
    }
}
