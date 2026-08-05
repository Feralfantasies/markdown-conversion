use std::fs;

use crate::models::{Choice, Page};

// ── Top-level segment / key-value helpers ────────────────────────────────────

/// Split a string on top-level commas, respecting double-quoted strings,
/// parentheses `()` and square brackets `[]`. Nested delimiters and commas
/// inside quotes do not split.
fn split_top_level_segments(input: &str) -> Vec<String> {
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;

    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        if in_string {
            current.push(ch);
            if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                current.push(ch);
            }
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                current.push(ch);
            }
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                segments.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        segments.push(current);
    }

    segments
}

/// Split a `key=value` segment on the first `=` that appears at depth zero
/// (outside any string, parentheses, or brackets).
fn split_key_value(segment: &str) -> Option<(String, String)> {
    let chars: Vec<char> = segment.chars().collect();
    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in chars.iter().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }

        if in_string {
            if *ch == '\\' {
                escaped = true;
            } else if *ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '=' if paren_depth == 0 && bracket_depth == 0 => {
                let key: String = chars[..idx].iter().collect();
                let val: String = chars[idx + 1..].iter().collect();
                return Some((key.trim().to_string(), clean_scalar(&val)));
            }
            _ => {}
        }
    }

    None
}

/// If the value is a *fully* quoted string (the closing quote is the last
/// character), strip the surrounding quotes. Otherwise return it as-is so that
/// callers can interpret compound values (e.g. `msg="..." var` or `[...]`).
fn clean_scalar(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') {
        let chars: Vec<char> = trimmed.chars().collect();
        let mut i = 1;
        let mut escaped = false;
        while i < chars.len() {
            let ch = chars[i];
            if escaped {
                escaped = false;
                i += 1;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                i += 1;
                continue;
            }
            if ch == '"' {
                // Fully quoted only if this quote is the final character.
                if i == chars.len() - 1 {
                    return chars[1..i].iter().collect();
                }
                break;
            }
            i += 1;
        }
    }
    trimmed.to_string()
}

/// Parse a metadata block's contents into key/value pairs. Values that are
/// fully quoted strings are unquoted; compound values (arrays, msg templates
/// with a trailing variable) are returned raw for the caller to interpret.
fn parse_metadata_block(block: &str) -> Vec<(String, String)> {
    split_top_level_segments(block)
        .into_iter()
        .filter_map(|seg| split_key_value(&seg))
        .collect()
}

// ── Value interpretation helpers ─────────────────────────────────────────────

/// Interpret a `msg` value. Handles both quoted templates and an optional
/// trailing variable name used to fill the `{}` placeholder:
///
///   msg="It is currently {}" current_time   -> ("It is currently {}", Some("current_time"))
///   msg="Plain message"                     -> ("Plain message", None)
fn parse_msg_value(value: &str) -> (String, Option<String>) {
    let trimmed = value.trim();

    if trimmed.starts_with('"') {
        let chars: Vec<char> = trimmed.chars().collect();
        let mut message = String::new();
        let mut i = 1; // skip opening quote
        let mut escaped = false;

        while i < chars.len() {
            let ch = chars[i];
            if escaped {
                message.push(ch);
                escaped = false;
                i += 1;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                i += 1;
                continue;
            }
            if ch == '"' {
                break;
            }
            message.push(ch);
            i += 1;
        }

        // Everything after the closing quote is the optional variable name.
        let remainder: String = chars
            .get(i + 1..)
            .map(|c| c.iter().collect())
            .unwrap_or_default();
        let var = remainder.trim().trim_end_matches(',').trim().to_string();
        let var = if var.is_empty() { None } else { Some(var) };
        (message, var)
    } else {
        let message = trimmed.trim_end_matches(',').trim().to_string();
        (message, None)
    }
}

/// Interpret an array value like `[a, b, c]` into a list of strings. Items are
/// split on top-level commas so nested `(...)` are preserved intact.
fn parse_array_value(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    let inner = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    split_top_level_segments(inner)
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Split a simple comma-separated list (req / add_item / remove_item).
fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Extract the contents of the first balanced `{...}` block in `input`,
/// ignoring braces inside double-quoted strings.
fn extract_metadata_block(input: &str) -> Option<String> {
    let chars: Vec<char> = input.chars().collect();
    let start = chars.iter().position(|&c| c == '{')?;
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut i = start;

    while i < chars.len() {
        let ch = chars[i];
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if in_string {
            if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(chars[start + 1..i].iter().collect());
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}

// ── Choice block parsing ─────────────────────────────────────────────────────

/// Parse a single choice (possibly spanning multiple lines) and extract the
/// display text, target link, and metadata.
fn parse_choice_block(block: &str) -> Option<Choice> {
    let trimmed = block.trim();
    if !trimmed.starts_with('-') {
        return None;
    }

    // Remove leading "-".
    let after_dash = trimmed.get(1..)?.trim();

    // Display text: first [...].
    let open_bracket = after_dash.find('[')?;
    let close_bracket = after_dash.find(']')?;
    if close_bracket <= open_bracket {
        return None;
    }
    let display = after_dash[open_bracket + 1..close_bracket].to_string();
    let remaining = &after_dash[close_bracket + 1..];

    // Link: first (...).
    let link_open = remaining.find('(')?;
    let link_close = remaining.find(')')?;
    if link_close <= link_open {
        return None;
    }
    let link = remaining[link_open + 1..link_close].to_string();
    let remaining = &remaining[link_close + 1..];

    // Optional metadata block.
    let mut cost: Option<f64> = None;
    let mut msg: Option<String> = None;
    let mut msg_var: Option<String> = None;
    let mut func: Option<Vec<String>> = None;
    let mut functions: Option<Vec<String>> = None;
    let mut req: Option<Vec<String>> = None;
    let mut add_item: Option<Vec<String>> = None;
    let mut remove_item: Option<Vec<String>> = None;
    let mut check: Option<String> = None;

    if let Some(meta) = extract_metadata_block(remaining) {
        let pairs = parse_metadata_block(&meta);

        // Bare number shortcut: `{1}` is treated as cost.
        if pairs.is_empty() {
            if let Ok(n) = meta.trim().parse::<f64>() {
                cost = Some(n);
            }
        } else {
            for (key, val) in pairs {
                match key.as_str() {
                    "cost" => {
                        cost = val.parse::<f64>().ok();
                    }
                    "msg" => {
                        let (message, var) = parse_msg_value(&val);
                        msg = Some(message);
                        msg_var = var;
                    }
                    "functions" => {
                        functions = Some(parse_array_value(&val));
                    }
                    "func" => {
                        // Legacy single-function syntax wrapped in ().
                        let inner = val.trim().trim_start_matches('(').trim_end_matches(')');
                        func = Some(
                            inner
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect(),
                        );
                    }
                    "req" => req = Some(split_list(&val)),
                    "add_item" => add_item = Some(split_list(&val)),
                    "remove_item" => remove_item = Some(split_list(&val)),
                    "check" => check = Some(val),
                    _ => {}
                }
            }
        }
    }

    // Clean up link: remove [[ ]] wrapper and trailing .md.
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
        msg_var,
        func,
        functions,
        req,
        add_item,
        remove_item,
        check,
    })
}

// ── Multi-line choice block accumulation ─────────────────────────────────────

/// Track how braces/strings evolve across a line so we know when a multi-line
/// choice block is complete.
fn scan_line_state(line: &str, in_string: &mut bool, brace_depth: &mut i32, saw_brace: &mut bool) {
    let mut escaped = false;
    for ch in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if *in_string {
            if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                *in_string = false;
            }
            continue;
        }
        match ch {
            '"' => *in_string = true,
            '{' => {
                *brace_depth += 1;
                *saw_brace = true;
            }
            '}' => *brace_depth -= 1,
            _ => {}
        }
    }
}

fn block_is_open(saw_brace: bool, brace_depth: i32, in_string: bool) -> bool {
    in_string || (saw_brace && brace_depth > 0)
}

fn choice_complete(saw_brace: bool, brace_depth: i32, in_string: bool) -> bool {
    if in_string {
        return false;
    }
    if !saw_brace {
        // Single-line choice with no metadata block.
        return true;
    }
    brace_depth <= 0
}

/// Group raw choice-section lines into complete choice blocks. A new choice
/// begins at a line starting with `-` (when no block is open); subsequent
/// lines are continuations until the `{...}` block balances.
fn collect_choice_blocks(choice_lines: &[String]) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut has_current = false;
    let mut in_string = false;
    let mut brace_depth: i32 = 0;
    let mut saw_brace = false;

    let reset = |in_string: &mut bool, brace_depth: &mut i32, saw_brace: &mut bool| {
        *in_string = false;
        *brace_depth = 0;
        *saw_brace = false;
    };

    for line in choice_lines {
        let trimmed = line.trim();
        let starts_new = trimmed.starts_with('-') && !in_string && brace_depth <= 0;

        if starts_new {
            if has_current {
                blocks.push(std::mem::take(&mut current));
                reset(&mut in_string, &mut brace_depth, &mut saw_brace);
            }
            current = line.clone();
            has_current = true;
            scan_line_state(line, &mut in_string, &mut brace_depth, &mut saw_brace);
            if choice_complete(saw_brace, brace_depth, in_string) {
                blocks.push(std::mem::take(&mut current));
                has_current = false;
                reset(&mut in_string, &mut brace_depth, &mut saw_brace);
            }
            continue;
        }

        if has_current {
            // Ignore blank lines that sit outside an open block.
            if trimmed.is_empty() && !block_is_open(saw_brace, brace_depth, in_string) {
                continue;
            }
            current.push('\n');
            current.push_str(line);
            scan_line_state(line, &mut in_string, &mut brace_depth, &mut saw_brace);
            if choice_complete(saw_brace, brace_depth, in_string) {
                blocks.push(std::mem::take(&mut current));
                has_current = false;
                reset(&mut in_string, &mut brace_depth, &mut saw_brace);
            }
        }
    }

    if has_current && !current.trim().is_empty() {
        blocks.push(current);
    }

    blocks
}

// ── Page parsing ─────────────────────────────────────────────────────────────

/// Parse a markdown document into a Page struct.
pub fn parse_markdown_content(content: &str) -> Option<Page> {
    let mut title_found = String::new();
    let mut story_lines = Vec::<String>::new();
    let mut choice_lines = Vec::new();
    let mut in_choices = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Look for the main title (# heading).
        if !in_choices
            && trimmed.starts_with("# ")
            && !trimmed.starts_with("## ")
            && title_found.is_empty()
        {
            title_found = trimmed.get(2..).unwrap_or("").to_string();
            continue;
        }

        // Detect the start of the choices section.
        if !in_choices && (trimmed.starts_with("### Choices") || trimmed == "### Choices") {
            in_choices = true;
            continue;
        }

        if in_choices {
            // Gather choice lines (including continuations of multi-line blocks).
            choice_lines.push(line.to_string());
        } else {
            // Gather story text (skip leading blank lines, keep internal ones).
            if !trimmed.is_empty()
                || (!story_lines.is_empty() && !story_lines.last().unwrap().is_empty())
            {
                story_lines.push(line.to_string());
            }
        }
    }

    // Group the raw lines into complete choice blocks, then parse each.
    let choices: Vec<Choice> = collect_choice_blocks(&choice_lines)
        .into_iter()
        .filter_map(|block| parse_choice_block(&block))
        .collect();

    // Clean up story text.
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

    // ── Metadata block parsing ───────────────────────────────────────────

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
    fn test_parse_metadata_block_functions_array() {
        let pairs = parse_metadata_block(
            "cost=10,\n  functions=[\n  set_character_class(road_warrior),\n  set_background(road.png)\n  ]",
        );
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("cost".to_string(), "10".to_string()));
        assert!(pairs[1].1.contains("set_character_class(road_warrior)"));
        assert!(pairs[1].1.contains("set_background(road.png)"));
    }

    // ── msg value interpretation ─────────────────────────────────────────

    #[test]
    fn test_parse_msg_value_plain() {
        let (message, var) = parse_msg_value("Just burning stamina");
        assert_eq!(message, "Just burning stamina");
        assert_eq!(var, None);
    }

    #[test]
    fn test_parse_msg_value_with_variable() {
        let (message, var) = parse_msg_value(
            "\"You look down at your watch to see the time, its currently {}\" current_time",
        );
        assert_eq!(
            message,
            "You look down at your watch to see the time, its currently {}"
        );
        assert_eq!(var, Some("current_time".to_string()));
    }

    // ── Choice parsing ───────────────────────────────────────────────────

    #[test]
    fn test_parse_choice_line_basic() {
        let line =
            "- [Search the cupboards](starting_zone/cottage/cupboard_investigation) {cost=5}";
        let choice = parse_choice_block(line).expect("Should parse");
        assert_eq!(choice.display, "Search the cupboards");
        assert_eq!(choice.link, "starting_zone/cottage/cupboard_investigation");
        assert_eq!(choice.cost, Some(5.0));
    }

    #[test]
    fn test_parse_choice_line_with_msg() {
        let line = r#"- [Burn some Stamina](self) {cost=100, msg="Just burning stamina"}"#;
        let choice = parse_choice_block(line).expect("Should parse");
        assert_eq!(choice.link, "self");
        assert_eq!(choice.cost, Some(100.0));
        assert_eq!(choice.msg, Some("Just burning stamina".to_string()));
        assert_eq!(choice.msg_var, None);
    }

    #[test]
    fn test_parse_choice_line_obsidian_link() {
        let line =
            "- [Confirm Warrior Archetype]([[start_room]]) {cost=0, func=(set_stats=warrior)}";
        let choice = parse_choice_block(line).expect("Should parse");
        assert_eq!(choice.link, "start_room");
        assert_eq!(choice.cost, Some(0.0));
        assert!(choice.func.is_some());
    }

    #[test]
    fn test_parse_choice_line_simple_cost() {
        // Handle format like ` {1} ` — .md is stripped from link
        let line = "- [Skim though the books on the shelf](read_books.md) {1}";
        let choice = parse_choice_block(line).expect("Should parse");
        assert_eq!(choice.link, "read_books");
        assert_eq!(choice.display, "Skim though the books on the shelf");
    }

    #[test]
    fn test_parse_multiline_choice_with_functions_and_msg() {
        let block = r#"- [Check your watch](self) {
  cost=0,
  functions=[
  current_time=get_time()
  ],
  msg="You look down at your watch to see the time, its currently {}" current_time,
  }"#;
        let choice = parse_choice_block(block).expect("Should parse multi-line block");
        assert_eq!(choice.display, "Check your watch");
        assert_eq!(choice.link, "self");
        assert_eq!(choice.cost, Some(0.0));
        assert_eq!(
            choice.functions,
            Some(vec!["current_time=get_time()".to_string()])
        );
        assert_eq!(
            choice.msg,
            Some("You look down at your watch to see the time, its currently {}".to_string())
        );
        assert_eq!(choice.msg_var, Some("current_time".to_string()));
    }

    // ── Full page parsing ────────────────────────────────────────────────

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
    fn test_parse_markdown_content_multiline_choices() {
        let content = "# Starting Point

Adventure begins.

### Choices

- [Explore the road]([[road.md]]) {
  cost=10,
  functions=[
  set_character_class(road_warrior),
  set_background(road.png)
  ]
  }
- [Explore the forest]([[forest.md]]) {
  cost=10,
  functions=[
  set_character_class(forest_wizard),
  set_background(forest.png)
  ]
  }
- [Check your watch](self) {
  cost=0,
  functions=[
  current_time=get_time()
  ],
  msg=\"You look down at your watch to see the time, its currently {}\" current_time,
  }";
        let page = parse_markdown_content(content).expect("Should parse");
        assert_eq!(page.title, "Starting Point");
        assert_eq!(page.choices.len(), 3);

        let road = &page.choices[0];
        assert_eq!(road.display, "Explore the road");
        assert_eq!(road.link, "road");
        assert_eq!(road.cost, Some(10.0));
        assert_eq!(
            road.functions,
            Some(vec![
                "set_character_class(road_warrior)".to_string(),
                "set_background(road.png)".to_string(),
            ])
        );

        let forest = &page.choices[1];
        assert_eq!(forest.link, "forest");
        assert_eq!(
            forest.functions,
            Some(vec![
                "set_character_class(forest_wizard)".to_string(),
                "set_background(forest.png)".to_string(),
            ])
        );

        let watch = &page.choices[2];
        assert_eq!(watch.link, "self");
        assert_eq!(watch.cost, Some(0.0));
        assert_eq!(
            watch.functions,
            Some(vec!["current_time=get_time()".to_string()])
        );
        assert_eq!(watch.msg_var, Some("current_time".to_string()));
    }
}
