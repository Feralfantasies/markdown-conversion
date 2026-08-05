---
type: Reference
title: JSON Schema
description: Structure of the output JSON — Story, Page, and Choice types with field definitions.
tags: [schema, json, output]
generated:
  by: claude-4-20250514/anthropic
  at: "2025-07-28T17:10:00Z"
sources:
  - type: document
    source: src/models.rs
---

# JSON Schema

## Story

The root document containing the entire adventure collection.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `entry_point` | string | Yes | Filename (without extension) of the starting page |
| `pages` | `Page[]` | Yes | Array of all pages, sorted by `category` then `filename` |

## Page

A single node in the adventure graph.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `filename` | string | Yes | Filename without `.md` extension |
| `category` | string | Yes | Directory path relative to story root (e.g. `starting_zone/cottage`). Top-level files are `root`. |
| `title` | string | Yes | Page title from `#` heading (empty string if absent) |
| `story` | string | Yes | Narrative body text (empty string if absent) |
| `choices` | `Choice[]` | Yes | Array of selectable outcomes |

## Choice

A single selection option presented to the player.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `display` | string | Yes | Text shown on the button or option |
| `link` | string | Yes | Target page identifier. `self` means stay on current page. Extensions are stripped; `[[ ]]` unwrapped. |
| `cost` | number \| null | Optional | Stamina cost |
| `msg` | string \| null | Optional | Additional message to display. May contain a `{}` placeholder filled using `msg_var` |
| `msg_var` | string \| null | Optional | Variable name whose value fills the `{}` placeholder in `msg` |
| `functions` | string[] \| null | Optional | Actions for the client to execute (e.g. `["set_character_class(road_warrior)", "set_background(road.png)"]`) |
| `func` | string[] \| null | Optional | Legacy single-function syntax (e.g. `["set_stats=warrior"]`) |
| `req` | string[] \| null | Optional | Required inventory items |
| `add_item` | string[] \| null | Optional | Items added on selection |
| `remove_item` | string[] \| null | Optional | Items consumed on selection |
| `check` | string \| null | Optional | Skill check specification (e.g. `"INT:15"`) |

> Client responsibility: the API is stateless and only transports these
> action payloads. The client processes `functions`, applies `cost`, renders
> `msg` (substituting `msg_var` into the `{}` placeholder), and evaluates
> `req` / `add_item` / `remove_item` / `check` as needed.

## Minimal Example

```json
{
  "entry_point": "character_creator",
  "pages": [
    {
      "filename": "start",
      "category": "root",
      "title": "The Beginning",
      "story": "You stand at a crossroads.",
      "choices": [
        {
          "display": "Go left",
          "link": "left_path",
          "cost": 1.0
        },
        {
          "display": "Go right",
          "link": "right_path",
          "cost": 1.0,
          "msg": "Dangerous"
        },
        {
          "display": "Explore the road",
          "link": "road",
          "cost": 10.0,
          "functions": [
            "set_character_class(road_warrior)",
            "set_background(road.png)"
          ]
        },
        {
          "display": "Check your watch",
          "link": "self",
          "cost": 0.0,
          "functions": ["current_time=get_time()"],
          "msg": "You look down at your watch to see the time, its currently {}",
          "msg_var": "current_time"
        }
      ]
    }
  ]
}
```
