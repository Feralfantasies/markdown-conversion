---
type: Reference
title: CLI Usage
description: Command-line interface reference — arguments, flags, and usage examples.
tags: [cli, usage, reference]
generated:
  by: claude-opus-4-20250514/anthropic
  at: "2025-07-28T17:10:00Z"
sources:
  - type: document
    source: src/main.rs
---

# CLI Usage

## Synopsis

```bash
markdown-converter <story-dir> [OPTIONS]
```

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `story-dir` | Yes | Path to the directory containing `.md` adventure files |

## Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--entry-point` | `-e` | `character_creator` | Filename (without extension) of the game's entry page |
| `--output` | `-o` | *(stdout)* | File path to write JSON output |
| `--help` | `-h` | — | Print help message |

## Examples

### Basic usage — stdout

```bash
cargo run -- /path/to/story
```

### Specify output file

```bash
cargo run -- /path/to/story -o story.json
```

### Custom entry point

```bash
cargo run -- /path/to/story -e start_room
```

### Full options

```bash
cargo run -- /path/to/story -e character_creator -o output/game.json
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success — JSON emitted |
| `1` | Error (e.g., missing directory, I/O failure) |

## Installation

```bash
cargo install --path .
markdown-converter /path/to/story
```
