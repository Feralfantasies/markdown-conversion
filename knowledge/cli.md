---
type: Reference
title: CLI Usage
description: Command-line interface reference — arguments, flags, and usage examples.
tags: [cli, usage, reference]
generated:
  by: claude-4-20250514/anthropic
  at: "2026-08-04T20:11:00Z"
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
| `--entry-point` | `-e` | `starting_point` | Filename (without extension) of the game's entry page |
| `--port` | `-p` | `3000` | Port for the HTTP API server to listen on |
| `--help` | `-h` | — | Print help message |

## Environment

At startup the binary optionally loads a `.env` file from the current working
directory if one exists. If no `.env` file is present (e.g. inside a container)
startup proceeds normally — all configuration is supplied via CLI flags.

## Examples

### Basic usage

```bash
cargo run -- /path/to/story
```

### Custom entry point

```bash
cargo run -- /path/to/story -e start_room
```

### Custom port

```bash
cargo run -- /path/to/story -p 8080
```

### Full options

```bash
cargo run -- /path/to/story -e character_creator -p 8080
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success — server shut down cleanly |
| non-zero | Error (e.g., missing story directory, bind failure) |

## Installation

```bash
cargo install --path .
markdown-converter /path/to/story