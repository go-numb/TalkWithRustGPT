# セットアップ・起動手順

## 前提条件

- Node.js (npm)
- Rust / Cargo
- Tauri CLI v2

## 環境変数の設定

プロジェクトルートまたは `src-tauri/` ディレクトリに `.env` ファイルを作成し、以下を設定します。

```env
# OpenAI API キー (ChatGPT / DALL-E 3)
CHATGPTTOKEN=sk-xxxxxxxxxxxxxxxx

# Anthropic API キー (Claude)
ANTHROPIC_API_KEY=sk-ant-xxxxxxxxxxxxxxxx

# Google Gemini API キー
GOOGLE_GEMINI_API_KEY=xxxxxxxxxxxxxxxx

# 棒読みちゃん ボイスID (省略可。省略時は音声読み上げ無効)
VOICEID=1
```

### 環境変数の詳細

| 変数名 | 必須 | 説明 |
|--------|------|------|
| `CHATGPTTOKEN` | ChatGPT 使用時 | OpenAI の API キー |
| `ANTHROPIC_API_KEY` | Claude 使用時 | Anthropic の API キー |
| `GOOGLE_GEMINI_API_KEY` | Gemini 使用時 | Google の Gemini API キー |
| `VOICEID` | 任意 | 棒読みちゃんのボイス番号。未設定時は音声読み上げをスキップ |

> **注意**: `CHATGPTTOKEN` または `ANTHROPIC_API_KEY` のいずれか一方がないと、起動時にアラートが表示されます。

### モデルのカスタマイズ (省略可)

環境変数でモデルをカスタマイズできます。カンマ区切りで `高性能モデル,経済モデル` の順に指定します。

```env
# ChatGPT モデル (デフォルト: chatgpt-4o,gpt-4o-mini)
CHATGPT_MODELS=gpt-4o,gpt-4o-mini

# Claude モデル (デフォルト: claude-3-7-sonnet,claude-3-5-haiku)
CLAUDE_MODELS=claude-3-7-sonnet-latest,claude-3-5-haiku-latest

# Gemini モデル (デフォルト: gemini-2.0-flash,gemini-2.0-flash)
GEMINI_MODELS=gemini-2.0-flash,gemini-2.0-flash
```

## 開発サーバーの起動

```bash
# 依存関係のインストール
npm install

# 開発モードで起動
npm run tauri dev
```

## ビルド (本番用)

```bash
npm run tauri build
```

ビルド成果物は `src-tauri/target/release/bundle/` に生成されます。

## 棒読みちゃんの設定 (オプション)

音声読み上げ機能を使用する場合:

1. 棒読みちゃんを起動しておく
2. `.env` に `VOICEID` を設定する (例: `VOICEID=1`)
3. アプリケーションを起動すると、AI 応答が自動的に読み上げられます

棒読みちゃんが起動していない場合はエラーを出力しますが、アプリケーションの動作には影響しません。
