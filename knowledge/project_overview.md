---
type: Project
title: Project Overview
description: High-level architecture, design decisions, and component overview of the markdown-converter.
tags: [architecture, project, overview]
generated:
  by: claude-4-20250514/anthropic
  at: "2025-07-28T17:10:00Z"
sources:
  - type: repository
    source: https://github.com/acleveland/markdown-converter
  - type: spec
    source: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md
---

# Project Overview

The **markdown-converter** is a Rust CLI tool that transforms a collection of markdown files — authored in a simple choose-your-own-adventure format — into structured JSON suitable for downstream game engines.

## Problem Statement

Story authors write their adventures as markdown files in Obsidian or any text editor. The tool ingests those files, extracts structural metadata (title, narrative body, selections with attached game logic), and produces a self-contained JSON output that a frontend can consume directly.

## Architecture

The project is a single binary split into three modules:

```
src/
  main.rs      — CLI entry point (clap)
  models.rs    — Data structures (Story, Page, Choice) with serde serialisation
  parser.rs    — Markdown → Page conversion
  loader.rs    — Recursive directory scan, category derivation, collection assembly
```

### Processing Pipeline

1. **Load** — `loader.rs` recursively walks the story directory for `.md` files, computes `category` from the relative path, and parses each file.
2. **Parse** — `parser.rs` extracts the `# Title`, story body (everything before `### Choices`), and each `- [label](link) {metadata}` selection line.
3. **Emit** — `main.rs` serialises the full collection to pretty-printed JSON (stdout or `--output` file).

## Design Decisions

| Decision | Rationale |
|----------|-----------|
| `category` derived from path | Keeps categorisation implicit — no extra metadata required in source files |
| `.md` stripped from links | Normalised identifiers match `filename` field consistently |
| Obsidian `[[ ]]` unwrapped | Same output whether author uses relative paths or wiki-links |
| Bare numbers as cost | `{1}` is a common shorthand; treated as `{cost=1}` |
| `self` link preserved | Self-referencing selections (e.g., retry, crafting) need an explicit target |

## Input Format

Each `.md` file follows this convention:

```markdown
# Title

Story text here.

### Choices
- [Go left](left) {cost=5}
- [Go right](right) {cost=5, msg="Dangerous path"}
```

See [Parser Specification](parser.md) for the complete format reference.

## Output Format

The tool emits a JSON document with a `Story` root containing an `entry_point` and a `pages` array. Each `Page` carries `filename`, `category`, `title`, `story`, and `choices`.

See [JSON Schema](json_schema.md) for the complete schema.
