---
type: Example
title: Example Input / Output
description: Concrete walkthrough of input markdown files and their JSON output.
tags: [example, walkthrough]
generated:
  by: claude-opus-4-20250514/anthropic
  at: "2025-07-28T17:10:00Z"
sources:
  - type: repository
    source: /home/acleveland/sundown/story
---

# Example Input / Output

## Input

Source file `start_room.md`:

```markdown
# The Awakening

You slowly awaken, your face is warm from the sunlight seeping
through the small window beside the bed. Looking around you don't
recognise this room, it is full of strange furniture and odd artefacts.

### Choices
- [Search the cupboards](starting_zone/cottage/cupboard_investigation) {cost=5}
- [Brew some potions](misc/alchemy_recipies) {cost=1}
- [Burn some Stamina](self) {cost=100, msg="Just burning stamina"}
```

## Output (excerpt)

```json
{
  "entry_point": "character_creator",
  "pages": [
    {
      "filename": "start_room",
      "category": "root",
      "title": "The Awakening",
      "story": "You slowly awaken, your face is warm...",
      "choices": [
        {
          "display": "Search the cupboards",
          "link": "starting_zone/cottage/cupboard_investigation",
          "cost": 5.0
        },
        {
          "display": "Brew some potions",
          "link": "misc/alchemy_recipies",
          "cost": 1.0
        },
        {
          "display": "Burn some Stamina",
          "link": "self",
          "cost": 100.0,
          "msg": "Just burning stamina"
        }
      ]
    }
  ]
}
```

## Observations

1. **Category** — `start_room.md` sits at the story root, so `category` is `root`. A file at `starting_zone/cottage/bedroom.md` would be `starting_zone/cottage`.
2. **Links** — relative paths are preserved verbatim; `.md` extensions are stripped.
3. **Self-reference** — `self` is preserved to indicate the player stays on the same page (e.g., retry, craft).
4. **Optional metadata** — `msg` only appears when specified in the source; absent keys are omitted from JSON (`skip_serializing_if`).
