---
type: Policy
title: Development Policy
description: Branching strategy and pull-request workflow rules for the markdown-converter project.
tags: [policy, workflow, branching]
generated:
  by: claude-opus-4-20250514/anthropic
  at: "2025-07-28T17:45:00Z"
---

# Development Policy

## Branching Strategy

### Rule: No Direct Commits to `main`

All development work **must** be carried out on feature branches. Nothing should ever be committed directly into `main`.

```text
main ← PR merges only
 ├─ feature/<name> ← development happens here
 └─ fix/<name>    ← bug fixes here
```

### Workflow

1. **Create a feature branch** from `main`:

   ```bash
   git checkout -b feature/<descriptive-name>
   ```

2. **Develop and commit** on the feature branch

3. **Push** the branch:

   ```bash
   git push origin feature/<descriptive-name>
   ```

4. **Raise a PR** to merge into `main`:

   ```bash
   gh pr create \
     --base main \
     --head feature/<descriptive-name> \
     --title "Short title" \
     --body "Description of changes"
   ```

5. **Merge** only after review/approval

6. **Delete** the feature branch (optional):

   ```bash
   git branch -d feature/<descriptive-name>
   git push origin --delete feature/<descriptive-name>
   ```

### Enforcement

This is not just a preference — **it is mandatory**. Future development on this repo must follow this convention without exception.
