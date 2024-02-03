// Prevents additional console window on Windows in release, Ok(DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// // my modules
mod mods;

// new OpneAI
use futures::StreamExt;
use rs_openai::chat::Role as ChatGPTRole;
use rs_openai::chat::{
    ChatCompletionMessage, ChatCompletionMessageRequestBuilder, CreateChatRequestBuilder,
};
use rs_openai::OpenAI;
use tiktoken_rs::cl100k_base;

use core::panic;
use std::env;

use once_cell::sync::Lazy;
use std::sync::Mutex;

// Files
use chrono::prelude::*;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::Write;

use directories::UserDirs;

#[derive(Clone, Debug)]
struct Message {
    role: String,
    content: String,
}
// LAZY_STATICを使ってgpt_requestから安全にアクセスできるようにします
static MESSAGES: Lazy<Mutex<Vec<Message>>> = Lazy::new(|| Mutex::new(Vec::new()));

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
    let messages = match MESSAGES
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
            if message.role == ChatGPTRole::User.to_string() {
                format!("{:?}: {}", message.role, message.content)
            } else {
                format!("{:?}: {}\n----------------", message.role, message.content)
            }
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    let result: String = match file.write_all(data_str.as_bytes()) {
        Ok(_) => "memo is success, data written to file successfully".to_string(),
        Err(e) => format!("memo is fail, unable to write to file, error: {}", e),
    };

    println!("{}", result.as_str());
    result
}

#[tauri::command]
async fn gpt_stream_request(b: u8, msg: &str) -> std::result::Result<String, String> {
    // タイムスタンプを取得
    let start = Local::now();

    // 環境変数からAPIキーを取得
    let apikey = match env::var("CHATGPTTOKEN") {
        Ok(val) => val,
        Err(e) => format!("couldn't interpret CHATGPTTOKEN: {}", e).to_string(),
    };

    // create client with APIKEY
    let client = OpenAI::new(&OpenAI {
        api_key: apikey,
        org_id: None,
    });

    let mut set_model: &str = "gpt-3.5-turbo-0125";
    if b == 1 {
        set_model = "gpt-4-0125-preview";
    }

    // メッセージ履歴に保存する
    // グローバル変数のロックを短くするため、リクエストをはさみ二度アクセスしている
    let messages: Vec<Message> = {
        let mut guard_messages: std::sync::MutexGuard<'_, Vec<Message>> = MESSAGES.lock().map_err(
            |err: std::sync::PoisonError<std::sync::MutexGuard<'_, Vec<Message>>>| {
                format!("lazy struct data lock error: {}", err)
            },
        )?;
        guard_messages.push(Message {
            role: ChatGPTRole::User.to_string(),
            content: msg.to_string(),
        });

        guard_messages.clone()
    };

    // 履歴を渡すために、ChatCompletionMessageに変換します
    let pass_vec: Vec<ChatCompletionMessage> = messages
        .iter()
        .map(|message| {
            ChatCompletionMessageRequestBuilder::default()
                .role(ChatGPTRole::User)
                .content(message.content.clone())
                .name(message.role.clone())
                .build()
                .unwrap()
        })
        .collect();

    // リクエストボディを作成
    let req = match CreateChatRequestBuilder::default()
        .model(set_model.to_string())
        .stream(true)
        .messages(pass_vec)
        .build()
    {
        Ok(req) => req,
        Err(e) => return Err(format!("CreateChatRequestBuilder error: {}", e)),
    };

    // リクエストを送信
    let mut stream = match client.chat().create_with_stream(&req).await {
        Ok(stream) => stream,
        Err(e) => return Err(format!("client.chat().create_with_stream error: {}", e)),
    };

    let mut result = String::new();
    let mut delta = String::new();
    while let Some(response) = stream.next().await {
        response.unwrap().choices.iter().for_each(|choice| {
            if let Some(ref content) = choice.delta.content {
                delta.push_str(content);
            }
        });

        // Stop文字を定義し、中途処理を行います
        if delta.ends_with('.') || delta.ends_with('。') || delta.ends_with('\n') {
            result.push_str(&delta);

            // メッセージを発言
            // 棒読みちゃんが起動していない場合は無視します
            mods::voice::say(delta.as_str());
            // デルタ文字列を初期化
            delta = String::new();
        }
    }

    // マークダウン整形
    let markdown_content =
        match markdown::to_html_with_options(result.as_str(), &markdown::Options::gfm()) {
            Ok(content) => content,
            Err(e) => return Err(format!("markdown::to_html_with_options error: {}", e)),
        };

    // レスポンスをメッセージ履歴に保存し
    // メッセージ履歴を表示
    // Streamでは取れないトークン数を計算する
    let mut tokenize_resource: String = String::new();
    match MESSAGES
        .lock()
        .map_err(|err| format!("lazy struct data lock error: {}", err))
    {
        Ok(mut guard_messages) => {
            let pass_msg = Message {
                role: ChatGPTRole::Assistant.to_string(),
                content: result.to_string(),
            };
            guard_messages.push(pass_msg);
            // メッセージ履歴を表示
            for message in guard_messages.iter() {
                tokenize_resource.push_str(message.content.as_str());
            }
        }
        Err(e) => return Err(format!("lazy struct data lock error: {}", e)),
    }

    // 応答メッセージをヒストリに追加
    let end = Local::now();
    let bpe = cl100k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(tokenize_resource.as_str());
    let msg = format!(
        "{}\n\nModel: {}, Total token: {}, Elaps: {}s",
        markdown_content,
        req.model,
        tokens.len(),
        end.signed_duration_since(start).num_seconds(),
    );

    // マークダウン整形済みのメッセージを返します
    Ok(msg)
}

#[tauri::command]
fn gpt_reset_messages() {
    match MESSAGES
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
            gpt_stream_request,
            gpt_reset_messages,
            memo
        ])
        .on_window_event(move |event| {
            if let tauri::WindowEvent::Destroyed = event.event() {
                // ウィンドウイベントを監視
                // ウィンドウ終了時に履歴をメモします
                println!("Window destroyed");
                memo();
                let _ = event.window().close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
