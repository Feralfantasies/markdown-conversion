mod loader;
mod models;
mod parser;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "markdown-converter")]
#[command(
    about = "Convert a collection of markdown files into a choose-your-own-adventure game JSON"
)]
struct Cli {
    /// Directory containing markdown story files
    story_dir: PathBuf,

    /// Entry point page (filename without extension)
    #[clap(short, long, default_value = "character_creator")]
    entry_point: String,

    /// Output file path (default: stdout)
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let story = loader::load_story_collection(&cli.story_dir, &cli.entry_point);

    let json = serde_json::to_string_pretty(&story).expect("Failed to serialize to JSON");

    if let Some(out_path) = cli.output {
        std::fs::write(&out_path, &json).expect("Failed to write output file");
        eprintln!("Wrote {} pages to {:?}", story.pages.len(), out_path);
    } else {
        println!("{}", json);
    }
}
