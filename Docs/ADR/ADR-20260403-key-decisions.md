# ADR: Runtime Passive Mode для main overlay

## Контекст
После перевода overlay в passive/click-through режим на macOS стало ясно, что одно только «всегда passive» поведение неудобно: окно перестаёт красть фокус, но пользователь теряет обычную mouse-интерактивность. Потребовался минимальный способ переключать режим без новой отдельной подсистемы.

## Решения
1. Хранить passive mode в backend state Tauri и управлять им через существующую команду `set_main_window_passive`, дополнив её чтением текущего состояния через `get_main_window_passive`.
2. Интегрировать переключение в уже существующую shortcut-систему как действие `toggle_passive_mode`, а не делать отдельный локальный hotkey-механизм.
3. Держать состояние сессионным: frontend синхронизируется с backend через runtime event `passive-mode-changed`, без `localStorage` persistence на текущем этапе.

## Альтернативы (кратко)
- Всегда оставлять окно passive: отклонено, т.к. ломает обычное взаимодействие с input и кнопками.
- Делать persistence сразу: отложено, т.к. сначала нужно проверить UX и платформенные edge cases на macOS.
- Добавлять отдельную кнопку поверх overlay: отклонено в этой итерации как более навязчивый UI.

## Последствия
- На macOS появился управляемый компромисс между stealth UX и обычной интерактивностью.
- Shortcut settings UI получает новое действие автоматически через общую конфигурацию shortcuts.
- Для пользователя появилось явное состояние режима в overlay UI, но только как индикатор, без дополнительной управляющей кнопки.
- По итогам первичного тестирования выявлен crash при переключении режима через shortcut на macOS; решение требует отдельного исправления.

## Ссылки/файлы
- `src-tauri/src/window.rs:13`
- `src-tauri/src/window.rs:159`
- `src-tauri/src/window.rs:187`
- `src-tauri/src/lib.rs:58`
- `src/config/shortcuts.ts:85`
- `src/contexts/app.context.tsx:150`
- `src/contexts/app.context.tsx:370`
- `src/pages/app/index.tsx:78`
