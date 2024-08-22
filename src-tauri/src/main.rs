// Prevents additional console window on Windows in release, Ok(DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// // my modules
mod manage;
mod sub;

use tauri::State;

use serde_json::json;

use dotenv::dotenv;

// Logger
use log::info;

use std::env;

use std::sync::Arc;

use parking_lot::Mutex as Mut;

// const APPNAME: &str = "Talk with RustGPT";

#[tauri::command]
fn request_system(
    num: u8,
    state: State<'_, Arc<Mut<manage::message::Shelf>>>,
) -> Result<String, String> {
    let prompt: String = sub::prompts::choose(num);

    let mut shelf = state.lock();

    let system_message = json!([{
        "type": "text",
        "text": prompt,
    }]);
    shelf
        .system_messages
        .add("system".to_string(), system_message);

    Ok("success".to_string())
}

#[tauri::command]
async fn claude_request(
    b: u8,
    msg: &str,
    src: &str,
    state: State<'_, Arc<Mut<manage::message::Shelf>>>,
) -> Result<String, String> {
    let start_time = chrono::Local::now();

    let (set_model, max_tokens) = if b == 1 {
        ("claude-3-5-sonnet-20240620", 8192)
    } else {
        ("claude-3-opus-20240229", 4096)
    };

    // create request content
    let content = manage::utils::create_content_for_claude(msg, src);

    // add new request message, and get message history
    let messages = {
        let mut mut_shelf = state.lock();
        mut_shelf.add_to_messages("user".to_string(), content);

        let guard_shelf = mut_shelf.clone();
        guard_shelf.get_messages()
    };

    // get system prompt
    // meke simple string for claude
    let system_prompt = {
        let mut_shelf = state.lock();
        let system_prompt = mut_shelf.system_messages.get();
        if !system_prompt.is_empty() {
            // 最期の配列をStringで出力
            let prompt = system_prompt.last().unwrap();
            let prompt_text = prompt.content["text"].as_str().unwrap_or("");
            prompt_text.to_string()
        } else {
            "".to_string()
        }
    };

    // request
    let body = if system_prompt.is_empty() {
        json!({
            "model": set_model,
            "max_tokens": max_tokens,
            "messages": messages,
        })
    } else {
        json!({
            "model": set_model,
            "max_tokens": max_tokens,
            "messages": messages,
            "system": system_prompt,
        })
    };
    let res = match manage::claude::request(body).await {
        Ok(res) => res,
        Err(e) => return Err(format!("Request error: {}", e)),
    };

    // get response message
    let text = match manage::utils::get_content_for_claude(&res) {
        Ok(text) => text,
        Err(e) => format!("Error: {}", e),
    };

    // メッセージを履歴に追加
    // create request content
    let result_content = manage::utils::create_content_for_claude(&text, "");

    let history_messages = {
        let mut mut_shelf = state.lock();
        mut_shelf.add_to_messages("assistant".to_string(), result_content);
        let guard_shelf = mut_shelf.clone();
        guard_shelf.get_messages()
    };

    // for (index, message) in history_messages.iter().enumerate() {
    //     println!(
    //         "{} - role: {}, content: {}",
    //         index, message.role, message.content
    //     );
    // }

    let all_messages_string = history_messages
        .iter()
        .map(|message| {
            message.content[0]["text"]
                .as_str()
                .unwrap_or("none")
                .to_string()
        })
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

#[tauri::command]
async fn chatgpt_request(
    b: u8,
    msg: &str,
    src: &str,
    state: State<'_, Arc<Mut<manage::message::Shelf>>>,
) -> Result<String, String> {
    let start_time = chrono::Local::now();

    let (set_model, max_tokens) = if b == 1 {
        ("gpt-4o", 4096)
    } else {
        ("gpt-4o-mini", 16384)
    };

    // create request content
    let content = manage::utils::create_content_for_chatgpt(msg, src);

    // add new request message, and get message history
    let mut messages = {
        let mut mut_shelf = state.lock();
        mut_shelf.add_to_messages("user".to_string(), content);

        let guard_shelf = mut_shelf.clone();
        guard_shelf.get_messages()
    };

    // get system prompt
    let messages = {
        let mut_shelf = state.lock();
        let system_prompt = mut_shelf.system_messages.get();
        if let Some(prompt) = system_prompt.last() {
            messages.push(prompt.clone());
        }
        messages
    };

    // request
    let body = json!({
        "model": set_model,
        "max_tokens": max_tokens,
        "messages": messages,
    });
    let res = match manage::chatgpt::request(body).await {
        Ok(res) => res,
        Err(e) => return Err(format!("Request error: {}", e)),
    };

    // get response message
    let text = match manage::utils::get_content_for_chatgpt(&res) {
        Ok(text) => text,
        Err(e) => format!("Error: {}", e),
    };

    // メッセージを履歴に追加
    // create request content
    let result_content = manage::utils::create_content_for_chatgpt(&text, "");

    let history_messages = {
        let mut mut_shelf = state.lock();
        mut_shelf.add_to_messages("assistant".to_string(), result_content);
        let guard_shelf = mut_shelf.clone();
        guard_shelf.get_messages()
    };

    // for (index, message) in history_messages.iter().enumerate() {
    //     println!(
    //         "{} - role: {}, content: {}",
    //         index, message.role, message.content
    //     );
    // }

    let all_messages_string = history_messages
        .iter()
        .map(|message| {
            message.content[0]["text"]
                .as_str()
                .unwrap_or("none")
                .to_string()
        })
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

fn main() {
    dotenv().ok();
    env_logger::init();

    let shelf = Arc::new(Mut::new(manage::message::Shelf::new()));
    let clone_shelf = shelf.clone();

    tauri::Builder::default()
        .manage(shelf)
        .invoke_handler(tauri::generate_handler![
            is_there_env,
            reset,
            request_system,
            claude_request,
            chatgpt_request,
            memo
        ])
        .on_window_event(move |event| {
            if let tauri::WindowEvent::Destroyed = event.event() {
                // ウィンドウイベントを監視
                // ウィンドウ終了時に履歴をメモします
                info!("Window destroyed");
                memo_for_ended(clone_shelf.clone());
                let _ = event.window().close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn is_there_env() -> bool {
    let chatgpt = env::var("CHATGPTTOKEN");
    let anth = env::var("ANTHROPIC_API_KEY");
    // 上記環境変数一方が存在すればTrue, どちらもなければFalse
    chatgpt.is_ok() || anth.is_ok()
}

#[tauri::command]
fn memo(state: State<'_, Arc<Mut<manage::message::Shelf>>>) -> String {
    let shelf = state.lock();
    match shelf.memo() {
        Ok(_) => "memo is success".to_string(),
        Err(e) => format!("memo error: {}", e),
    }
}

fn memo_for_ended(state: Arc<Mut<manage::message::Shelf>>) -> String {
    let shelf = state.lock();
    match shelf.memo() {
        Ok(_) => "memo is success".to_string(),
        Err(e) => format!("memo error: {}", e),
    }
}

#[tauri::command]
fn reset(state: State<'_, Arc<Mut<manage::message::Shelf>>>) -> String {
    let mut shelf = state.lock();
    match shelf.reset() {
        Ok(_) => "success reset messages".to_string(),
        Err(e) => format!("messages reset error: {}", e),
    }
}
