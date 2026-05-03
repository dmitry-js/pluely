# Pluely

Pluely is a desktop AI assistant built with Tauri, React, TypeScript, and Rust.

This fork focuses on fast iteration, practical desktop UX, screenshot-based assistance, system-audio interview support, and stable overlay behavior on macOS and Linux. The product direction is a real-time copilot for technical interviews, meetings, and screenshot/code-solving workflows.

## Current status

| Area | What was done / improved | Status | Next |
|---|---|---:|---|
| Overlay window | Global shortcuts, movement controls, keyboard-first UX, macOS stability cleanup | Stable | UX polish |
| macOS opacity | Native opacity updates are stabilized and avoid unsafe runtime re-panelization | Stable | More lifecycle testing |
| macOS passive mode | Enabled by default on launch, runtime toggle is temporarily disabled | Limited | Restore with a panel-safe implementation |
| Screenshot pipeline | Manual/auto capture, multi-screenshot submit, stateless screenshot requests | Stable | Cross-platform validation |
| Screenshot capture | Hide -> capture -> restore, overlay exclusion, position restore, debug logs behind env flag | Stable | Linux WM/compositor QA |
| AI responses | OpenAI backend-first path, streaming, keep-alive client, warm-up, response controls | Stable | Latency/quality tuning |
| System audio / STT | Serialized STT queue, pending transcript buffer, manual Cmd+Enter answer flow, optional auto-generate | Active | Improve STT reliability |
| Pause/Resume audio | Pause is a frontend gate with a short grace window; Stop remains a separate capture-session control | Active | Revisit backend-level pause/flush only with measured evidence |
| STT diagnostics | Dev-only WAV dumps for emitted VAD segments via `PLUELY_SAVE_STT_SEGMENTS` | Available | Use for VAD/STT quality debugging |

Note: the macOS passive mode runtime toggle is intentionally unavailable for now. Launch-time passive mode can be disabled for development with `PLUELY_PASSIVE_DEFAULT=false`.

## Core features

- Overlay desktop window with global shortcuts
- Stability-first overlay UX on macOS
- Screenshot capture in full-screen or selection mode
- Manual and auto screenshot processing
- Multi-screenshot submission
- Streaming AI responses
- Configurable prompts and response settings
- Custom AI provider support
- Built-in OpenAI Whisper STT through the Rust backend command path
- Custom STT provider support
- System-audio transcript accumulation with manual answer generation
- Pause/Resume and Stop controls for system-audio capture
- Local-first architecture

## Quick start

```bash
npm install
npm run tauri dev
```

For macOS development, disable launch-time click-through/passive mode when you need to interact with the overlay:

```bash
PLUELY_PASSIVE_DEFAULT=false npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Development diagnostics

Enable screenshot debug logs:

```bash
VITE_SCREENSHOT_DEBUG=true npm run tauri dev
```

Save emitted VAD/STT WAV segments for manual listening:

```bash
PLUELY_SAVE_STT_SEGMENTS=true npm run tauri dev
```

On macOS, STT debug WAV files are written to the system temp directory, for example:

```text
/var/folders/.../T/pluely-stt-segments/
```

The exact path is logged as:

```text
[stt-debug] saved speech segment path=...
```

Important: Pause does not stop backend audio capture/VAD. The backend can keep segmenting and saving WAV files while paused, while the frontend decides whether paused segments enter the STT pipeline. This is dev-only diagnostics; clean up `pluely-stt-segments` after long sessions.

## Known limitations

- Runtime passive mode toggle on macOS is disabled until a panel-safe implementation is ready.
- Direct desktop client -> OpenAI STT can be unreliable for users behind unstable VPN or restricted networks.
- A minimal STT proxy backend is a high-priority next step: the client should send audio to a Pluely backend, which forwards to the STT provider, handles timeout/retry/fallback, and returns normalized transcript/errors.
- Backend-level audio pause/flush is not currently kept. The current model is a frontend pause gate with a short grace window.

## Documentation

- `Docs/project_summary_ru.md` - current project summary in Russian.
- `Docs/ADR/` - architecture decision records.
- `Docs/NOTES/discussion_log_ru.md` - discussion log.
- `Docs/NOTES/todo_ru.md` - short priority TODO list.
