# markdown-converter

Convert a collection of markdown files into structured JSON for a text based adventure game engine.

## Quick Start

```bash
cargo run -- /path/to/story
```

## Knowledge Base

All documentation follows the [Open Knowledge Format (OKF) v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md).

| Document | Description |
|----------|-------------|
| [Project Overview](knowledge/project_overview.md) | Architecture, design decisions, processing pipeline |
| [Parser Specification](knowledge/parser.md) | Input format: titles, story text, choices, metadata blocks |
| [CLI Usage](knowledge/cli.md) | Command-line arguments, flags, examples |
| [JSON Schema](knowledge/json_schema.md) | Output types — `Story`, `Page`, `Choice` definitions |
| [Example](knowledge/example.md) | Concrete input/output walkthrough |

## How It Works

1. **Scan** — Recursively walks a story directory for `.md` files
2. **Parse** — Extracts `# Title`, story body, and `- [label](link) {metadata}` selections
3. **Emit** — Produces pretty-printed JSON with categorical page grouping
