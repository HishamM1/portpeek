---
name: codex-review
description: Review the current git diff (or a named target) with Codex using gpt-5.6-terra at medium reasoning effort. Read-only — reports findings, makes no edits.
---

# Codex Review

Runs an external Codex review pass over the working changes. Model and effort are
fixed: `gpt-5.6-terra`, reasoning effort `medium`. Read-only sandbox — Codex never
edits the tree, it only reports.

## Steps

1. Determine the review target. Default is the committed + staged + unstaged diff
   of the current branch vs its merge-base with `main`. Use `git diff HEAD` (not a
   bare `git diff`) so staged changes are included:
   ```bash
   git diff main...HEAD && git diff HEAD
   ```
   If the user named a path, PR, or commit range, scope to that instead. Name the
   exact scope in the Codex prompt (branch + merge-base, or the path/range) so
   Codex reviews the same diff you're looking at, not a different tree.

2. Run Codex read-only with the fixed model and effort. Always `--skip-git-repo-check`.
   Run from the repository root so Codex reads the right tree — don't hardcode an
   absolute path; if you must pass `-C`, resolve it with `git rev-parse --show-toplevel`:
   ```bash
   codex exec --skip-git-repo-check \
     -m gpt-5.6-terra \
     --config model_reasoning_effort="medium" \
     --sandbox read-only \
     -C "$(git rev-parse --show-toplevel)" \
     "Review the diff of this branch vs its merge-base with main (committed + staged + unstaged) for correctness bugs, missing edge cases, and unnecessary complexity. Focus on the changed lines. List findings most-severe first with file:line. Do not restate what the code does; only report problems. If the diff is clean, say so plainly." \
     2>/dev/null
   ```
   `2>/dev/null` suppresses Codex's thinking tokens (stderr). But it also hides
   auth/model/path failures — so if Codex exits non-zero or prints nothing, re-run
   **without** `2>/dev/null` to see the actual error before treating the review as
   "clean".

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
