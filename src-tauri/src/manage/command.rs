use std::{error::Error, path::Prefix};

use tauri::command;

// prefix for command
const PREFIX: &str = "/";

#[derive(Debug, PartialEq, Eq)]
pub enum TypeCommand {
    AllMessages, // すべてのメッセージ履歴を取得
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandError {
    NotFound,
    InvalidFormat,
    // 他のエラー型も必要に応じて追加可能
}

impl TypeCommand {
    pub fn new() -> Self {
        TypeCommand::AllMessages
    }

    pub fn vars() -> Vec<String> {
        ["all"]
            .iter()
            .map(|x| format!("{}{}", PREFIX, x))
            .collect::<Vec<String>>()
    }
}

pub fn to_command(s: &str) -> TypeCommand {
    // remove prefix
    let command = s.trim_start_matches(PREFIX);

    match command {
        "all" => TypeCommand::AllMessages,
        _ => TypeCommand::AllMessages,
    }
}

// 独自コマンドを文字列から抽出する
pub fn find_command(s: &str) -> Result<TypeCommand, CommandError> {
    let commands = TypeCommand::vars();

    // 設定されたコマンドが文字列に含まれている場合は、そのコマンドを返す
    for command in commands {
        if s.contains(command.as_str()) {
            return Ok(to_command(command.as_str()));
        }
    }

    Err(CommandError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_command() {
        let command = find_command("test command text sample/all");
        assert!(command.is_ok());
        assert_eq!(command.unwrap(), TypeCommand::AllMessages);

        let command = find_command("text sample /test");
        assert_eq!(command.err(), Some(CommandError::NotFound));
    }
}
