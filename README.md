# Pluely 🚀

Pluely is a lightweight desktop AI assistant built with Tauri, React, TypeScript, and Rust.

It currently focuses on screenshot-based assistance, fast streaming AI responses, and an always-available overlay workflow. The project is evolving toward a meeting copilot / real-time assistant for technical work.

## Current status

| Area | What was done / improved | Status | Next |
|---|---|---:|---|
| Overlay window | Global shortcuts, window movement, fixed-step movement, macOS overlay stability cleanup | ✅ | UX polish |
| macOS opacity | Native window opacity is now stable in runtime updates | ✅ | More real-world testing |
| macOS passive mode | Runtime toggle temporarily disabled to avoid unstable panel/window behavior | ⚠️ | Restore with a safe implementation |
| Screenshot pipeline | Manual / Auto mode, multi-screenshot submit, stateless screenshot requests | ✅ | More real-world testing |
| Screenshot capture | Hide → capture → restore, overlay exclusion, position restore | ✅ | Cross-platform validation |
| AI responses | Streaming, keep-alive, warm-up, response controls | ✅ | More latency/quality tuning |
| Prompting | Screenshot prompt v2, coding override for screenshot short mode | ✅ | More benchmark tasks |
| Meeting mode | Not yet the main workflow | 🚧 | Real-time transcript + suggestions |

Note: The macOS passive mode toggle is temporarily unavailable.
It will be restored once a safe implementation is ready.

## Core features

- Overlay desktop window with global shortcuts
- Stability-first overlay UX on macOS
- Screenshot capture in full-screen or selection mode
- Manual and auto screenshot processing
- Multi-screenshot submission
- Streaming AI responses
- Configurable prompts and response settings
- Custom AI provider support
- Local-first architecture

## Quick start

```bash
npm install
npm run tauri dev
````

## Build

```bash
npm run tauri build
```
