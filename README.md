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

## Required
set env CHATGPTTOKEN  
set env ANTHROPIC_API_KEY  
set env GOOGLE_GEMINI_API_KEY  
// If you specify the voice_id of the 棒読みちゃん, she will speak.  
set env VOICEID

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



## via golang client
[talk with GPT @go-numb](https://github.com/go-numb/TalkWithGPT)

## Author

[@_numbP](https://twitter.com/_numbP)

## License

[MIT](https://github.com/go-numb/TalkWithRustGPT/blob/master/LICENSE)