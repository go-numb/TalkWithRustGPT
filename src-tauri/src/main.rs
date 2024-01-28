// Prevents additional console window on Windows in release, Ok(DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// // my modules
// mod mods;

use chatgpt::prelude::*;
use chatgpt::types::{CompletionResponse, Role};

// markdown
use markdown;

use core::panic;
use std::env;
use std::time::Duration;

use once_cell::sync::Lazy;
use std::sync::Mutex;

// Files
use chrono::prelude::*;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::Write;

use directories::UserDirs;

// LAZY_STATICを使ってgpt_requestから安全にアクセスできるようにします
static CHAT_MESSAGES: Lazy<Mutex<Vec<ChatMessage>>> = Lazy::new(|| Mutex::new(Vec::new()));

const APPNAME: &str = "Talk with RustGPT";

#[tauri::command]
fn memo() -> String {
    // save to dir
    let user_dir: UserDirs = match UserDirs::new() {
        Some(dir) => dir,
        None => panic!("Unable to get user directory"),
    };
    let document_dir = match user_dir.document_dir() {
        Some(doc_dir) => doc_dir,
        None => panic!("Unable to get document directory"),
    };
    let save_dir = document_dir.join(".appdata").join(APPNAME);

    // Create the directory if it doesn't already exist
    create_dir_all(save_dir.as_path()).unwrap();

    // Create a new file to write the data
    let date: DateTime<Local> = Local::now();
    let filename = format!("memo_{}.txt", date.format("%Y-%m-%d-%H%M%S"));

    let path = save_dir.join(filename.as_str());

    println!("save to dir: {:?}", path.to_str());

    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            panic!("Unable create file for memo, error: {}", e);
        }
    };

    // Convert the vector to a string and write it to the file
    let messages = match CHAT_MESSAGES
        .lock()
        .map_err(|err| format!("lazy struct data lock error: {}", err))
    {
        Ok(guard_message) => guard_message.clone(),
        Err(e) => panic!("lazy struct data lock error: {}", e),
    };

    let data_str: String = messages
        .iter()
        .map(|message| {
            println!("memo data: {}", message.content);
            if message.role == Role::User {
                format!("{:?}: {}", message.role, message.content)
            } else {
                format!("{:?}: {}\n----------------", message.role, message.content)
            }
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    let result: String = match file.write_all(data_str.as_bytes()) {
        Ok(_) => format!("memo is success, data written to file successfully"),
        Err(e) => format!("memo is fail, unable to write to file, error: {}", e),
    };

    println!("{}", result.as_str());
    result
}

#[tauri::command]
async fn gpt_request(b: u8, msg: &str) -> std::result::Result<String, String> {
    // 環境変数からAPIキーを取得
    let apikey = match env::var("CHATGPTTOKEN") {
        Ok(val) => val,
        Err(e) => format!("couldn't interpret CHATGPTTOKEN: {}", e).to_string(),
    };

    let mut set_model: ChatGPTEngine = ChatGPTEngine::Custom("gpt-3.5-turbo-1106");
    if b == 1 {
        set_model = ChatGPTEngine::Custom("gpt-4-0125-preview");
    }

    let client = match ChatGPT::new_with_config(
        apikey,
        ModelConfigurationBuilder::default()
            .timeout(Duration::new(120, 0))
            .temperature(1.0)
            .engine(set_model)
            .build()
            .unwrap(),
    ) {
        Ok(client) => client,
        Err(e) => return Err(format!("ChatGPT client error: {}", e)),
    };

    // メッセージヒストリをロックしていますが、エラーが起きた場合はString型のエラーメッセージを返します。
    let messages = {
        let mut guard_message = CHAT_MESSAGES
            .lock()
            .map_err(|err| format!("lazy struct data lock error: {}", err))?;
        guard_message.push(ChatMessage {
            role: Role::User,
            content: msg.to_string(),
        });
        guard_message.clone()
    };

    // ChatGPT サーバにリクエストを送信し、結果を待ちますが、エラーがあれば String 型で返します。
    let res: CompletionResponse = client
        .send_history(&messages)
        .await
        .map_err(|err| format!("ChatGPT client error: {}", err))?;

    let markdown_content =
        markdown::to_html_with_options(res.message().content.as_str(), &markdown::Options::gfm())?;

    // 応答メッセージをヒストリに追加
    let msg = format!(
        "{}\n\nModel: {}, Total token: {}",
        markdown_content, res.model, res.usage.total_tokens
    );

    match CHAT_MESSAGES
        .lock()
        .map_err(|err| format!("lazy struct data lock error: {}", err))
    {
        Ok(mut guard_message) => {
            guard_message.push(res.message().clone());
            // メッセージ履歴を表示
            for (i, message) in messages.iter().enumerate() {
                println!("{} - {:?}", i, message);
            }
        }
        Err(e) => return Err(format!("lazy struct data lock error: {}", e)),
    }

    Ok(msg)
}

#[tauri::command]
fn gpt_reset_messages() {
    match CHAT_MESSAGES
        .lock()
        .map_err(|err| format!("lazy struct data lock error: {}", err))
    {
        Ok(mut guard_message) => {
            guard_message.clear();
        }
        Err(e) => println!("lazy struct data lock error: {}", e),
    }
    println!("gpt_reset_messages is success");
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            gpt_request,
            gpt_reset_messages,
            memo
        ])
        .on_window_event(move |event| match event.event() {
            // ウィンドウイベントを監視
            // ウィンドウ終了時に履歴をメモします
            tauri::WindowEvent::Destroyed => {
                println!("Window destroyed");
                memo();
                let _ = event.window().close();
            }
            _ => (),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
