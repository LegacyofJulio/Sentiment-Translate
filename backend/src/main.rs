use axum::{
    http::{request, StatusCode},
    routing::{get, post},
    Json, Router,
};
use dotenv_codegen::dotenv;
use rust_bert::pipelines::translation::{Language, TranslationModel, TranslationModelBuilder};
use serde::{self, Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
const URL: &str = "https://integrate.api.nvidia.com/v1/chat/completions";
#[derive(Debug, Serialize, Deserialize)]
struct Input {
    input: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Response {
    response: String,
}
#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/gen", post(gen_response))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn root() -> &'static str {
    "Bienvenido a Sentiment Translate"
}
async fn gen_response(Json(payload): Json<Input>) -> (StatusCode, Json<Response>) {
    let response = parse_data(payload.input).await;
    let response = Response { response };
    (StatusCode::CREATED, Json(response))
}
async fn send_request(input: String) -> String {
    let api_key = dotenv!("API_KEY");
    let client = reqwest::Client::new();
    let prompt = "You are a sentiment analysis chatbot. Your primary function is to analyze the emotional tone and sentiment of sentences provided by users. When given a sentence, you should:

1. Identify the overall sentiment (positive, negative, or neutral)
2. Provide a brief explanation of why you classified it as such
3. Highlight any key words or phrases that influenced your analysis
4. Rate the sentiment intensity on a scale of 1-5, where 1 is very negative and 5 is very positive

If the user provides multiple sentences, analyze each one separately. If asked, you can also provide tips on how to adjust the sentence to change its sentiment.

Remember to be objective in your analysis and avoid personal biases. If a sentence is ambiguous or lacks clear sentiment, state this in your response.

You should not engage in general conversation or perform tasks unrelated to sentiment analysis. Always redirect the user to provide a sentence for analysis if they ask something off-topic.";
    let body = json!({
            "model": "meta/llama3-70b-instruct",
            "messages": [{"role":"system","content":prompt},{"role":"user","content":input}],
            "temperature": 0.5,
            "top_p": 0.7,
            "max_tokens": 1024,
            "stream": false
    });
    let res = client
        .post(URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .unwrap();
    let res = res.text().await.unwrap();
    let json_response: Value = serde_json::from_str(res.as_str()).unwrap();
    json_response["choices"][0]["message"]["content"].to_string()
}
async fn parse_data(input: String) -> String {
    let input = tokio::task::spawn_blocking(move || {
        let model = TranslationModelBuilder::new()
            .with_source_languages(vec![Language::Spanish])
            .with_target_languages(vec![Language::English])
            .create_model()
            .unwrap();
        let output = model
            .translate(&[input.as_str()], None, Language::English)
            .unwrap();
        let mut res: String = String::new();
        for sentence in output {
            res.push_str(&sentence);
        }
        info!("{}", res);
        let input = res;
        input
    })
    .await
    .unwrap();
    let response = send_request(input).await;
    response
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_request() {
        let res = send_request("Hello!".to_string()).await;
        assert!(res.status().is_success());
    }
}
