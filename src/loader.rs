use std::path::{Path, PathBuf};

use crate::models::{Page, Story};
use crate::parser;

/// Recursively scan a directory for `.md` files and parse them into pages.
pub fn load_story_collection(root: &Path, entry_point: &str) -> Story {
    let mut pages: Vec<Page> = Vec::new();

    collect_md_files(root, root, &mut pages);

    // Sort by category then filename for deterministic output
    pages.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then(a.filename.cmp(&b.filename))
    });

    Story {
        entry_point: entry_point.to_string(),
        pages,
    }
}

/// Compute the category from a file path relative to the story root.
/// e.g. "character_creator" -> "root" (top level files)
///      character_setup/class_warrior.md -> "character_setup"
///      starting_zone/cottage/bedroom_morning.md -> "starting_zone/cottage"
fn compute_category(root: &Path, file_path: &Path) -> String {
    let rel = file_path
        .strip_prefix(root)
        .unwrap_or(file_path)
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_string_lossy()
        .to_string();

    if rel.is_empty() {
        "root".to_string()
    } else {
        rel
    }
}

fn collect_md_files(dir: &Path, root: &Path, pages: &mut Vec<Page>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut files: Vec<PathBuf> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_md_files(&path, root, pages);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }

    for file in files {
        let category = compute_category(root, &file);
        if let Some(mut page) = parser::parse_file(&file) {
            page.category = category;
            pages.push(page);
        }
    }
}
