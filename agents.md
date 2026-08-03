# AI Agent Instructions

This repository is the **markdown-converter** — a Rust application that converts a collection of markdown adventure files into structured JSON and serves them via an HTTP API.

## Primary Knowledge Source: `knowledge/`

Before starting any work in this repository, **always read the `knowledge/` directory**. It is the single source of truth for project documentation and is maintained in **OKF (Open Knowledge Format)**.

### Structure

| File                            | Purpose                                                                      |
| ------------------------------- | ---------------------------------------------------------------------------- |
| `knowledge/index.md`            | Index of all documents — start here                                          |
| `knowledge/project_overview.md` | High-level architecture and design decisions                                 |
| `knowledge/parser.md`           | Markdown input format specification (title, story, choices, metadata blocks) |
| `knowledge/api.md`              | HTTP API endpoints and client interaction                                    |
| `knowledge/cli.md`              | Command-line arguments and startup behaviour                                 |
| `knowledge/json_schema.md`      | Output types (Story, Page, Choice)                                           |
| `knowledge/example.md`          | Concrete input/output walkthrough                                            |
| `test_story/`                   | Test fixture used by the integration tests                                   |

### OKF Format Conventions

Every document in `knowledge/` uses YAML frontmatter. When creating or updating a document, follow this template:

```yaml
---
type: Reference # e.g. Index, Reference, Guide, Example
title: Document Title
description: One-line summary of what this document covers.
tags: [relevant, tags]
generated:
  by: agent-or-author-name
  at: "YYYY-MM-DDTHH:MM:SSZ"
sources:
  - type: document
    source: path/to/source_file.rs # source files this doc was derived from
---
```

Rules:

- **`knowledge/index.md`** must list every document in the directory. When adding a new document, add it to the Concepts table in `index.md`.
- Each document should cover **one focused area** of the application.
- Keep frontmatter `sources` up to date so documents can be traced back to their source files.

## Mandatory: Keep `knowledge/` Up To Date

Whenever you add, change, or remove features in this codebase, you **must** update the corresponding documentation in `knowledge/` as part of the same change. This includes (but is not limited to):

- **New or changed API endpoints** → update `knowledge/api.md`
- **Changes to the markdown parsing rules** → update `knowledge/parser.md`
- **New or changed CLI arguments / startup behaviour** → update `knowledge/cli.md`
- **Changes to data structures (Story, Page, Choice)** → update `knowledge/json_schema.md`
- **Architectural changes (new modules, state management, library/binary split)** → update `knowledge/project_overview.md`
- **New examples or changed fixture format** → update `knowledge/example.md`
- **New documentation areas not covered above** → create a new OKF document and register it in `knowledge/index.md`

If a document is missing for an area you are working on, create it using the OKF template above and add it to the index.

## Testing

- Integration tests live in `tests/story_api.rs` and run with `cargo test`.
- The tests start a real Axum server in-process on an ephemeral port using the `test_story/` fixture, then validate the base URL response and perform a BFS exploration of every unique choice option (with loop tracking).
- If you change the API or the `test_story` fixture, update the assertions in `tests/story_api.rs` accordingly.

## Workflow Checklist for AI Agents

1. Read `knowledge/index.md` and the documents relevant to the task.
2. Make the code changes.
3. Run `cargo test` and ensure all tests pass.
4. Update or create the relevant `knowledge/` documents (including the index if a new document was added).
5. Ensure frontmatter metadata (`generated`, `sources`) reflects the change.
