// Prevents additional console window on Windows in release, Ok(DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// // my modules
mod mods;

use dotenv::dotenv;
// use openai_api_rs::v1::message;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Logger
use log::{error, info};

// new OpneAI
// use futures::StreamExt;
use rs_openai::chat::Role as ChatGPTRole;
// use rs_openai::chat::{
//     ChatCompletionMessage, ChatCompletionMessageRequestBuilder, CreateChatRequestBuilder,
// };
// use rs_openai::OpenAI;
use tiktoken_rs::cl100k_base;

// new openai
use tauri::regex::Regex;

use openai_api_rs::v1::api::Client as OClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
// use openai_api_rs::v1::common::GPT4_O;

use std::vec;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}
#[derive(Serialize, Deserialize)]
struct RequestBody {
    model: String,
    system: Option<String>,
    max_tokens: u32,
    messages: Vec<Message>,
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

    info!("save to dir: {:?}", path.to_str());

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
            info!("memo data: {}", message.content);
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

    info!("{}", result.as_str());
    result
}

#[tauri::command]
async fn claude_request(b: u8, msg: &str) -> std::result::Result<String, String> {
    // タイムスタンプを取得
    let start = Local::now();

    // 環境変数からAPIキーを取得
    let api_key = env::var("ANTHROPIC_API_KEY").expect("Expected an API key");

    let mut set_model: &str = "claude-3-sonnet-20240229";
    if b == 1 {
        set_model = "claude-3-opus-20240229";
    }

    let (messages, system_message_content) = {
        let mut guard_messages = MESSAGES
            .lock()
            .map_err(|err| format!("lazy struct data lock error: {}", err))?;

        guard_messages.push(Message {
            role: ChatGPTRole::User.to_string(),
            content: msg.to_string(),
        });

        let (filtered_messages, system_messages): (Vec<Message>, Vec<Message>) = guard_messages
            .iter()
            .cloned()
            .partition(|message| message.role != "system");

        let system_message_content = system_messages
            .first()
            .map(|message| message.content.clone())
            .unwrap_or_else(String::new);

        (filtered_messages, system_message_content)
    };

    // println!("system: {:?}", system_message_content);
    // println!("messages: {:?}", messages);

    // クライアントを作成
    let client = Client::new();
    let body = RequestBody {
        model: set_model.to_string(),
        system: Some(system_message_content),
        max_tokens: 4096,
        messages,
    };

    // リクエストを送信
    let res = match client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(err) => {
            println!("Error: {}", err);
            return Err(format!("Request error: {}", err));
        }
    };

    // レスポンスボディをテキストとして表示（必要に応じて）
    let res_json: Value = match res.json().await {
        Ok(res) => res,
        Err(err) => {
            print!("{}", err);
            Value::Null
        }
    };

    // // `content[0].text`にアクセス
    // print!("{:}", res_json);

    let result =
        if let Some(first_content) = res_json["content"].as_array().and_then(|arr| arr.first()) {
            if let Some(text_value) = first_content["text"].as_str() {
                // println!("content[0].textの値: {}", text_value);
                text_value.to_string()
            } else {
                println!("`text`キーの型がstringではありません。");
                return Err(format!(
                    "`text`キーの型がstringではありません。res: {}",
                    res_json
                ));
            }
        } else {
            println!(
                "`content`配列が空、または`content`キーが存在しません。res: {}",
                res_json
            );
            return Err(format!(
                "`content`配列が空、または`content`キーが存在しません。res: {}",
                res_json
            ));
        };

    // VoiceIDの指定を読み込み
    let is_voice: bool;
    let voice_id: i16 = match env::var("VOICEID") {
        Ok(val) => {
            is_voice = true;
            info!("VOICEID: {}", val);
            val.parse().unwrap()
        }
        Err(e) => {
            info!("couldn't interpret VOICEID: {}", e);
            is_voice = false;
            1
        }
    };
    // メッセージを発言
    // 棒読みちゃんが起動していない場合は無視します
    if is_voice {
        match mods::voice::say(voice_id, result.as_str()) {
            Ok(_) => {}
            Err(e) => {
                info!(
                    "棒読みちゃんが起動していないか、エラーが発生しました: {}",
                    e
                );
                // is_voice = false;
            }
        };
    }

    // println!("result: {}", result);

    // マークダウン整形
    let markdown_content = convert_markdown_to_html(result.as_str())?;

    // レスポンスをメッセージ履歴に保存し
    // メッセージ履歴を表示
    // Streamでは取れないトークン数を計算する
    let new_response = Message {
        role: ChatGPTRole::Assistant.to_string(),
        content: result.to_string(),
    };

    // メッセージを処理して連結
    let all_messages =
        process_and_concat_messages(new_response).expect("Failed to process and concat messages");

    // println!("markdown_content: {}", markdown_content);

    // トークン数・実行時間を算出し、整形する
    Ok(create_response(
        markdown_content.as_str(),
        set_model,
        all_messages.as_str(),
        start,
    ))
}

// #[tauri::command]
// async fn gpt_stream_request(b: u8, msg: &str) -> std::result::Result<String, String> {
//     // タイムスタンプを取得
//     let start = Local::now();
//     // 環境変数からAPIキーを取得
//     let apikey = match env::var("CHATGPTTOKEN") {
//         Ok(val) => val,
//         Err(e) => format!("couldn't interpret CHATGPTTOKEN: {}", e).to_string(),
//     };

//     // create client with APIKEY
//     let client = OpenAI::new(&OpenAI {
//         api_key: apikey,
//         org_id: None,
//     });

//     let mut set_model: &str = "gpt-3.5-turbo-0125";
//     if b == 1 {
//         // set_model = "gpt-4-0125-preview";
//         set_model = "gpt-4o";
//     }

//     // メッセージ履歴に保存する
//     // グローバル変数のロックを短くするため、リクエストをはさみ二度アクセスしている
//     let messages: Vec<Message> = {
//         let mut guard_messages: std::sync::MutexGuard<'_, Vec<Message>> = MESSAGES.lock().map_err(
//             |err: std::sync::PoisonError<std::sync::MutexGuard<'_, Vec<Message>>>| {
//                 format!("lazy struct data lock error: {}", err)
//             },
//         )?;
//         guard_messages.push(Message {
//             role: ChatGPTRole::User.to_string(),
//             content: msg.to_string(),
//         });

//         guard_messages.clone()
//     };

//     // 履歴を渡すために、ChatCompletionMessageに変換します
//     let pass_vec: Vec<ChatCompletionMessage> = messages
//         .iter()
//         .map(|message| {
//             ChatCompletionMessageRequestBuilder::default()
//                 .role(ChatGPTRole::User)
//                 .content(message.content.clone())
//                 .name(message.role.clone())
//                 .build()
//                 .unwrap()
//         })
//         .collect();

//     // リクエストボディを作成
//     let req = match CreateChatRequestBuilder::default()
//         .model(set_model.to_string())
//         .stream(true)
//         .messages(pass_vec)
//         .build()
//     {
//         Ok(req) => req,
//         Err(e) => return Err(format!("CreateChatRequestBuilder error: {}", e)),
//     };

//     // リクエストを送信
//     let mut stream = match client.chat().create_with_stream(&req).await {
//         Ok(stream) => stream,
//         Err(e) => return Err(format!("client.chat().create_with_stream error: {}", e)),
//     };

//     // VoiceIDの指定を読み込み
//     let mut is_voice: bool;
//     let voice_id: i16 = match env::var("VOICEID") {
//         Ok(val) => {
//             is_voice = true;
//             info!("VOICEID: {}", val);
//             val.parse().unwrap()
//         }
//         Err(e) => {
//             info!("couldn't interpret VOICEID: {}", e);
//             is_voice = false;
//             1
//         }
//     };

//     let mut result = String::new();
//     let mut delta = String::new();
//     let mut reason = String::new();
//     while let Some(res) = stream.next().await {
//         let response = match res {
//             Ok(response) => response,
//             Err(e) => {
//                 return Err(format!("stream.next().await error: {}", e));
//             }
//         };

//         response.choices.iter().for_each(|choice| {
//             if let Some(ref content) = choice.delta.content {
//                 delta.push_str(content);
//             }
//             // 終了理由を取得する
//             reason = match &choice.finish_reason {
//                 Some(reason) => reason.clone(),
//                 None => String::new(),
//             };
//         });

//         // Stop文字を定義し、中途処理を行います
//         if delta.ends_with('.')
//             || delta.ends_with('。')
//             || delta.ends_with('\n')
//             || !reason.is_empty()
//         {
//             result.push_str(&delta);
//             // メッセージを発言
//             // 棒読みちゃんが起動していない場合は無視します
//             if is_voice {
//                 match mods::voice::say(voice_id, delta.as_str()) {
//                     Ok(_) => {}
//                     Err(e) => {
//                         info!(
//                             "棒読みちゃんが起動していないか、エラーが発生しました: {}",
//                             e
//                         );
//                         is_voice = false;
//                     }
//                 };
//             }
//             // デルタ文字列を初期化
//             delta = String::new();
//         }
//     }

//     // マークダウン整形
//     let markdown_content = convert_markdown_to_html(result.as_str())?;

//     // レスポンスをメッセージ履歴に保存し
//     // メッセージ履歴を表示
//     // Streamでは取れないトークン数を計算する
//     let new_response = Message {
//         role: ChatGPTRole::Assistant.to_string(),
//         content: result.to_string(),
//     };
//     // メッセージを処理して連結
//     let all_messages =
//         process_and_concat_messages(new_response).expect("Failed to process and concat messages");

//     // トークン数・実行時間を算出し、整形する
//     Ok(create_response(
//         markdown_content.as_str(),
//         set_model,
//         all_messages.as_str(),
//         start,
//     ))
// }

#[tauri::command]
async fn gpt_request(b: u8, msg: &str) -> std::result::Result<String, String> {
    let start = Local::now();
    let apikey = match env::var("CHATGPTTOKEN") {
        Ok(val) => val,
        Err(e) => return Err(format!("couldn't interpret CHATGPTTOKEN: {}", e)),
    };

    let client = OClient::new(apikey.to_string());

    let base64_regex = Regex::new(r"data:image/png;base64,[^\s]+").unwrap();
    let mut base64_data = String::new();

    // Extract base64 string from msg
    for cap in base64_regex.captures_iter(msg) {
        base64_data = cap[0].to_string(); // Assuming there is only one base64 encoded string
    }

    let set_model: &str = if b == 1 {
        "gpt-4o"
    } else {
        "gpt-3.5-turbo-0125"
    };

    // Remove base64 string from msg
    let updated_msg = base64_regex.replace_all(msg, "").to_string();

    let messages: Vec<Message> = {
        let mut guard_messages: std::sync::MutexGuard<'_, Vec<Message>> = MESSAGES
            .lock()
            .map_err(|err| format!("lazy struct data lock error: {}", err))?;
        guard_messages.push(Message {
            role: ChatGPTRole::User.to_string(),
            content: updated_msg.to_string(),
        });
        guard_messages.clone()
    };

    // Function to determine the MessageRole
    fn determine_message_role(role: &str) -> chat_completion::MessageRole {
        match role {
            _ if role == ChatGPTRole::User.to_string() => chat_completion::MessageRole::user,
            _ if role == ChatGPTRole::Assistant.to_string() => {
                chat_completion::MessageRole::assistant
            }
            _ => chat_completion::MessageRole::system,
        }
    }

    // Handling image and text content separately for chat_completion messages
    let pass_vec: Vec<chat_completion::ChatCompletionMessage> = messages
        .iter()
        .flat_map(|message| {
            let role = determine_message_role(&message.role);

            if !base64_data.is_empty() {
                if role.clone() == chat_completion::MessageRole::user {
                    vec![chat_completion::ChatCompletionMessage {
                        role: role.clone(),
                        content: chat_completion::Content::ImageUrl(vec![
                            chat_completion::ImageUrl {
                                r#type: chat_completion::ContentType::text,
                                text: Some(message.content.clone()),
                                image_url: None,
                            },
                            chat_completion::ImageUrl {
                                r#type: chat_completion::ContentType::image_url,
                                text: None,
                                image_url: Some(chat_completion::ImageUrlType {
                                    url: base64_data.to_string(),
                                }),
                            },
                        ]),
                        name: None,
                    }]
                } else {
                    vec![chat_completion::ChatCompletionMessage {
                        role: role.clone(),
                        content: chat_completion::Content::Text(message.content.clone()),
                        name: None,
                    }]
                }
            } else {
                vec![chat_completion::ChatCompletionMessage {
                    role: role.clone(),
                    content: chat_completion::Content::Text(message.content.clone()),
                    name: None,
                }]
            }
        })
        .collect();

    let req = ChatCompletionRequest::new(set_model.to_string(), pass_vec);
    req.clone().stream(true);

    // // リクエストの確認、プリント
    // println!("{:?}\n", req);

    let stream = match client.chat_completion(req) {
        Ok(stream) => stream,
        Err(e) => return Err(format!("client.chat_completion error: {}", e)),
    };

    // Option(string)から、stringに変換
    let result: String = match stream.choices[0].message.content {
        Some(ref content) => content.clone(),
        None => String::new(),
    };

    // VoiceIDの指定を読み込み
    let is_voice: bool;
    let voice_id: i16 = match env::var("VOICEID") {
        Ok(val) => {
            is_voice = true;
            info!("VOICEID: {}", val);
            val.parse().unwrap()
        }
        Err(e) => {
            info!("couldn't interpret VOICEID: {}", e);
            is_voice = false;
            1
        }
    };
    // メッセージを発言
    // 棒読みちゃんが起動していない場合は無視します
    if is_voice {
        match mods::voice::say(voice_id, result.as_str()) {
            Ok(_) => {}
            Err(e) => {
                info!(
                    "棒読みちゃんが起動していないか、エラーが発生しました: {}",
                    e
                );
                // is_voice = false;
            }
        };
    }

    // println!("result: {}", result);

    // マークダウン整形
    let markdown_content = convert_markdown_to_html(result.as_str())?;

    // レスポンスをメッセージ履歴に保存し
    // メッセージ履歴を表示
    // Streamでは取れないトークン数を計算する
    let new_response = Message {
        role: ChatGPTRole::Assistant.to_string(),
        content: result.to_string(),
    };

    // メッセージを処理して連結
    let all_messages =
        process_and_concat_messages(new_response).expect("Failed to process and concat messages");

    // println!("markdown_content: {}", markdown_content);

    // トークン数・実行時間を算出し、整形する
    Ok(create_response(
        markdown_content.as_str(),
        set_model,
        all_messages.as_str(),
        start,
    ))
}

fn convert_markdown_to_html(markdown_text: &str) -> Result<String, String> {
    markdown::to_html_with_options(markdown_text, &markdown::Options::gfm())
        .map_err(|e| format!("markdown::to_html_with_options error: {}", e))
}

fn process_and_concat_messages(new_message: Message) -> Result<String, String> {
    match MESSAGES.lock() {
        Ok(mut guard_messages) => {
            guard_messages.push(new_message);
            let concat_messages = guard_messages
                .iter()
                .map(|message| message.content.as_str())
                .collect::<String>();
            Ok(concat_messages)
        }
        Err(e) => Err(format!("lazy struct data lock error: {}", e)),
    }
}

fn create_response(
    markdown_content: &str,
    set_model: &str,
    tokenize_resource: &str,
    start: chrono::DateTime<chrono::Local>,
) -> String {
    let end = Local::now();
    let bpe = cl100k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(tokenize_resource);
    let msg = format!(
        "{}\n\nModel: {}, Total token: {}, Elaps: {}s",
        markdown_content,
        set_model,
        tokens.len(),
        end.signed_duration_since(start).num_seconds(),
    );
    msg
}

#[tauri::command]
fn reset_messages() {
    match MESSAGES
        .lock()
        .map_err(|err| format!("lazy struct data lock error: {}", err))
    {
        Ok(mut guard_message) => {
            guard_message.clear();
        }
        Err(e) => error!("lazy struct data lock error: {}", e),
    }
    info!("reset_messages is success");
}

#[tauri::command]
fn request_system(num: u8) -> std::result::Result<String, String> {
    let order: String = mods::prompts::choose(num);

    // メッセージ履歴にrole: systemを差し込む
    match MESSAGES
        .lock()
        .map_err(|err| format!("lazy struct data lock error: {}", err))
    {
        Ok(mut guard_messages) => {
            match order.as_str() {
                // normalまたはunknownのメッセージを削除します
                "normal" => {
                    guard_messages.retain(|message| message.content != "unknown");
                }
                _ => {
                    guard_messages.push(Message {
                        role: ChatGPTRole::System.to_string(),
                        content: order.clone(),
                    });
                }
            }
            Ok(format!("system message: {}", order))
        }
        Err(e) => Err(format!("lazy struct data lock error: {}", e)),
    }
}

#[tauri::command]
fn is_there_env() -> bool {
    let chatgpt = env::var("CHATGPTTOKEN");
    let anth = env::var("ANTHROPIC_API_KEY");
    // 上記環境変数一方が存在すればTrue, どちらもなければFalse
    chatgpt.is_ok() || anth.is_ok()
}

fn main() {
    dotenv().ok();
    env_logger::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            is_there_env,
            gpt_request,
            reset_messages,
            request_system,
            claude_request,
            memo
        ])
        .on_window_event(move |event| {
            if let tauri::WindowEvent::Destroyed = event.event() {
                // ウィンドウイベントを監視
                // ウィンドウ終了時に履歴をメモします
                info!("Window destroyed");
                memo();
                let _ = event.window().close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
