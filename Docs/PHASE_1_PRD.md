# Phase 1 PRD – Window Control Enhancements

## Objective

Implement basic window control enhancements for personal use:

1. Move window using keyboard shortcuts.
2. Adjust window opacity using keyboard shortcuts.

No modifications to paid features or license logic.

---

## Scope

### Feature 1 – Window Movement

Add global shortcuts:

- Cmd + Shift + Left  → Move window left
- Cmd + Shift + Right → Move window right
- Cmd + Shift + Up    → Move window up
- Cmd + Shift + Down  → Move window down

Behavior:
- Move by fixed step (e.g. 20px).
- Prevent negative window positions if possible.
- Should work on macOS and Linux.

---

### Feature 2 – Opacity Control

Add global shortcuts:

- Cmd + Shift + PageUp   → Increase opacity
- Cmd + Shift + PageDown → Decrease opacity

Behavior:
- Opacity range: 0.3 – 1.0
- Step: 0.1
- Clamp values safely.
- Apply immediately to main window.
- Persist value only if trivial (optional in Phase 1).

---

## Non-Goals

- No UI settings panel.
- No settings persistence system (unless trivial).
- No modification of paid feature logic.
- No redesign of window management system.
- No cross-platform Windows support.

---

## Technical Approach

- Add new Tauri commands in Rust.
- Register global shortcuts.
- Use Tauri window API:
  - get_position
  - set_position
  - set_opacity
- Keep changes minimal.
- Avoid modifying existing shortcut system unless necessary.

---

## Risks

- Shortcut conflicts with existing registration.
- macOS permission issues.
- Wayland limitations on some Linux environments.

If shortcut conflict occurs:
- Detect and log.
- Consider unregistering previous binding.

---

## Definition of Done

- App builds successfully.
- Shortcuts work reliably.
- No regressions in existing functionality.
- No changes in license/payment logic.