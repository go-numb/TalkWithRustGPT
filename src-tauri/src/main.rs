// Prevents additional console window on Windows in release, Ok(DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(target_os = "macos", windows_subsystem = "macos")]

// // my modules
mod manage;
mod sub;
use dotenv::dotenv;
use markdown::message;
use serde_json::json;
use tauri::State;
// Logger
use log::info;
use parking_lot::Mutex as Mut;
use std::env;
use std::sync::Arc;

#[tauri::command]
async fn claude_request(
    b: u8,
    msg: &str,
    src: &str,
    state: State<'_, Arc<Mut<manage::message::Shelf>>>,
) -> Result<String, String> {
    let start_time = chrono::Local::now();

    let (high, low) = manage::claude::model();
    let (set_model, max_tokens) = if b == 1 { (high, 8192) } else { (low, 4096) };

    // add new request message, and get message history
    let messages = {
        let set_src = if src.is_empty() {
            None
        } else {
            Some(src.to_string())
        };
        let mut mut_shelf = state.lock();
        mut_shelf.add_to_messages("user".to_string(), msg.to_string(), set_src);

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
            prompt.content.to_string()
        } else {
            "".to_string()
        }
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
                        "content": manage::claude::to_content(m.clone())
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
                    "content": manage::claude::to_content(m.clone())
                })
            }).collect::<Vec<_>>(),
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
    let history_messages = {
        let mut mut_shelf = state.lock();
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

    let all_messages_string = history_messages
        .iter()
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
async fn chatgpt_request(
    b: u8,
    msg: &str,
    src: &str,
    state: State<'_, Arc<Mut<manage::message::Shelf>>>,
) -> Result<String, String> {
    let start_time = chrono::Local::now();

    let (high, low) = manage::chatgpt::model();
    let (set_model, max_tokens) = if b == 1 { (high, 4096) } else { (low, 16384) };

    // add new request message, and get message history
    let mut messages = {
        let set_src = if src.is_empty() {
            None
        } else {
            Some(src.to_string())
        };
        let mut mut_shelf = state.lock();
        mut_shelf.add_to_messages("user".to_string(), msg.to_string(), set_src);

        let guard_shelf = mut_shelf.clone();
        guard_shelf.get_messages()
    };

    // system prompt append messages
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
        "messages": messages.iter().map(|m| {
            json!({
                "role": m.role,
                "content": manage::chatgpt::to_content(m.clone())
            })
        }).collect::<Vec<_>>(),
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
    let history_messages = {
        let mut mut_shelf = state.lock();
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

    let all_messages_string = history_messages
        .iter()
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
async fn chatgpt_request_to_dell3(size: u8, msg: &str) -> Result<String, String> {
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

#[tauri::command]
async fn gemini_request(
    b: u8,
    msg: &str,
    src: &str,
    state: State<'_, Arc<Mut<manage::message::Shelf>>>,
) -> Result<String, String> {
    let start_time = chrono::Local::now();

    let (high, low) = manage::gemini::model();
    let (set_model, _max_tokens) = if b == 1 {
        (high.as_str(), 8192)
    } else {
        (low.as_str(), 8192)
    };

    // add new request message, and get message history
    let messages = {
        let set_src = if src.is_empty() {
            None
        } else {
            Some(src.to_string())
        };
        let mut mut_shelf = state.lock();
        mut_shelf.add_to_messages("user".to_string(), msg.to_string(), set_src);

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
            prompt.content.to_string()
        } else {
            "".to_string()
        }
    };

    // request
    let body = if system_prompt.is_empty() {
        json!({
            "contents": messages.iter().map(|m| {
                json!({
                    // roleがuserの場合はuser、それ以外はmodel as assistant
                    "role": if m.role == "user" { "user" } else { "model" },
                    "parts": manage::gemini::to_content(m.clone()),
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
                    "parts": manage::gemini::to_content(m.clone()),
                })
            }).collect::<Vec<_>>(),
        })
    };
    let res = match manage::gemini::request(set_model, body).await {
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
        let mut mut_shelf = state.lock();
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

    let all_messages_string = history_messages
        .iter()
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
            chatgpt_request_to_dell3,
            gemini_request,
            memo,
            all_messages
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

fn memo_for_ended(state: Arc<Mut<manage::message::Shelf>>) -> String {
    let mut shelf = state.lock();

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
fn reset(state: State<'_, Arc<Mut<manage::message::Shelf>>>) -> String {
    let mut shelf = state.lock();
    match shelf.reset() {
        Ok(_) => "success reset messages".to_string(),
        Err(e) => format!("messages reset error: {}", e),
    }
}

#[tauri::command]
fn request_system(
    num: u8,
    state: State<'_, Arc<Mut<manage::message::Shelf>>>,
) -> Result<String, String> {
    let prompt: String = sub::prompts::choose(num);

    let mut shelf = state.lock();

    shelf
        .system_messages
        .add("system".to_string(), prompt, None);

    Ok("success".to_string())
}

#[tauri::command]
async fn all_messages(
    state: State<'_, Arc<Mut<manage::message::Shelf>>>,
) -> Result<String, String> {
    let shelf = state.lock();
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
    Ok(all_messages_string)
}
