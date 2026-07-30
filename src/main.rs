mod loader;
mod models;
mod parser;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Json, Router, extract::Path, http::StatusCode, middleware::from_fn, routing::get,
};
use clap::Parser;

/// Markdown-to-game API server
#[derive(Parser, Debug)]
#[command(name = "markdown-converter")]
#[command(
    about = "Serve a choose-your-own-adventure story as a JSON API"
)]
struct Cli {
    /// Directory containing markdown story files
    story_dir: PathBuf,

    /// Entry point page (filename without extension)
    #[clap(short, long, default_value = "character_creator")]
    entry_point: String,

    /// Port to listen on
    #[clap(short, long, default_value = "3000")]
    port: u16,
}

// ── API state ────────────────────────────────────────────────────────────────

/// Shared app state, wrapped in Arc for Clone.
#[derive(Clone)]
pub struct AppState {
    pages: Arc<HashMap<String, models::Page>>,
    entry_point: Arc<String>,
}

impl AppState {
    fn new(story_dir: &PathBuf, entry_point: &str) -> Self {
        let story = loader::load_story_collection(story_dir, entry_point);
        let mut pages: HashMap<String, models::Page> = HashMap::new();

        for page in story.pages {
            pages.insert(page.filename.clone(), page);
        }

        eprintln!(
            "Loaded {} pages, entry point: {}",
            pages.len(),
            entry_point
        );

        Self {
            pages: Arc::new(pages),
            entry_point: Arc::new(entry_point.to_string()),
        }
    }
}

// ── Middleware ───────────────────────────────────────────────────────────────

async fn cors_header(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::http::Response<axum::body::Body> {
    let mut res = next.run(req).await;
    res.headers_mut().insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        axum::http::HeaderValue::from_static("*"),
    );
    res.headers_mut().insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_METHODS,
        axum::http::HeaderValue::from_static("GET"),
    );
    res
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET / — Serve the entry/start page.
async fn get_start(
    state: axum::extract::State<AppState>,
) -> Result<Json<models::Page>, (StatusCode, String)> {
    match state.pages.get(&*state.entry_point) {
        Some(page) => Ok(Json(page.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            format!("Entry point '{}' not found", &*state.entry_point),
        )),
    }
}

/// GET /pages/{id} — Serve a specific page by filename.
async fn get_page(
    Path(id): Path<String>,
    state: axum::extract::State<AppState>,
) -> Result<Json<models::Page>, (StatusCode, String)> {
    match state.pages.get(&id) {
        Some(page) => Ok(Json(page.clone())),
        None => Err((StatusCode::NOT_FOUND, format!("Page '{}' not found", id))),
    }
}

/// GET /pages — List all available page IDs.
async fn list_pages(
    state: axum::extract::State<AppState>,
) -> Json<Vec<String>> {
    let mut ids: Vec<String> = state.pages.keys().cloned().collect();
    ids.sort();
    Json(ids)
}

// ── Router ───────────────────────────────────────────────────────────────────

fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_start))
        .route("/pages", get(list_pages))
        .route("/pages/{id}", get(get_page))
        .fallback(get(fallback))
        .layer(from_fn(cors_header))
        .with_state(state)
}

/// 404 fallback for unhandled routes.
async fn fallback() -> (StatusCode, String) {
    (
        StatusCode::NOT_FOUND,
        "Endpoint not found. Try GET / or GET /pages/{id}.".to_string(),
    )
}

// ── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let state = AppState::new(&args.story_dir, &args.entry_point);
    let router = create_router(state);

    let addr = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    eprintln!("Story API listening on http://{}", addr);
    axum::serve(listener, router)
        .await
        .expect("Server failed");
}
