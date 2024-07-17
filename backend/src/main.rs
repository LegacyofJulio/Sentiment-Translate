use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use rust_bert::pipelines::{
    sentiment::SentimentModel,
    translation::{Language, TranslationModel, TranslationModelBuilder},
};
use serde::{self, Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
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
    let response = analyze_sentiments(payload.input).await;
    let response = Response { response };
    (StatusCode::CREATED, Json(response))
}
async fn analyze_sentiments(input: String) -> String {
    tokio::task::spawn_blocking(move || {
        let sentiment_classifier = SentimentModel::new(Default::default()).unwrap();
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
        let input = [res.trim()];
        let output = sentiment_classifier.predict(&input);
        format!("Translated input:{}\nresults: {:?}", res, output)
    })
    .await
    .unwrap()
}
