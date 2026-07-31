---
type: Reference
title: Parser Specification
description: Detailed specification of the input markdown format — titles, story text, choices, and metadata blocks.
tags: [parser, format, specification]
generated:
  by: claude-4-20250514/anthropic
  at: "2025-07-28T17:10:00Z"
sources:
  - type: document
    source: src/parser.rs
---

# Parser Specification

The parser reads a collection of markdown files and structures each into a `Page` with three sections: **title**, **story text**, and **selections** (choices).

## File Structure

A valid input file follows this layout:

```markdown
# Page Title

Opening narrative paragraph(s).

Continuing text with any standard markdown — bold, italic, lists, headings.

### Choices
- [Display text](link-target) {cost=5}
- [Another option](self) {cost=10, msg="Warning message"}
```

## Title

The first top-level heading (`#`) is extracted as the page title.

| Attribute | Rule |
|-----------|------|
| Marker | `#` (single hash + space) |
| Behaviour | Everything after `#` on the same line |
| Fallback | Empty string if no heading found |

## Story Text

All content between the title and `### Choices` is captured as the story body, including sub-headings (`##`, `###`, etc.), lists, code blocks, and inline formatting.

The `### Choices` sentinel itself is **not** included in the story text.

### Examples

| Input | Included? |
|-------|-----------|
| `## Background` | Yes |
| `### Lore` | Yes |
| `### Choices` | No — triggers switch to selection parsing |

## Selections (Choices)

After `### Choices`, each line prefixed with `-` is parsed as a choice.

### Syntax

```
- [Display text](link) {metadata}
```

| Part | Required | Description |
|------|----------|-------------|
| `-` | Yes | Bullet prefix |
| `[Display text]` | Yes | Label shown to player |
| `(link)` | Yes | Target page identifier |
| `{metadata}` | Yes | Game-logic key-value pairs |

### Link Formats

| Syntax | Normalised form | Note |
|--------|----------------|--------|
| `(page)` | `page` | Plain relative link |
| `(page.md)` | `page` | Extension stripped |
| `([[page]])` | `page` | Obsidian wiki-link unwrapped |
| `([[page.md]])` | `page` | Both applied |
| `(self)` | `self` | Self-reference (stay on current page) |

### Metadata Keys

The `{...}` block supports:

| Key | Type | Description | Example |
|-----|------|-------------|---------|
| `cost` | number | Stamina cost of the selection | `{cost=5}` |
| `msg` | string | Additional message shown alongside or instead of story text | `{msg="Just burning stamina"}` |
| `func` | list | Functions to execute on selection, wrapped in `()` | `{func=(set_stats=warrior)}` |
| `req` | list | Inventory items required to make this selection | `{req=herb_red}` |
| `add_item` | list | Items added on selection | `{add_item=potion_health}` |
| `remove_item` | list | Items consumed on selection | `{remove_item=herb_red}` |
| `check` | string | Skill check to evaluate | `{check=INT:15}` |

#### Bare Number Shortcut

A block containing only a number is treated as `cost`:

```
- [Do something](page) {1}
```

is equivalent to:

```
- [Do something](page) {cost=1}
```

### Edge Cases

| Scenario | Behaviour |
|----------|-----------|
| Empty file | Produces a page with empty `title`, `story`, and no `choices` |
| File with title only | `story` is empty; no `choices` |
| `### Choices` with no bullets | `choices` is an empty list |
| Unrecognised metadata key | Silently ignored |
| Malformed metadata | Entire choice falls back to defaults; line is not silently dropped |
