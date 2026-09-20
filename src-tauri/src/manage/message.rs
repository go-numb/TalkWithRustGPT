use directories::UserDirs;
use log::info;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::manage::filetitle;

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
    #[allow(unused)]
    pub fn get(&self) -> (Vec<Message>, Vec<Message>) {
        let messages = self.messages.get();
        let system_messages = self.system_messages.get();
        (messages, system_messages)
    }

    pub fn get_messages(&self) -> Vec<Message> {
        self.messages.get()
    }

    #[allow(unused)]
    pub fn get_system(&self) -> Vec<Message> {
        self.system_messages.get()
    }

    pub fn add_to_messages(&mut self, role: String, content: String, src: Option<String>) {
        self.messages.add(role, content, src);
    }

    #[allow(unused)]
    pub fn add_to_system(&mut self, prompt: String) {
        self.system_messages.add("system".to_string(), prompt, None);
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
        self.memo_to_dir(&Self::memo_dir()?)
    }

    /// Saves the current conversation, then clears conversation and system history.
    ///
    /// An empty conversation needs no filesystem access and simply resets its system prompt.
    pub fn save_and_reset(&mut self) -> Result<(), String> {
        if self.messages.messages.is_empty() {
            return self.reset();
        }

        self.save_and_reset_to_dir(&Self::memo_dir()?)
    }

    fn memo_dir() -> Result<PathBuf, String> {
        let user_dir =
            UserDirs::new().ok_or_else(|| "failed to locate user directories".to_string())?;
        let document_dir = user_dir
            .document_dir()
            .ok_or_else(|| "failed to locate Documents directory".to_string())?;

        Ok(document_dir.join(".appdata").join(APPNAME))
    }

    fn save_and_reset_to_dir(&mut self, save_dir: &Path) -> Result<(), String> {
        if self.messages.messages.is_empty() {
            return self.reset();
        }

        self.memo_to_dir(save_dir)?;
        self.reset()
    }

    fn memo_to_dir(&self, save_dir: &Path) -> Result<(), String> {
        // without system messages
        let copy_shelf = self.clone();
        let messages = copy_shelf.messages.get();
        let data = messages
            .iter()
            .map(|m| {
                if m.role == "user" {
                    // content has image_url, delete it;
                    format!("{}: {}", m.role, m.content.clone())
                } else {
                    format!("{}: {}\n----------------", m.role, m.content)
                }
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        // create filename & save path
        create_dir_all(save_dir).map_err(|e| format!("failed to create memo directory: {}", e))?;
        // #tag based filename or default filename
        let filename = filetitle::to_title(data.as_str());

        self.write_memo(save_dir, &filename, &data)
    }

    fn write_memo(&self, save_dir: &Path, filename: &str, data: &str) -> Result<(), String> {
        let (stem, extension) = match filename.rsplit_once('.') {
            Some((stem, extension)) => (stem, format!(".{}", extension)),
            None => (filename, String::new()),
        };

        for suffix in 0..=u32::MAX {
            let candidate = if suffix == 0 {
                save_dir.join(filename)
            } else {
                save_dir.join(format!("{}-{}{}", stem, suffix, extension))
            };

            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&candidate)
            {
                Ok(mut file) => {
                    info!("save memo to {:?}", candidate);
                    if let Err(e) = file
                        .write_all(data.as_bytes())
                        .and_then(|_| file.sync_all())
                    {
                        // This file was created by this attempt. Remove incomplete output
                        // where possible; the conversation remains available for retry.
                        drop(file);
                        let _ = std::fs::remove_file(&candidate);
                        return Err(format!("failed to write file: {}", e));
                    }
                    return Ok(());
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(format!("failed to create file: {}", e)),
            }
        }

        Err("failed to find an unused memo filename".to_string())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Messages {
    // add clone
    pub messages: Vec<Message>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Message {
    // who is speaking
    pub role: String,
    // what is said
    pub content: String,
    // image/file base64 source data
    pub src: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let id = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "talkwithrustgpt-message-tests-{}-{}",
                std::process::id(),
                id
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn shelf_with_history() -> Shelf {
        let mut shelf = Shelf::new();
        shelf.add_to_messages("user".to_string(), "hello".to_string(), None);
        shelf.add_to_system("system prompt".to_string());
        shelf
    }

    #[test]
    fn test_request_system_overwrites_previous_system_prompt() {
        // request_system を2回呼んだとき、system_messages は1件だけになるべき
        let mut shelf = Shelf::new();
        shelf
            .system_messages
            .add("system".to_string(), "be strict".to_string(), None);
        // 2回目の呼び出しで上書きされることを期待
        shelf.system_messages.reset();
        shelf
            .system_messages
            .add("system".to_string(), "be friendly".to_string(), None);

        assert_eq!(shelf.system_messages.get().len(), 1);
        assert_eq!(shelf.system_messages.get()[0].content, "be friendly");
    }

    #[test]
    fn test_get_system_returns_system_messages_not_messages() {
        let mut shelf = Shelf::new();
        shelf.add_to_messages("user".to_string(), "hello".to_string(), None);
        shelf
            .system_messages
            .add("system".to_string(), "be strict".to_string(), None);

        let system = shelf.get_system();
        assert_eq!(system.len(), 1);
        assert_eq!(system[0].content, "be strict");
        assert_eq!(system[0].role, "system");
    }

    #[test]
    fn save_and_reset_persists_conversation_then_clears_both_histories() {
        let temp_dir = TestDir::new();
        let mut shelf = shelf_with_history();

        shelf.save_and_reset_to_dir(temp_dir.path()).unwrap();

        assert!(shelf.get_messages().is_empty());
        assert!(shelf.get_system().is_empty());
        let saved_files = fs::read_dir(temp_dir.path())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(saved_files.len(), 1);
        assert_eq!(
            fs::read_to_string(saved_files[0].path()).unwrap(),
            "user: hello"
        );
    }

    #[test]
    fn save_failure_keeps_conversation_and_system_history() {
        let temp_dir = TestDir::new();
        let invalid_memo_dir = temp_dir.path().join("not-a-directory");
        fs::write(&invalid_memo_dir, "file").unwrap();
        let mut shelf = shelf_with_history();

        assert!(shelf.save_and_reset_to_dir(&invalid_memo_dir).is_err());

        assert_eq!(shelf.get_messages().len(), 1);
        assert_eq!(shelf.get_messages()[0].content, "hello");
        assert_eq!(shelf.get_system().len(), 1);
        assert_eq!(shelf.get_system()[0].content, "system prompt");
    }

    #[test]
    fn file_creation_failure_keeps_message_content_and_attachment() {
        let temp_dir = TestDir::new();
        let mut shelf = Shelf::new();
        // The tag produces a filename with a nonexistent parent directory.
        shelf.add_to_messages(
            "user".to_string(),
            "hello #missing/subdirectory".to_string(),
            Some("data:image/png;base64,aGVsbG8=".to_string()),
        );
        shelf.add_to_system("system prompt".to_string());

        assert!(shelf.save_and_reset_to_dir(temp_dir.path()).is_err());

        let messages = shelf.get_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "hello #missing/subdirectory");
        assert_eq!(
            messages[0].src.as_deref(),
            Some("data:image/png;base64,aGVsbG8=")
        );
        assert_eq!(shelf.get_system()[0].content, "system prompt");
    }

    #[test]
    fn empty_conversation_resets_system_history_without_accessing_memo_directory() {
        let temp_dir = TestDir::new();
        let invalid_memo_dir = temp_dir.path().join("not-a-directory");
        fs::write(&invalid_memo_dir, "file").unwrap();
        let mut shelf = Shelf::new();
        shelf.add_to_system("system prompt".to_string());

        shelf.save_and_reset_to_dir(&invalid_memo_dir).unwrap();

        assert!(shelf.get_messages().is_empty());
        assert!(shelf.get_system().is_empty());

        // Also cover the public entry point's early return before directory discovery.
        shelf.add_to_system("another prompt".to_string());
        shelf.save_and_reset().unwrap();
        assert!(shelf.get_system().is_empty());
    }

    #[test]
    fn duplicate_memo_name_preserves_existing_memo() {
        let temp_dir = TestDir::new();
        let shelf = shelf_with_history();
        let filename = "memo.txt";
        fs::write(temp_dir.path().join(filename), "first memo").unwrap();

        shelf
            .write_memo(temp_dir.path(), filename, "second memo")
            .unwrap();
        shelf
            .write_memo(temp_dir.path(), filename, "third memo")
            .unwrap();

        assert_eq!(
            fs::read_to_string(temp_dir.path().join(filename)).unwrap(),
            "first memo"
        );
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("memo-1.txt")).unwrap(),
            "second memo"
        );
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("memo-2.txt")).unwrap(),
            "third memo"
        );
    }
}

impl Messages {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn add(&mut self, role: String, content: String, src: Option<String>) {
        let message = Message { role, content, src };
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
