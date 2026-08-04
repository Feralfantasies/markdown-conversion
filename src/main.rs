use std::path::PathBuf;

use clap::Parser;
use dotenvy::dotenv;

use markdown_converter::{create_router, AppState};

/// Markdown-to-game API server
#[derive(Parser, Debug)]
#[command(name = "markdown-converter")]
#[command(about = "Serve a choose-your-own-adventure story as a JSON API")]
struct Cli {
    /// Directory containing markdown story files
    story_dir: PathBuf,

    /// Entry point page (filename without extension)
    #[clap(short, long, default_value = "starting_point")]
    entry_point: String,

    /// Port to listen on
    #[clap(short, long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    // Load `.env` if present; it's optional (e.g. container deployments
    // pass environment variables directly).
    let _ = dotenv();
    let args = Cli::parse();

    let state = AppState::new(&args.story_dir, &args.entry_point);
    let router = create_router(state);

    let addr = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    eprintln!("Story API listening on http://{}", addr);
    axum::serve(listener, router).await.expect("Server failed");
}
