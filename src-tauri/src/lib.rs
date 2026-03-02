use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use log::info;
use tauri::State;

use crate::manage::utils::convert_markdown_to_html;

mod manage;
mod sub;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenv::dotenv().ok();
    env_logger::init();

    let shelf = Arc::new(Mutex::new(manage::message::Shelf::new()));
    let clone_shelf = shelf.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(shelf)
        .invoke_handler(tauri::generate_handler![
            is_there_env,
            reset,
            request_system,
            manage::claude::claude_request,
            manage::chatgpt::chatgpt_request,
            manage::chatgpt::chatgpt_request_to_dell3,
            manage::gemini::gemini_request,
            memo,
            all_messages,
            files_to_string,
        ])
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                // ウィンドウイベントを監視
                // ウィンドウ終了時に履歴をメモします
                info!("Window destroyed");
                memo_for_ended(clone_shelf.clone());
                let _ = window.close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn is_there_env() -> bool {
    let chatgpt = std::env::var("CHATGPTTOKEN");
    let anth = std::env::var("ANTHROPIC_API_KEY");
    // 上記環境変数一方が存在すればTrue, どちらもなければFalse
    chatgpt.is_ok() || anth.is_ok()
}

#[tauri::command]
fn memo(state: State<'_, Arc<Mutex<manage::message::Shelf>>>) -> String {
    let shelf = state.lock().unwrap();

    // if memo has messages
    let messages = shelf.get_messages();
    if messages.is_empty() {
        return "no messages".to_string();
    }

    match shelf.memo() {
        Ok(_) => "memo is success".to_string(),
        Err(e) => format!("memo error: {}", e),
    }
}

fn memo_for_ended(state: Arc<Mutex<manage::message::Shelf>>) -> String {
    let mut shelf = state.lock().unwrap();

    // if memo has messages
    let messages = shelf.get_messages();
    if messages.is_empty() {
        return "no messages".to_string();
    }

    match shelf.memo() {
        Ok(_) => {
            shelf.reset().unwrap();
            "memo is success".to_string()
        }
        Err(e) => format!("memo error: {}", e),
    }
}

#[tauri::command]
fn reset(state: State<'_, Arc<Mutex<manage::message::Shelf>>>) -> String {
    let mut shelf = state.lock().unwrap();
    match shelf.reset() {
        Ok(_) => "success reset messages".to_string(),
        Err(e) => format!("messages reset error: {}", e),
    }
}

#[tauri::command]
fn request_system(
    num: u8,
    state: State<'_, Arc<Mutex<manage::message::Shelf>>>,
) -> Result<String, String> {
    let prompt: String = sub::prompts::choose(num);

    let mut shelf = state.lock().unwrap();

    shelf.system_messages.reset();
    shelf
        .system_messages
        .add("system".to_string(), prompt, None);

    Ok("success".to_string())
}

#[tauri::command(rename_all = "snake_case")]
async fn all_messages(
    is_raw: bool,
    state: State<'_, Arc<Mutex<manage::message::Shelf>>>,
) -> Result<String, String> {
    let shelf = state.lock().unwrap();
    let messages = shelf.get_messages();

    if messages.is_empty() {
        return Err("no message history".to_string());
    }

    let all_messages_string = messages
        .iter()
        .map(|message| {
            if message.role == "user" {
                format!("{}: {}\n\n", message.role, message.content)
            } else {
                format!(
                    "{}: {}\n--------------------\n\n",
                    message.role, message.content
                )
            }
        })
        .collect::<String>();

    if is_raw {
        // 出力を整形せず出力する
        // \n to <br> 変換
        return Ok(all_messages_string.replace("\n", "<br>"));
    }

    match convert_markdown_to_html(&all_messages_string) {
        Ok(html) => Ok(html),
        Err(e) => Err(format!("Error converting to HTML: {}", e)),
    }
}

#[tauri::command]
fn files_to_string(filepaths: Vec<PathBuf>) -> String {
    let mut all_messages_string = String::new();
    for filepath in filepaths {
        if filepath.is_dir() || !filepath.exists() {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(filepath.clone()) {
            let ext = filepath.extension();
            let extention = ext.unwrap_or_default();
            all_messages_string.push_str(&format!(
                "{}\n```{}\n{}\n```\n\n",
                filepath.display(),
                extention.to_string_lossy(),
                content
            ));
        }
    }

    if all_messages_string.is_empty() {
        return "no files".to_string();
    }

    all_messages_string
}
