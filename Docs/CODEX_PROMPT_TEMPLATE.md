# CODEX TASK TEMPLATE

## 1. Context

You are working inside a fork of the Pluely desktop application (Tauri + Rust + TypeScript).

Important:
- This is a personal fork.
- We prioritize fast implementation.
- We avoid touching licensing / payment logic.
- We minimize invasive refactoring.
- We aim to stay reasonably merge-safe with upstream.

Current platform targets:
- macOS
- Linux

Do NOT assume Windows-specific APIs unless explicitly required.

---

## 2. Architectural Constraints

STRICT RULES:

- Do NOT modify license checks or payment logic.
- Do NOT refactor unrelated code.
- Do NOT reformat the entire file.
- Do NOT rename existing functions unless explicitly instructed.
- Do NOT move files unless instructed.
- Only implement the requested task.
- If unsure — ask instead of guessing.

When adding new functionality:
- Prefer minimal changes.
- Reuse existing patterns in the repository.
- Follow existing naming conventions.
- Keep logic small and readable.

---

## 3. Task

[DESCRIBE ONE SINGLE TASK HERE]

Example:
"Add a new Tauri command `move_window_by(dx: i32, dy: i32)` that shifts the main window position by the given delta."

---

## 4. Acceptance Criteria

The task is complete only if:

- Code compiles without errors.
- No existing features are broken.
- No unrelated files were modified.
- No license-related logic was touched.
- Function works on macOS and Linux.
- Errors are handled properly (no unwrap() unless already used in surrounding code).

---

## 5. Output Requirements

- Keep explanations short.
- If modifying more than 3 files, briefly explain what and why.
- Do not modify unrelated files.
- Apply changes directly.

---

## 6. If Something Is Unclear

If required information is missing:

- Ask a clarifying question.
- Do NOT invent architecture.
- Do NOT assume file structure.
- Do NOT create new folders unless explicitly requested.