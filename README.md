# Talk with RustGPT
GUI: Tauri + React + Typescript

Switch API & Models:  
API: OpenAI ChatGPT
API: Anthropic Claude

![TalkWithRustGPT](https://github.com/go-numb/TalkWithRustGPT/blob/images/public/talkwithgpt.png)

## Future
- Switch ChatGPT model
- Input Voice
- Save Conversation history to UserDir

## Required
set env CHATGPTTOKEN
set env ANTHROPIC_API_KEY

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

## tauri::commands for ivoke
- [x] gpt_request
- [x] reset_messages
- [x] memo


## via golang client
[talk with GPT @go-numb](https://github.com/go-numb/TalkWithGPT)

## Author

[@_numbP](https://twitter.com/_numbP)

## License

[MIT](https://github.com/go-numb/TalkWithRustGPT/blob/master/LICENSE)