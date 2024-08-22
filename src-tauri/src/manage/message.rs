use directories::UserDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;

use std::fs::File;

use std::result::Result;

use log::info;

use std::fs::create_dir_all;

const APPNAME: &str = "Talk with RustGPT";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Shelf {
    pub messages: Messages,
    pub system_messages: Messages,
}

impl Shelf {
    pub fn new() -> Self {
        Self {
            messages: Messages::new(),
            system_messages: Messages::new(),
        }
    }

    pub fn get(&self) -> (Vec<Message>, Vec<Message>) {
        let messages = self.messages.get();
        let system_messages = self.system_messages.get();
        (messages, system_messages)
    }

    pub fn get_messages(&self) -> Vec<Message> {
        self.messages.get()
    }

    pub fn get_system(&self) -> Vec<Message> {
        self.messages.get()
    }

    pub fn add_to_messages(&mut self, role: String, content: Value) {
        self.messages.add(role, content);
    }

    pub fn add_to_system(&mut self, prompt: Value) {
        self.system_messages.add("system".to_string(), prompt);
    }

    pub fn reset(&mut self) -> Result<(), String> {
        self.messages.reset();
        self.system_messages.reset();

        if self.messages.messages.is_empty() {
            println!("success length: {}", self.messages.messages.len());
            Ok(())
        } else {
            println!("failed length: {}", self.messages.messages.len());
            Err("failed to reset messages".to_string())
        }
    }

    pub fn memo(&self) -> Result<(), String> {
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
        let copy_shelf = self.clone();
        let messages = copy_shelf.messages.get();
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Messages {
    // add clone
    pub messages: Vec<Message>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Value,
}

impl Messages {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn add(&mut self, role: String, content: Value) {
        let message = Message { role, content };
        self.messages.push(message);
    }

    pub fn get(&self) -> Vec<Message> {
        self.messages.clone()
    }

    // reset messages
    pub fn reset(&mut self) {
        self.messages.clear();
    }
}
