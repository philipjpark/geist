mod agent;
mod corpus;
mod discovery;
mod ir;
mod massive;
mod news;
mod pipeline;
mod reddit;
mod trends;
mod x_corpus;

use axum::{
    extract::Json,
    http::Method,
    routing::{get, post},
    Router,
};
use ir::{AnalyzeRequest, ParseRequest, ParseResponse};
use tower_http::cors::{Any, CorsLayer};

async fn health() -> &'static str {
    "geist-api-ok"
}

async fn parse(Json(req): Json<ParseRequest>) -> Json<ParseResponse> {
    let semantic_ir = ir::parse_text(&req.text, &req.mode, &req.market_type);
    Json(ParseResponse { semantic_ir })
}

async fn analyze(Json(req): Json<AnalyzeRequest>) -> Json<ir::AnalyzeResponse> {
    Json(pipeline::analyze_from_ir(&req.semantic_ir).await)
}

async fn discover(Json(req): Json<discovery::DiscoverRequest>) -> Json<discovery::DiscoverResponse> {
    Json(discovery::run_discovery(&req.seed, &req.sources).await)
}

fn load_env() {
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_filename("../../.env");
}

#[tokio::main]
async fn main() {
    load_env();

    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("API_PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("{}:{}", host, port);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/parse", post(parse))
        .route("/analyze", post(analyze))
        .route("/discover", post(discover))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Geist API listening on http://localhost:{}", port);
    axum::serve(listener, app).await.unwrap();
}
