use directories::UserDirs;
use serde_json::Value;
use std::io::Write;
use std::sync::{Arc, Mutex};

use std::fs::File;

use std::result::Result;

use log::info;

use std::fs::create_dir_all;

const APPNAME: &str = "Talk with RustGPT";

#[derive(Debug, Clone)]
pub struct MsgBox {
    pub messages: Messages,
    pub system_messages: Messages,
}

impl MsgBox {
    pub fn new() -> Self {
        Self {
            messages: Messages::new(),
            system_messages: Messages::new(),
        }
    }

    pub fn get(self) -> (Vec<Message>, Vec<Message>) {
        let messages = self.messages.get();
        let system_messages = self.system_messages.get();
        (messages, system_messages)
    }

    pub fn reset(self) {
        self.messages.reset();
        self.system_messages.reset();
    }

    pub fn memo(self) -> Result<(), String> {
        // save to dir
        let user_dir: UserDirs = UserDirs::new().unwrap();
        let document_dir = user_dir.document_dir().unwrap();
        let save_dir = document_dir.join(".appdata").join(APPNAME);

        create_dir_all(save_dir.as_path()).unwrap();
        let date = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let filename = format!("memo-{}.txt", date);

        let full_path = save_dir.join(filename.as_str());
        info!("save memo to {:?}", full_path);

        // without system messages
        let messages = self.messages.get();
        let data = messages
            .iter()
            .map(|m| {
                if m.role == "user" {
                    format!("{}: {}", m.role, m.content)
                } else {
                    format!("{}: {}\n----------------", m.role, m.content)
                }
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        let mut file = match File::create(full_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(format!("failed to create file: {}", e));
            }
        };

        let result = match file.write_all(data.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("failed to write file: {}", e)),
        };

        result
    }
}

#[derive(Debug, Clone)]
pub struct Messages {
    pub messages: Arc<Mutex<Vec<Message>>>,
}
#[derive(Debug, Clone)]
pub struct Message {
    role: String,
    content: Value,
}

impl Messages {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add(self, role: String, content: Value) {
        let mut messages = self.messages.lock().unwrap();
        let message = Message { role, content };
        messages.push(message);
    }

    pub fn get(self) -> Vec<Message> {
        let messages = self.messages.lock().unwrap();
        messages.clone()
    }

    // reset messages
    pub fn reset(self) {
        let mut messages = self.messages.lock().unwrap();
        messages.clear();
    }
}
