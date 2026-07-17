---
name: codex-review
description: Review the current git diff (or a named target) with Codex using gpt-5.6-terra at medium reasoning effort. Read-only — reports findings, makes no edits.
---

# Codex Review

Runs an external Codex review pass over the working changes. Model and effort are
fixed: `gpt-5.6-terra`, reasoning effort `medium`. Read-only sandbox — Codex never
edits the tree, it only reports.

## Steps

1. Determine the review target. Default is the uncommitted + committed diff of the
   current branch vs its merge-base with `main`:
   ```bash
   git diff main...HEAD && git diff
   ```
   If the user named a path, PR, or commit range, scope the prompt to that instead.

2. Run Codex read-only with the fixed model and effort. Always `--skip-git-repo-check`,
   always append `2>/dev/null` to suppress thinking tokens:
   ```bash
   codex exec --skip-git-repo-check \
     -m gpt-5.6-terra \
     --config model_reasoning_effort="medium" \
     --sandbox read-only \
     -C "C:/Projects/portpeek" \
     "Review the current git diff on this branch for correctness bugs, missing edge cases, and unnecessary complexity. Focus on the changed lines. List findings most-severe first with file:line. Do not restate what the code does; only report problems. If the diff is clean, say so plainly." \
     2>/dev/null
   ```

3. Read Codex's findings critically — it is a peer, not an authority (see the
   base codex skill's "Critical Evaluation" notes). Verify each finding against the
   actual code before acting. Discard false positives and say why.

4. Summarize surviving findings for the user, then fix or defer each with their
   direction.

## Resume

To continue the same review (e.g. after fixes, to re-check):
```bash
echo "your follow-up" | codex exec --skip-git-repo-check resume --last 2>/dev/null
```
No flags on resume — it inherits model, effort, and sandbox from the original run.
