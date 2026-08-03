---
type: Reference
title: API Reference
description: HTTP API endpoints, request/response formats, and client interaction patterns for the story service.
tags: [api, http, endpoints]
generated:
  by: claude-4-20250514/anthropic
  at: "2025-07-28T17:30:00Z"
sources:
  - type: document
    source: src/main.rs
---

# API Reference

The story service exposes a simple REST API for serving choose-your-own-adventure pages. It is **stateless** — no game state is tracked on the server. Clients maintain their own state (current page, inventory, stats).

## Base URL

```
http://<host>:<port>
```

Default port: `3000`

## Endpoints

### GET / — Entry Page

Returns the starting page for a new game session.

| Aspect | Value |
|--------|-------|
| Method | `GET` |
| URL | `/` |
| Response | `Page` JSON object |
| Status codes | `200 OK`, `404 Not Found` |

```bash
curl http://localhost:3000/
```

### GET /pages — Page List

Returns a sorted list of all available page IDs.

| Aspect | Value |
|--------|-------|
| Method | `GET` |
| URL | `/pages` |
| Response | `string[]` (sorted filenames) |
| Status codes | `200 OK` |

```bash
curl http://localhost:3000/pages
```

### GET /pages/{id} — Specific Page

Returns a page by its filename (without `.md` extension).

| Aspect | Value |
|--------|-------|
| Method | `GET` |
| URL | `/pages/{id}` |
| Response | `Page` JSON object |
| Status codes | `200 OK`, `404 Not Found` |

```bash
curl http://localhost:3000/pages/start_room
```

### 404 Fallback

Any unhandled route returns 404:

```
Endpoint not found. Try GET / or GET /pages/{id}.
```

## Response Format

### Page Object

```json
{
  "filename": "start_room",
  "category": "root",
  "title": "The Awakening",
  "story": "You slowly awaken...",
  "choices": [
    {
      "display": "Go outside",
      "link": "outside_cottage",
      "cost": 1.0
    }
  ]
}
```

See [JSON Schema](json_schema.md) for full type definitions.

## Client Flow

```
┌─────────────┐
│ Frontend App │
└──────┬──────┘
       │ 1. GET /
       ▼
┌─────────────┐     2. Return entry page
│ Story Service │ ←── GET /pages/start_room
└─────────────┘     3. Return page data
       │           4. Client renders page
       │           5. Player selects choice
       │           6. Client tracks state
       │           7. Client fetches next
       │              page via choice.link
       └─────────────────────────────┘
```

## CORS

Cross-Origin Resource Sharing headers are included for browser-based clients. `Access-Control-Allow-Origin: *` is set on all responses.

## State Management

The service is **read-only**. Clients are responsible for:

- Tracking current page ID
- Maintaining inventory (items, stats)
- Evaluating choice conditions (`req`, `check`)
- Applying choices (`add_item`, `remove_item`, `func`)
