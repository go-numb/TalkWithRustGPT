# Tauri コマンドリファレンス

フロントエンドから `invoke()` で呼び出せる Rust コマンドの一覧です。

---

## `is_there_env`

環境変数が設定されているか確認します。

```typescript
invoke("is_there_env") => boolean
```

- `CHATGPTTOKEN` または `ANTHROPIC_API_KEY` のいずれかが存在すれば `true`
- どちらも未設定の場合は `false`
- アプリ起動時に呼ばれ、未設定の場合は警告を表示

---

## `claude_request`

Claude API にリクエストを送信します。

```typescript
invoke("claude_request", { b: number, msg: string, src: string }) => Promise<string>
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `b` | `u8` | モデル選択。`1` = 高性能、`0` = 経済的 |
| `msg` | `string` | ユーザーのメッセージ |
| `src` | `string` | 画像の Base64 データ URL (なければ空文字) |

**戻り値**: マークダウン変換済み HTML + モデル情報

---

## `chatgpt_request`

ChatGPT API にリクエストを送信します。

```typescript
invoke("chatgpt_request", { b: number, msg: string, src: string }) => Promise<string>
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `b` | `u8` | モデル選択。`1` = 高性能 (`chatgpt-4o`)、`0` = 経済的 (`gpt-4o-mini`) |
| `msg` | `string` | ユーザーのメッセージ |
| `src` | `string` | 画像の Base64 データ URL (なければ空文字) |

**戻り値**: マークダウン変換済み HTML + モデル情報

---

## `chatgpt_request_to_dell3`

DALL-E 3 で画像を生成します。

```typescript
invoke("chatgpt_request_to_dell3", { size: number, msg: string }) => Promise<string>
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `size` | `u8` | 画像サイズ。`1` = 1024×1024、`2` = 1792×1024、`3` = 1024×1792 |
| `msg` | `string` | 画像生成プロンプト |

**戻り値**: JSON 文字列 `{ "prompt": "<修正プロンプト>", "url": "<画像URL>" }`

---

## `gemini_request`

Gemini API にリクエストを送信します。

```typescript
invoke("gemini_request", { b: number, msg: string, src: string }) => Promise<string>
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `b` | `u8` | モデル選択。`1` = 高性能、`0` = 経済的 |
| `msg` | `string` | ユーザーのメッセージ |
| `src` | `string` | 画像の Base64 データ URL (なければ空文字) |

**戻り値**: マークダウン変換済み HTML + モデル情報

---

## `memo`

現在の会話履歴をファイルに保存します。

```typescript
invoke("memo") => Promise<string>
```

**戻り値**:
- `"memo is success"` - 保存成功
- `"no messages"` - 保存するメッセージなし
- `"memo error: <エラー内容>"` - エラー発生時

---

## `reset`

会話履歴 (通常メッセージ + システムプロンプト) をリセットします。

```typescript
invoke("reset") => Promise<string>
```

**戻り値**:
- `"success reset messages"` - リセット成功
- `"messages reset error: <エラー内容>"` - エラー発生時

---

## `request_system`

システムプロンプトを設定します。

```typescript
invoke("request_system", { num: number }) => Promise<string>
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `num` | `u8` | プロンプト番号 (下表参照) |

| 番号 | プロンプトの内容 |
|------|---------------|
| `0` | none (クリア) |
| `1` | 厳格で正確な回答 |
| `2` | 親しみやすく友好的な |
| `3` | 肯定的な |
| `4` | 批判的な視点 |

**戻り値**: `"success"`

---

## `all_messages`

全会話履歴を文字列で返します。

```typescript
invoke("all_messages", { is_raw: boolean }) => Promise<string>
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `is_raw` | `boolean` | `true` = 改行を `<br>` に変換のみ、`false` = マークダウン→HTML 変換 |

**戻り値**: 会話履歴の文字列 (HTML)

**エラー**: `"no message history"` (履歴が空の場合)

---

## `files_to_string`

ファイルパスの配列を受け取り、各ファイルの内容をコードブロック形式でまとめて返します。

```typescript
invoke("files_to_string", { filepaths: string[] }) => Promise<string>
```

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `filepaths` | `string[]` | ファイルパスの配列 |

**戻り値**: 各ファイルを結合した文字列

```
/path/to/file.rs
```rs
// ファイル内容
\```

/path/to/file2.txt
```txt
// ファイル内容
\```
```

- ディレクトリや存在しないパスはスキップ
- すべてスキップされた場合は `"no files"` を返す
