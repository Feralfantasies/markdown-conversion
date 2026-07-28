use std::fs;

use crate::models::{Choice, Page};

/// Parse the metadata block `{...}` from a choice line.
/// Extracts keys like cost, msg, func, req, add_item, remove_item, check.
fn parse_metadata_block(block: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut current_key = None::<String>;
    let mut current_val = String::new();
    let mut paren_depth: u32 = 0;

    let mut i = 0;
    let chars: Vec<char> = block.chars().collect();

    while i < chars.len() {
        let ch = chars[i];

        if ch == '(' {
            current_val.push(ch);
            paren_depth += 1;
            i += 1;
            continue;
        }

        if ch == ')' {
            current_val.push(ch);
            paren_depth = paren_depth.saturating_sub(1);
            i += 1;
            continue;
        }

        if ch == '=' && paren_depth == 0 {
            current_key = Some(current_val.trim().to_string());
            current_val.clear();
            i += 1;
            continue;
        }

        if ch == '"' {
            // Quoted value
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                current_val.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1; // skip closing quote
            }
            if let Some(key) = current_key.take() {
                pairs.push((key, current_val.clone()));
            }
            current_val.clear();
            i += 1;
            continue;
        }

        if ch == ',' && paren_depth == 0 {
            // End of a value (unquoted)
            if let Some(key) = current_key.take() {
                pairs.push((key, current_val.trim().to_string()));
            }
            current_val.clear();
            i += 1;
            continue;
        }

        // Skip whitespace at start of value (no key yet)
        if current_key.is_none() && current_val.is_empty() && ch.is_whitespace() {
            i += 1;
            continue;
        }

        current_val.push(ch);
        i += 1;
    }

    // Handle last value if no trailing comma
    if let Some(key) = current_key.take() {
        if !current_val.trim().is_empty() {
            pairs.push((key, current_val.trim().to_string()));
        }
    }

    pairs
}

/// Parse a single choice line and extract the display text, target link, and metadata.
fn parse_choice_line(line: &str) -> Option<Choice> {
    // Match: - [Display Text](link) {metadata}
    // or:    - [Display Text]([[link]]) {metadata}
    // or:    - [Display Text](link) {cost=X} (bare link)

    let trimmed = line.trim();
    if !trimmed.starts_with('-') {
        return None;
    }

    // Remove leading "- " or "- "
    let after_dash = trimmed.get(1..)?.trim();

    // Extract display text from [...].
    let open_bracket = after_dash.find('[')?;
    let close_bracket = after_dash.find(']')?;
    let display = after_dash[open_bracket + 1..close_bracket].to_string();

    let remaining = &after_dash[close_bracket + 1..];

    // Extract link from (...).
    let link_open = remaining.find('(')?;
    let link_close = remaining.find(')')?;
    let link = remaining[link_open + 1..link_close].to_string();

    let remaining = &remaining[link_close + 1..];

    // Parse optional metadata block {...}
    let metadata_start = remaining.find('{');
    let mut cost: Option<f64> = None;
    let mut msg: Option<String> = None;
    let mut func: Option<Vec<String>> = None;
    let mut req: Option<Vec<String>> = None;
    let mut add_item: Option<Vec<String>> = None;
    let mut remove_item: Option<Vec<String>> = None;
    let mut check: Option<String> = None;

    if let Some(pos) = metadata_start {
        let block = &remaining[pos + 1..].trim_end_matches('}');
        let pairs = parse_metadata_block(block);

        // Handle bare number (e.g., {1}) as cost
        if pairs.is_empty() {
            let bare = block.trim();
            if let Ok(n) = bare.parse::<f64>() {
                cost = Some(n);
            }
        } else {
            for (key, val) in pairs {
                match key.as_str() {
                    "cost" => {
                        cost = Some(val.parse::<f64>().ok()?);
                    }
                    "msg" => {
                        msg = Some(val);
                    }
                    "func" => {
                        // Format: (set_stats=warrior) or (set_stats=warrior","set_flag=true)
                        let inner = val.trim_start_matches('(').trim_end_matches(')');
                        func = Some(inner.split(',').map(|s| s.trim().to_string()).collect());
                    }
                    "req" => {
                        req = Some(val.split(',').map(|s| s.trim().to_string()).collect());
                    }
                    "add_item" => {
                        add_item = Some(val.split(',').map(|s| s.trim().to_string()).collect());
                    }
                    "remove_item" => {
                        remove_item = Some(val.split(',').map(|s| s.trim().to_string()).collect());
                    }
                    "check" => {
                        check = Some(val);
                    }
                    _ => {}
                }
            }
        }
    }

    // Clean up link: remove [[ ]] wrapper and trailing .md
    let clean_link = if link.starts_with("[[") && link.ends_with("]]") {
        link[2..link.len() - 2].to_string()
    } else {
        link
    };
    let clean_link = clean_link
        .strip_suffix(".md")
        .unwrap_or(&clean_link)
        .to_string();

    Some(Choice {
        display,
        link: clean_link,
        cost,
        msg,
        func,
        req,
        add_item,
        remove_item,
        check,
    })
}

/// Parse a markdown file into a Page struct.
pub fn parse_markdown_content(content: &str) -> Option<Page> {
    let mut title_found = String::new();
    let mut story_lines = Vec::<String>::new();
    let mut choice_lines = Vec::new();
    let mut in_choices = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Look for the main title (# heading)
        if !in_choices
            && trimmed.starts_with("# ")
            && !trimmed.starts_with("## ")
            && title_found.is_empty()
        {
            title_found = trimmed.get(2..).unwrap_or("").to_string();
            continue;
        }

        // Detect the start of the choices section
        if !in_choices && (trimmed.starts_with("### Choices") || trimmed == "### Choices") {
            in_choices = true;
            continue;
        }

        if in_choices {
            // Gather choice lines
            if trimmed.starts_with('-') {
                choice_lines.push(line.to_string());
            }
        } else {
            // Gather story text
            if !trimmed.is_empty()
                || (!story_lines.is_empty() && !story_lines.last().unwrap().is_empty())
            {
                story_lines.push(line.to_string());
            }
        }
    }

    // Parse choices
    let choices: Vec<Choice> = choice_lines
        .iter()
        .filter_map(|line| parse_choice_line(line))
        .collect();

    // Clean up story text
    let story = story_lines.join("\n").trim().to_string();

    Some(Page {
        title: title_found,
        story,
        choices,
        filename: String::new(), // Will be set by caller
        category: String::new(), // Will be set by caller
    })
}

/// Parse a markdown file from disk.
pub fn parse_file(path: &std::path::Path) -> Option<Page> {
    let content = fs::read_to_string(path).ok()?;
    parse_markdown_content(&content).map(|mut page| {
        page.filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        page
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metadata_block_basic() {
        let pairs = parse_metadata_block("cost=5");
        assert_eq!(pairs, vec![("cost".to_string(), "5".to_string())]);
    }

    #[test]
    fn test_parse_metadata_block_multiple() {
        let pairs = parse_metadata_block("cost=5, msg=\"Just burning stamina\"");
        assert_eq!(
            pairs,
            vec![
                ("cost".to_string(), "5".to_string()),
                ("msg".to_string(), "Just burning stamina".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_metadata_block_func() {
        let pairs = parse_metadata_block("cost=0, func=(set_stats=warrior)");
        assert_eq!(
            pairs,
            vec![
                ("cost".to_string(), "0".to_string()),
                ("func".to_string(), "(set_stats=warrior)".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_choice_line_basic() {
        let line =
            "- [Search the cupboards](starting_zone/cottage/cupboard_investigation) {cost=5}";
        let choice = parse_choice_line(line).expect("Should parse");
        assert_eq!(choice.display, "Search the cupboards");
        assert_eq!(choice.link, "starting_zone/cottage/cupboard_investigation");
        assert_eq!(choice.cost, Some(5.0));
    }

    #[test]
    fn test_parse_choice_line_with_msg() {
        let line = r#"- [Burn some Stamina](self) {cost=100, msg="Just burning stamina"}"#;
        let choice = parse_choice_line(line).expect("Should parse");
        assert_eq!(choice.link, "self");
        assert_eq!(choice.cost, Some(100.0));
        assert_eq!(choice.msg, Some("Just burning stamina".to_string()));
    }

    #[test]
    fn test_parse_choice_line_obsidian_link() {
        let line =
            "- [Confirm Warrior Archetype]([[start_room]]) {cost=0, func=(set_stats=warrior)}";
        let choice = parse_choice_line(line).expect("Should parse");
        assert_eq!(choice.link, "start_room");
        assert_eq!(choice.cost, Some(0.0));
        assert!(choice.func.is_some());
    }

    #[test]
    fn test_parse_markdown_content_basic() {
        let content = "# The Awakening
You slowly awaken.

### Choices
- [Go outside](outside) {cost=1}";
        let page = parse_markdown_content(content).expect("Should parse");
        assert_eq!(page.title, "The Awakening");
        assert_eq!(page.story, "You slowly awaken.");
        assert_eq!(page.choices.len(), 1);
        assert_eq!(page.choices[0].link, "outside");
        assert_eq!(page.choices[0].cost, Some(1.0));
    }

    #[test]
    fn test_parse_choice_line_simple_cost() {
        // Handle format like ` {1} ` — .md is stripped from link
        let line = "- [Skim though the books on the shelf](read_books.md) {1}";
        let choice = parse_choice_line(line).expect("Should parse");
        assert_eq!(choice.link, "read_books");
        assert_eq!(choice.display, "Skim though the books on the shelf");
    }
}
