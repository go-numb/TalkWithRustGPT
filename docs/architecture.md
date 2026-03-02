# アーキテクチャ仕様

## 全体構成

```
┌─────────────────────────────────────────────┐
│               フロントエンド (React/TypeScript)  │
│                                               │
│  App.tsx ─── DrugComponent (D&D)             │
│           ─── ImageComponent (画像表示)       │
│           ─── prompts_list (プロンプト定義)   │
└──────────────────┬──────────────────────────┘
                   │ invoke() / listen()
                   │ (Tauri IPC)
┌──────────────────▼──────────────────────────┐
│              バックエンド (Rust / Tauri)       │
│                                               │
│  lib.rs (Tauriコマンド登録・イベント処理)      │
│  ├── manage/                                  │
│  │   ├── chatgpt.rs ──► OpenAI API            │
│  │   ├── claude.rs  ──► Anthropic API         │
│  │   ├── gemini.rs  ──► Google Gemini API     │
│  │   ├── message.rs (Shelf / メッセージ管理)  │
│  │   ├── filetitle.rs (ファイル名生成)        │
│  │   └── utils.rs (共通処理・レスポンス整形)  │
│  └── sub/                                     │
│      ├── prompts.rs (システムプロンプト)       │
│      └── voice.rs  (棒読みちゃん連携)         │
└─────────────────────────────────────────────┘
```

## 状態管理

### Rust 側 (共有状態)

```rust
Arc<Mutex<Shelf>>
```

`Shelf` 構造体がグローバルステートとして Tauri の `manage()` で登録されます。

```
Shelf
├── messages: Messages          // 会話履歴 (user / assistant)
└── system_messages: Messages   // システムプロンプト履歴
```

```
Messages
└── messages: Vec<Message>
    └── Message
        ├── role: String        // "user" | "assistant" | "system"
        ├── content: String     // メッセージ本文
        └── src: Option<String> // 画像の Base64 データ URL
```

### フロントエンド側 (React State)

| 変数 | 型 | 説明 |
|------|-----|------|
| `query` | `string` | 最後に送信したメッセージ (表示用) |
| `result` | `string` | AI からの回答 HTML |
| `model` | `number` | モデル選択 (0=経済, 1=高性能) |
| `AI` | `number` | AI 選択 (0=Claude, 1=ChatGPT, 2=Gemini) |
| `status` | `string` | フッターに表示するステータスメッセージ |
| `imageUrl` | `string \| null` | 送信待ちの画像 Base64 URL |
| `imageUrls` | `string[]` | 送信済みの画像一覧 |
| `isUpload` | `boolean` | 画像送信済みフラグ |
| `resultImageUrl` | `string \| null` | DALL-E 3 の生成画像 URL |

## AI リクエストフロー

```
フロントエンド
  ↓ invoke("claude_request" | "chatgpt_request" | "gemini_request", {b, msg, src})
Rust: XXX_request()
  ↓ Shelf.add_to_messages("user", msg, src)       // ユーザーメッセージを履歴に追加
  ↓ Shelf.get_messages()                           // 履歴取得
  ↓ HTTP POST → 各 AI API エンドポイント
  ↓ レスポンスからテキスト抽出
  ↓ Shelf.add_to_messages("assistant", text, None) // AIレスポンスを履歴に追加
  ↓ utils::say(text)                               // 棒読みちゃんで読み上げ
  ↓ utils::convert_markdown_to_html(text)          // マークダウン変換
  ↓ utils::create_response(...)                    // モデル名・トークン数・経過時間を付加
  ↓ Ok(html_string)
フロントエンド: setResult(html_string)
```

## 各 AI API エンドポイント

### ChatGPT

- **エンドポイント**: `https://api.openai.com/v1/chat/completions`
- **認証**: `Authorization: Bearer <CHATGPTTOKEN>`
- **メッセージ形式**:
  - テキストのみ: `[{"type": "text", "text": "..."}]`
  - 画像付き: `[{"type": "image_url", "image_url": {"url": "<base64>"}}, {"type": "text", "text": "..."}]`

### Claude

- **エンドポイント**: `https://api.anthropic.com/v1/messages`
- **認証**: `x-api-key: <ANTHROPIC_API_KEY>`
- **バージョンヘッダ**: `anthropic-version: 2023-06-01`
- **システムプロンプト**: リクエストボディの `system` フィールドに設定
- **メッセージ形式**:
  - テキストのみ: `[{"type": "text", "text": "..."}]`
  - 画像付き: `[{"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "..."}}, ...]`

### Gemini

- **エンドポイント**: `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key=<GOOGLE_GEMINI_API_KEY>`
- **認証**: URL パラメータの `key`
- **システムプロンプト**: リクエストボディの `systemInstruction.parts[].text` に設定
- **ロール変換**: `assistant` → `model` (Gemini の仕様に合わせて変換)
- **メッセージ形式**:
  - テキストのみ: `[{"text": "..."}]`
  - 画像付き: `[{"text": "..."}, {"inline_data": {"mime_type": "image/png", "data": "..."}}]`

## ファイル保存

会話をメモとして保存する際のフロー:

```
Shelf.memo()
  ↓ UserDirs::new().document_dir()  // ~/Documents を取得
  ↓ .appdata/Talk with RustGPT/     // 保存ディレクトリ (なければ作成)
  ↓ filetitle::to_title(data)       // ファイル名生成
      └── 会話に "#タグ" があれば "memo-日時-タグ.txt"
          なければ "memo-日時.txt"
  ↓ File::create(path).write_all()  // ファイル書き込み
```

## トークン計算

`tiktoken-rs` の `cl100k_base` エンコーダを使用して、全会話履歴のトークン数を計算します。

```rust
let bpe = cl100k_base().unwrap();
let tokens = bpe.encode_with_special_tokens(all_messages + image_src);
```

> cl100k_base は GPT-4 系モデルと同じエンコーダです。Claude / Gemini のトークン数とは若干異なる場合があります。
