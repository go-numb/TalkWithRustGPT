# Talk with RustGPT
API: Rust only
GUI: Tauri + React(Typescript).  
The attached image will only be accepted as a screenshot paste.

Model and AI can be used across the board. Supports launching in multiple windows.

Switch API & Models:  

API: OpenAI ChatGPT with image
- hight: 4o
- low: 4o-mini  

API: Anthropic Claude with image
- hight: 3.7 sonnet
- low: 3.5 haiku

API: Google Gemini with image
- hight: 2.0 flex
- low: 2.0 flex

※ you can use/change models by enviroment vars.

[Release](https://github.com/go-numb/TalkWithRustGPT/releases)  

![TalkWithRustGPT](https://github.com/go-numb/TalkWithRustGPT/blob/images/public/talkwithgpt.png)

## Future
- [x] switch_request(gpt_request, claude_request)
- [x] switch model
- [x] reset_messages -> memo, window close -> memo.
- [x] input voice, output voice
- [x] voice commands ["教えて", "送信", "エンター"]
- [x] get command for all messages
- [x] command matome & save(use filename from first #tag)
- [x] textfile drug and drop, insert message.
- [x] read AI answers aloud with Gemini TTS (female / male voice)
- [x] settings tab (TTS model, voices, image resize limit)

## Required
set env CHATGPTTOKEN  
set env ANTHROPIC_API_KEY  
set env GOOGLE_GEMINI_API_KEY  
// If you specify the voice_id of the 棒読みちゃん, she will speak.  
set env VOICEID

// Read-aloud button (🔊) uses Gemini TTS with GOOGLE_GEMINI_API_KEY.
// Model and voices are chosen in the Settings tab; these env vars only set its defaults.
// Options :: default gemini-3.8-flash-tts / Erinome / Charon (multilingual, reads the original language)
set env GEMINI_TTS_MODEL gemini-3.8-flash-tts
set env GEMINI_TTS_VOICE_FEMALE Erinome
set env GEMINI_TTS_VOICE_MALE Charon

// Options :: use model each ai
set env CHATGPT_MODELS gpt-4o.gpt-4o-mini
set env CLAUDE_MODELS claude-3-5-sonnet-20240620.claude-3-opus-20240229
set env GEMINI_MODELS gemini-1.5-pro-002.gemini-1.5-flash-002



## Usage
```rust
$ cargo tauri dev

// or 

$ cargo tauri build
```

```js
$ npm tauri dev

// or 

$ npm tauri build
```

## Build Windows .exe from WSL (Docker)
WSL 側の環境を汚さずに、Docker コンテナ内で Windows 向け `.exe`（GNU ターゲット）を生成します。

### Required
- WSL から `docker` コマンドが使えること（Docker Desktop + WSL integration 等）

### Run
```bash
chmod +x scripts/build-win-exe.sh
./scripts/build-win-exe.sh
```

### Output
- `artifacts/windows/*.exe`
- 実行に必要な `*.dll`
- 改ざん・転送確認用の `artifacts/windows/SHA256SUMS`

ビルド前に出力ディレクトリ内の古いファイルを削除し、必要なランタイム DLL が
見つからない場合は成功扱いにしません。Docker の DNS は既定で ExpressVPN の
トンネルゲートウェイ (`100.64.100.1`) を使い、
必要に応じて `DOCKER_DNS` 環境変数で変更できます。
依存パッケージとRustのビルド成果物はDocker named volumeへキャッシュするため、
2回目以降は差分のみを再ビルドします。

※ MSI/NSIS などの Windows インストーラ生成は、基本的に Windows 環境での `cargo tauri build` が必要です。



## via golang client
[talk with GPT @go-numb](https://github.com/go-numb/TalkWithGPT)

## Author

[@_numbP](https://twitter.com/_numbP)

## License

[MIT](https://github.com/go-numb/TalkWithRustGPT/blob/master/LICENSE)
