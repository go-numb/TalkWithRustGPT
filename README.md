# Talk with RustGPT
API: Rust only
GUI: Tauri + React(Typescript).  
The attached image will only be accepted as a screenshot paste.

Switch API & Models:  
API: OpenAI ChatGPT with image
- hight: 4o
- low: 4o-mini
API: Anthropic Claude with image
- hight: 3.5 sonnet
- low: 3 opus

[Release](https://github.com/go-numb/TalkWithRustGPT/releases)  

![TalkWithRustGPT](https://github.com/go-numb/TalkWithRustGPT/blob/images/public/talkwithgpt.png)

## Future
- [x] switch_request(gpt_request, claude_request)
- [x] switch model
- [x] reset_messages -> memo, window close -> memo.
- [x] input voice, output voice
- [x] voice commands ["教えて", "送信", "エンター"]

## Required
set env CHATGPTTOKEN  
set env ANTHROPIC_API_KEY  
// If you specify the voice_id of the 棒読みちゃん, she will speak.  
set env VOICEID

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