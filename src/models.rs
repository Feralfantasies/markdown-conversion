use serde::{Deserialize, Serialize};

/// A single choice option presented to the player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// Display text shown on the button/option
    pub display: String,
    /// Target page link (relative path, "self", or class reference)
    pub link: String,
    /// Stamina cost
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    /// Optional message to display to the player
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    /// Functions to execute (e.g., set_stats=warrior)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub func: Option<Vec<String>>,
    /// Required item(s) for this choice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub req: Option<Vec<String>>,
    /// Item(s) added when this choice is taken
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_item: Option<Vec<String>>,
    /// Item(s) removed when this choice is taken
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_item: Option<Vec<String>>,
    /// Skill check (e.g., INT, DEX, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<String>,
}

/// A single page/node in the adventure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// The filename (without extension)
    pub filename: String,
    /// Category derived from directory structure (e.g., "character_setup", "starting_zone/cottage")
    pub category: String,
    /// The title from the `# ` heading
    pub title: String,
    /// The story body text (everything before `### Choices`)
    pub story: String,
    /// The list of choices that lead to other pages
    pub choices: Vec<Choice>,
}

/// The root JSON container for the entire story collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    /// The entry point page (e.g., "character_creator")
    pub entry_point: String,
    /// All pages indexed by their filename
    pub pages: Vec<Page>,
}
