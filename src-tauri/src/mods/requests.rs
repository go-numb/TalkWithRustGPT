use rs_openai::chat::{ChatCompletionMessageRequestBuilder, CreateChatRequestBuilder, Role};
use rs_openai::OpenAI;

use std::env;

use futures::StreamExt;

#[tokio::main]
#[allow(dead_code)]
async fn gpt_request() -> Result<(), Box<dyn std::error::Error>> {
    // 環境変数からAPIキーを取得
    let apikey = match env::var("CHATGPTTOKEN") {
        Ok(val) => val,
        Err(e) => format!("couldn't interpret CHATGPTTOKEN: {}", e).to_string(),
    };

    println!("apikey: {}", apikey);

    let client = OpenAI::new(&OpenAI {
        api_key: apikey,
        org_id: None,
    });

    let req = CreateChatRequestBuilder::default()
        .model("gpt-3.5-turbo-1106")
        // .stream(true)
        .messages(vec![ChatCompletionMessageRequestBuilder::default()
            .role(Role::User)
            .content("Hello, I'm a human.")
            .name("Human".to_string())
            .build()?])
        .build()?;

    println!("request body: {:?}", req);

    let res = client.chat().create(&req).await?;
    println!("response body: {:?}", res.choices);

    Ok(())
}

#[tokio::main]
#[allow(dead_code)]
async fn gpt_stream_request() -> Result<String, Box<dyn std::error::Error>> {
    // 環境変数からAPIキーを取得
    let apikey = match env::var("CHATGPTTOKEN") {
        Ok(val) => val,
        Err(e) => format!("couldn't interpret CHATGPTTOKEN: {}", e).to_string(),
    };

    println!("apikey: {}", apikey);

    let client = OpenAI::new(&OpenAI {
        api_key: apikey,
        org_id: None,
    });

    let req = CreateChatRequestBuilder::default()
        .model("gpt-3.5-turbo-1106")
        .stream(true)
        .messages(vec![ChatCompletionMessageRequestBuilder::default()
            .role(Role::User)
            .content("400文字以上の長い小噺を提供してください。創作でも構いません。")
            .name("Human")
            .build()?])
        .build()?;

    println!("request body: {:?}", req);

    let mut stream = client.chat().create_with_stream(&req).await?;

    let mut result = String::new();
    let mut delta = String::new();
    while let Some(response) = stream.next().await {
        response.unwrap().choices.iter().for_each(|choice| {
            if let Some(ref content) = choice.delta.content {
                delta.push_str(content);
            }
        });

        if delta.ends_with('.') || delta.ends_with('。') || delta.ends_with('\n') {
            result.push_str(&delta);
            print!("{}", &delta);
            delta = String::new();
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpt_request() {
        match gpt_stream_request() {
            Ok(_) => println!("success"),
            Err(e) => println!("error: {}", e),
        };
    }

    #[test]
    fn test_gpt_stream_request() {
        match gpt_stream_request() {
            Ok(s) => println!("success, {}", s),
            Err(e) => println!("error: {}", e),
        };
    }
}
